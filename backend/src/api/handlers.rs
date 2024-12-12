use axum::{
    extract::{State, Query},
    response::{Json, IntoResponse},
    http::StatusCode,
};
use time::OffsetDateTime;
use crate::{AppState, 
    TimeRangeQuery, RunningTotal, IntervalAPIData, 
    PoolTotal, PoolTotalsResponse, PoolTotalsQuery,
    HealthResponse, MERGE_BLOCK, POOL_NAMES, POOL_ADDRESSES,
    MedianLVRQuery, MedianLVRResponse, PoolMedianLVR,
    MaxLVRQuery, MaxLVRResponse,
    LVRRatioQuery, LVRRatioResponse, 
    HistogramBucket, HistogramQuery, HistogramResponse,
    NonZeroProportionQuery, NonZeroProportionResponse,
    MarkoutRatio, LVRTotals};
use tracing::{error, debug, info, warn};
use futures::StreamExt;
use std::{sync::Arc, collections::{HashSet, HashMap}};
use parquet::arrow::arrow_reader::ParquetRecordBatchReader;
use arrow::array::{StringArray, UInt64Array, Int64Array, Float64Array,Array};
use arrow::datatypes::DataType;
use arrow::record_batch::RecordBatch;


const BLOCKS_PER_INTERVAL: u64 = 7200;
const FINAL_PARTIAL_BLOCKS: u64 = 5808;
const FINAL_INTERVAL_FILE: &str = "19857392_20000000.parquet";

pub async fn health_check() -> impl IntoResponse {
    let response = HealthResponse {
        status: "OK",
        version: env!("CARGO_PKG_VERSION"),
        timestamp: OffsetDateTime::now_utc().to_string(),
    };

    (StatusCode::OK, Json(response))
}

fn get_valid_pools() -> HashSet<String> {
    POOL_ADDRESSES.iter()
        .map(|&addr| addr.to_lowercase())
        .collect()
}

fn calculate_block_number(base_block: u64, interval_id: u64, file_path: &str) -> u64 {
    // Extract the file's starting block from the full path
    let file_start = file_path
        .split("intervals/")  // Split on intervals directory
        .nth(1)              // Take the part after "intervals/"
        .and_then(|name| name.trim_end_matches(".parquet").split('_').next())  // Get first number before underscore
        .and_then(|num| num.parse::<u64>().ok())
        .unwrap_or(base_block);

    // Check if this is the final interval file and it's interval 19
    if file_path.ends_with(FINAL_INTERVAL_FILE) && interval_id == 19 {
        // For the final interval in the final file, use partial blocks
        file_start + (interval_id * BLOCKS_PER_INTERVAL) + FINAL_PARTIAL_BLOCKS
    } else {
        // For all other intervals, use the file's start block
        file_start + (interval_id * BLOCKS_PER_INTERVAL)
    }
}


pub async fn get_running_total(
    State(state): State<Arc<AppState>>,
    Query(params): Query<TimeRangeQuery>,
) -> Result<Json<Vec<RunningTotal>>, StatusCode> {
    let start_block = params.start_block.unwrap_or(*MERGE_BLOCK);
    let end_block = params.end_block.unwrap_or(20_000_000);
    let is_aggregate = params.aggregate.unwrap_or(false);
    
    // Validate pool parameter when not aggregating
    if !is_aggregate {
        if let Some(ref pool) = params.pool {
            let valid_pools = get_valid_pools();
            if !valid_pools.contains(&pool.to_lowercase()) {
                warn!("Invalid pool address provided: {}", pool);
                return Err(StatusCode::BAD_REQUEST);
            }
        } else {
            warn!("Pool parameter required when not aggregating");
            return Err(StatusCode::BAD_REQUEST);
        }
    }
    
    info!(
        "Fetching {} running total for blocks {} to {}{}", 
        if is_aggregate { "aggregated" } else { "individual" },
        start_block, 
        end_block,
        params.pool.as_ref().map_or(String::new(), |p| format!(", pool: {}", p))
    );
    
    let valid_pools = get_valid_pools();
    let mut interval_totals: HashMap<(u64, u64, String, Option<String>), IntervalAPIData> = HashMap::new();
    let intervals_path = object_store::path::Path::from("intervals");
    
    let mut files_processed = 0;
    let mut files_skipped = 0;
    info!("Reading interval data from: {:?}", intervals_path);
    let mut interval_files = state.store.list(Some(&intervals_path));

    while let Some(meta_result) = interval_files.next().await {
        let meta = meta_result.map_err(|e| {
            error!("Failed to get file metadata: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let file_path = meta.location.to_string();
        files_processed += 1;
        
        // Skip files outside our range
        if let Some(file_name) = file_path.split('/').last() {
            let parts: Vec<&str> = file_name.split('_').collect();
            if parts.len() == 2 {
                if let (Ok(file_start), Ok(file_end)) = (
                    parts[0].parse::<u64>(),
                    parts[1].trim_end_matches(".parquet").parse::<u64>()
                ) {
                    if file_start > end_block || file_end < start_block {
                        files_skipped += 1;
                        continue;
                    }
                }
            }
        }

        debug!("Processing file: {}", file_path);

        let bytes = state.store.get(&meta.location)
            .await
            .map_err(|e| {
                error!("Failed to read file content: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .bytes()
            .await
            .map_err(|e| {
                error!("Failed to get file bytes: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        let record_reader = ParquetRecordBatchReader::try_new(bytes, 1024)
            .map_err(|e| {
                error!("Failed to create Parquet reader: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        for batch_result in record_reader {
            let batch = batch_result.map_err(|e| {
                error!("Failed to read batch: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            let interval_ids = get_uint64_column(&batch, "interval_id")?;
            let markout_times = get_string_column(&batch, "markout_time")?;
            let pool_addresses = get_string_column(&batch, "pair_address")?;
            let total_lvr_cents = get_uint64_column(&batch, "total_lvr_cents")?;
            let non_zero_counts = get_uint64_column(&batch, "non_zero_count")?;
            
            for i in 0..batch.num_rows() {
                if total_lvr_cents.is_null(i) || non_zero_counts.value(i) == 0 {
                    continue;
                }

                let pool_address = pool_addresses.value(i).to_lowercase();
                
                // Skip if not matching the specified pool (when not aggregating)
                if !is_aggregate && params.pool.as_ref().map_or(false, |p| p.to_lowercase() != pool_address) {
                    continue;
                }
                
                if !valid_pools.contains(&pool_address) {
                    continue;
                }

                let interval_id = interval_ids.value(i);
                let markout_time = markout_times.value(i).to_string();
                let lvr_cents = total_lvr_cents.value(i);
                // Apply markout time filter if specified
                if let Some(ref filter) = params.markout_time {
                    if &markout_time != filter {
                        continue;
                    }
                }

                let block_number = calculate_block_number(start_block, interval_id, &file_path);

                if block_number >= start_block && block_number <= end_block {
                    let file_start = file_path
                        .split("intervals/")
                        .nth(1)
                        .and_then(|name| name.trim_end_matches(".parquet").split('_').next())
                        .and_then(|num| num.parse::<u64>().ok())
                        .unwrap_or(start_block);

                    let key = if is_aggregate {
                        (file_start, interval_id, markout_time, None)
                    } else {
                        (file_start, interval_id, markout_time.clone(), Some(pool_address))
                    };

                    interval_totals
                        .entry(key)
                        .and_modify(|data| data.total = data.total.saturating_add(lvr_cents))
                        .or_insert(IntervalAPIData {
                            total: lvr_cents,
                            file_path: file_path.clone(),
                        });
                }
            }
        }
    }

    let results = process_interval_totals(interval_totals, start_block, end_block, is_aggregate);
    // Filter results for specific pool if needed
    let results = if !is_aggregate {
        results.into_iter()
            .filter(|r| {
                if let (Some(requested_pool), Some(result_address)) = (
                    params.pool.as_ref(),
                    r.pool_address.as_ref()
                ) {
                    requested_pool.to_lowercase() == result_address.to_lowercase()
                } else {
                    false
                }
            })
            .collect()
    } else {
        results
    };
    info!("Returning {} running total data points", results.len());
    Ok(Json(results))
}

fn process_interval_totals(
    interval_totals: HashMap<(u64, u64, String, Option<String>), IntervalAPIData>,
    start_block: u64,
    end_block: u64,
    is_aggregate: bool,
) -> Vec<RunningTotal> {
    let mut results = Vec::new();
    let mut last_totals: HashMap<String, u64> = HashMap::new();
    
    // Convert to sorted Vec for chronological processing
    let mut sorted_entries: Vec<_> = interval_totals.into_iter().collect();
    sorted_entries.sort_by(|a, b| {
        let block_a = a.0.0 + (a.0.1 * BLOCKS_PER_INTERVAL);
        let block_b = b.0.0 + (b.0.1 * BLOCKS_PER_INTERVAL);
        block_a.cmp(&block_b)
    });

    debug!("Processing {} sorted interval entries", sorted_entries.len());

    for ((file_start, interval_id, markout, pool_opt), data) in sorted_entries {
        let block_number = if data.file_path.ends_with(FINAL_INTERVAL_FILE) && interval_id == 19 {
            file_start + (interval_id * BLOCKS_PER_INTERVAL) + FINAL_PARTIAL_BLOCKS
        } else {
            file_start + (interval_id * BLOCKS_PER_INTERVAL)
        };

        if block_number >= start_block && block_number <= end_block {
            if is_aggregate {
                let current_total = last_totals
                    .entry(markout.clone())
                    .and_modify(|total| *total = total.saturating_add(data.total))
                    .or_insert(data.total);

                if results.last().map_or(true, |last: &RunningTotal| 
                    last.markout != markout || last.running_total_cents != *current_total
                ) {
                    results.push(RunningTotal {
                        block_number,
                        markout,
                        pool_name: None,
                        pool_address: None,
                        running_total_cents: *current_total,
                    });
                }
            } else if let Some(pool_address) = pool_opt {
                let pool_name = POOL_NAMES
                    .iter()
                    .find(|(addr, _)| addr.to_lowercase() == pool_address)
                    .map(|(_, name)| name.to_string())
                    .unwrap_or_else(|| pool_address.clone());

                let key = format!("{}_{}", markout, pool_name);
                let current_total = last_totals
                    .entry(key.clone())
                    .and_modify(|total| *total = total.saturating_add(data.total))
                    .or_insert(data.total);

                if results.last().map_or(true, |last: &RunningTotal| 
                    last.markout != markout || 
                    last.pool_name.as_ref() != Some(&pool_name) ||
                    last.running_total_cents != *current_total
                ) {
                    results.push(RunningTotal {
                        block_number,
                        markout,
                        pool_name: Some(pool_name),
                        pool_address: Some(pool_address),
                        running_total_cents: *current_total,
                    });
                }
            }
        }
    }

    // Sort results
    results.sort_by(|a, b| {
        a.block_number
            .cmp(&b.block_number)
            .then_with(|| a.markout.to_lowercase().cmp(&b.markout.to_lowercase()))
            .then(a.pool_name.cmp(&b.pool_name))
    });

    results
}

pub async fn get_lvr_ratios(
    State(state): State<Arc<AppState>>,
    Query(params): Query<LVRRatioQuery>,
) -> Result<Json<LVRRatioResponse>, StatusCode> {
    info!("Fetching LVR ratios with params: {:?}", params);
    
    let valid_pools = get_valid_pools();
    let mut totals = LVRTotals {
        realized: 0,
        theoretical: HashMap::new(),
    };

    let mut files_processed = 0;
    let intervals_path = object_store::path::Path::from("intervals");
    let mut interval_files = state.store.list(Some(&intervals_path));

    while let Some(meta_result) = interval_files.next().await {
        files_processed += 1;
        let meta = meta_result.map_err(|e| {
            error!("Failed to get file metadata: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        debug!("Processing interval file {}: {}", files_processed, meta.location);

        let bytes = state.store.get(&meta.location)
            .await
            .map_err(|e| {
                error!("Failed to read file: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .bytes()
            .await
            .map_err(|e| {
                error!("Failed to get bytes: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        let record_reader = ParquetRecordBatchReader::try_new(bytes, 1024)
            .map_err(|e| {
                error!("Failed to create Parquet reader: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        for batch_result in record_reader {
            let batch = batch_result.map_err(|e| {
                error!("Failed to read batch: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            let markout_times = get_string_column(&batch, "markout_time")?;
            let pool_addresses = get_string_column(&batch, "pair_address")?;
            let total_lvr_cents = get_uint64_column(&batch, "total_lvr_cents")?;
            let non_zero_counts = get_uint64_column(&batch, "non_zero_count")?;

            for i in 0..batch.num_rows() {
                if total_lvr_cents.is_null(i) || non_zero_counts.value(i) == 0 {
                    continue;
                }

                let pool_address = pool_addresses.value(i).to_lowercase();
                if !valid_pools.contains(&pool_address) {
                    continue;
                }

                let markout_time = markout_times.value(i);
                let lvr_cents = total_lvr_cents.value(i);

                // Only include intervals that had actual activity
                if lvr_cents > 0 {
                    if markout_time == "brontes" {
                        totals.realized = totals.realized.saturating_add(lvr_cents);
                        debug!("Added {} cents to realized total", lvr_cents);
                    } else {
                        totals.theoretical
                            .entry(markout_time.to_string())
                            .and_modify(|e| *e = e.saturating_add(lvr_cents))
                            .or_insert(lvr_cents);
                        debug!("Added {} cents to theoretical total for markout {}", lvr_cents, markout_time);
                    }
                }
            }
        }
    }

    info!(
        "Processed {} files. Found realized total of {} cents and {} theoretical markout times",
        files_processed,
        totals.realized,
        totals.theoretical.len()
    );

    let ratios = calculate_lvr_ratios(totals);
    
    info!("Calculated {} LVR ratios", ratios.len());
    Ok(Json(LVRRatioResponse { ratios }))
}

// calculate_lvr_ratios remains the same
fn calculate_lvr_ratios(totals: LVRTotals) -> Vec<MarkoutRatio> {
    let mut ratios = Vec::new();
    
    // Only calculate ratios if we have realized LVR data
    if totals.realized > 0 {
        for (markout_time, theoretical_lvr) in totals.theoretical {
            // Only include ratios where we have theoretical data
            if theoretical_lvr > 0 {
                // Calculate the ratio as a percentage
                let ratio = (totals.realized as f64 / theoretical_lvr as f64) * 100.0;
                let capped_ratio = ratio.min(100.0);
                
                debug!(
                    "Calculated ratio for {}: {:.2}% (realized: {}, theoretical: {})",
                    markout_time, capped_ratio, totals.realized, theoretical_lvr
                );
                
                ratios.push(MarkoutRatio {
                    markout_time,
                    ratio: capped_ratio,
                    realized_lvr_cents: totals.realized,
                    theoretical_lvr_cents: theoretical_lvr,
                });
            }
        }
    }

    // Sort by markout time for consistent ordering
    ratios.sort_by(|a, b| {
        if a.markout_time == "brontes" {
            std::cmp::Ordering::Greater
        } else if b.markout_time == "brontes" {
            std::cmp::Ordering::Less
        } else {
            match (a.markout_time.parse::<f64>(), b.markout_time.parse::<f64>()) {
                (Ok(a_val), Ok(b_val)) => a_val.partial_cmp(&b_val).unwrap_or(std::cmp::Ordering::Equal),
                _ => a.markout_time.cmp(&b.markout_time)
            }
        }
    });

    ratios
}

pub async fn get_pool_totals(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PoolTotalsQuery>,
) -> Result<Json<PoolTotalsResponse>, StatusCode> {
    let markout_time = params.markout_time.unwrap_or_else(|| String::from("brontes"));
    
    info!("Fetching pool totals for markout_time: {}", markout_time);
    let valid_pools = get_valid_pools();
    debug!("Number of valid pools: {}", valid_pools.len());
    
    let mut pool_totals = Vec::new();
    let mut files_processed = 0;
    let mut files_matched = 0;
    
    let checkpoints_path = object_store::path::Path::from("checkpoints");
    let mut checkpoint_files = state.store.list(Some(&checkpoints_path));
    
    while let Some(meta_result) = checkpoint_files.next().await {
        files_processed += 1;
        let meta = meta_result.map_err(|e| {
            error!("Failed to get file metadata: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        
        let file_path = meta.location.to_string();
        
        if !file_path.ends_with(&format!("_{}.parquet", markout_time)) {
            continue;
        }
        
        files_matched += 1;
        debug!("Processing checkpoint file: {}", file_path);

        let bytes = state.store.get(&meta.location)
            .await
            .map_err(|e| {
                error!("Failed to read checkpoint file: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .bytes()
            .await
            .map_err(|e| {
                error!("Failed to get file bytes: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        let record_reader = ParquetRecordBatchReader::try_new(bytes, 1)
            .map_err(|e| {
                error!("Failed to create Parquet reader: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        for batch_result in record_reader {
            let batch = batch_result.map_err(|e| {
                error!("Failed to read batch: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            // Get the column index and inspect its type
            let running_total_idx = batch.schema().index_of("running_total").map_err(|e| {
                error!("Failed to get running_total column index: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            let running_total = {
                let column = batch.column(running_total_idx);
                match column.data_type() {
                    DataType::Int64 => {
                        column.as_any()
                            .downcast_ref::<Int64Array>()
                            .map(|arr| arr.value(0))
                            .ok_or_else(|| {
                                error!("Failed to cast running_total as Int64Array");
                                StatusCode::INTERNAL_SERVER_ERROR
                            })?
                    },
                    DataType::UInt64 => {
                        column.as_any()
                            .downcast_ref::<UInt64Array>()
                            .map(|arr| arr.value(0) as i64)
                            .ok_or_else(|| {
                                error!("Failed to cast running_total as UInt64Array");
                                StatusCode::INTERNAL_SERVER_ERROR
                            })?
                    },
                    other => {
                        error!("Unexpected type for running_total: {:?}", other);
                        return Err(StatusCode::INTERNAL_SERVER_ERROR);
                    }
                }
            };

            let pair_addresses = get_string_column(&batch, "pair_address")?;
            let non_zero_proportions = get_float64_column(&batch, "non_zero_proportion")?;

            if batch.num_rows() == 0 {
                warn!("Empty batch in file: {}", file_path);
                continue;
            }

            let pair_address = pair_addresses.value(0).to_lowercase();
            if !valid_pools.contains(&pair_address) {
                debug!("Skipping invalid pool address: {}", pair_address);
                continue;
            }

            let non_zero_proportion = non_zero_proportions.value(0);

            // Only include pools with actual activity
            if non_zero_proportion > 0.0 {
                let pool_name = POOL_NAMES
                    .iter()
                    .find(|(addr, _)| addr.to_lowercase() == pair_address)
                    .map(|(_, name)| name.to_string())
                    .unwrap_or_else(|| pair_address.clone());

                debug!(
                    "Adding pool {} with total {} cents (non-zero proportion: {:.2}%)", 
                    pool_name, 
                    running_total,
                    non_zero_proportion * 100.0
                );

                pool_totals.push(PoolTotal {
                    pool_name,
                    pool_address: pair_address,
                    total_lvr_cents: running_total.unsigned_abs(),
                });
            }
        }
    }

    // Sort by total_lvr_cents descending
    pool_totals.sort_by(|a, b| b.total_lvr_cents.cmp(&a.total_lvr_cents));

    info!(
        "Process summary: Processed {} files, matched {} files, found {} active pools with data for markout time {}", 
        files_processed, 
        files_matched, 
        pool_totals.len(),
        markout_time
    );

    if pool_totals.is_empty() {
        warn!(
            "No pool totals found for markout_time: {}. This might indicate missing data or no activity.", 
            markout_time
        );
    }

    Ok(Json(PoolTotalsResponse { totals: pool_totals }))
}

// Helper functions remain the same
fn get_string_column<'a>(batch: &'a RecordBatch, name: &str) -> Result<&'a StringArray, StatusCode> {
    batch
        .column(batch.schema().index_of(name).map_err(|e| {
            error!("Failed to get {} column index: {}", name, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| {
            error!("Failed to cast {} column to StringArray", name);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

fn get_float64_column<'a>(batch: &'a RecordBatch, name: &str) -> Result<&'a Float64Array, StatusCode> {
    batch
        .column(batch.schema().index_of(name).map_err(|e| {
            error!("Failed to get {} column index: {}", name, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
        .as_any()
        .downcast_ref::<Float64Array>()
        .ok_or_else(|| {
            error!("Failed to cast {} column to Float64Array", name);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}


pub async fn get_pool_medians(
    State(state): State<Arc<AppState>>,
    Query(params): Query<MedianLVRQuery>,
) -> Result<Json<MedianLVRResponse>, StatusCode> {
    let markout_time = params.markout_time.unwrap_or_else(|| String::from("brontes"));
    
    info!("Fetching pool medians for markout_time: {}", markout_time);
    let valid_pools = get_valid_pools();
    
    // Track medians for each pool
    let mut pool_medians: HashMap<String, Vec<u64>> = HashMap::new();
    let mut files_processed = 0;
    
    let intervals_path = object_store::path::Path::from("intervals");
    let mut interval_files = state.store.list(Some(&intervals_path));

    while let Some(meta_result) = interval_files.next().await {
        let meta = meta_result.map_err(|e| {
            error!("Failed to get file metadata: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        files_processed += 1;
        debug!("Processing interval file {}: {}", files_processed, meta.location);

        let bytes = state.store.get(&meta.location)
            .await
            .map_err(|e| {
                error!("Failed to read file content: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .bytes()
            .await
            .map_err(|e| {
                error!("Failed to get file bytes: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        let record_reader = ParquetRecordBatchReader::try_new(bytes, 1024)
            .map_err(|e| {
                error!("Failed to create Parquet reader: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        for batch_result in record_reader {
            let batch = batch_result.map_err(|e| {
                error!("Failed to read batch: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            // Get column indices with proper error handling
            let markout_times = get_string_column(&batch, "markout_time")?;
            let pool_addresses = get_string_column(&batch, "pair_address")?;
            let median_lvrs = get_uint64_column(&batch, "median_lvr_cents")?;
            let non_zero_counts = get_uint64_column(&batch, "non_zero_count")?;

            // Process each row
            for i in 0..batch.num_rows() {
                let current_markout = markout_times.value(i);
                if current_markout != markout_time {
                    continue;
                }

                let pool_address = pool_addresses.value(i).to_lowercase();
                if !valid_pools.contains(&pool_address) {
                    continue;
                }

                let median_lvr = median_lvrs.value(i);
                let non_zero_count = non_zero_counts.value(i);

                // Only include medians from intervals with actual transactions
                if median_lvr > 0 && non_zero_count > 0 {
                    pool_medians
                        .entry(pool_address)
                        .or_default()
                        .push(median_lvr);
                }
            }
        }
    }

    debug!(
        "Processed {} files, found data for {} pools",
        files_processed,
        pool_medians.len()
    );

    // Calculate final medians for each pool
    let mut final_medians = Vec::new();
    for (pool_address, medians) in pool_medians {
        if !medians.is_empty() {
            let mut sorted_medians = medians;
            sorted_medians.sort_unstable();

            // Calculate median, ensuring we have enough data points
            let overall_median = if sorted_medians.len() >= 2 {
                if sorted_medians.len() % 2 == 0 {
                    let mid = sorted_medians.len() / 2;
                    (sorted_medians[mid - 1] + sorted_medians[mid]) / 2
                } else {
                    sorted_medians[sorted_medians.len() / 2]
                }
            } else if sorted_medians.len() == 1 {
                sorted_medians[0]
            } else {
                continue; // Skip pools with no valid medians
            };

            // Get pool name from constants
            let pool_name = POOL_NAMES
                .iter()
                .find(|(addr, _)| addr.to_lowercase() == pool_address)
                .map(|(_, name)| name.to_string())
                .unwrap_or_else(|| pool_address.clone());

            final_medians.push(PoolMedianLVR {
                pool_name,
                pool_address,
                median_lvr_cents: overall_median,
            });
        }
    }

    // Sort by median LVR descending
    final_medians.sort_by(|a, b| b.median_lvr_cents.cmp(&a.median_lvr_cents));

    info!(
        "Returning median LVRs for {} pools with markout time {}",
        final_medians.len(),
        markout_time
    );

    if final_medians.is_empty() {
        warn!("No median LVR data found for markout time {}", markout_time);
    }

    Ok(Json(MedianLVRResponse { 
        medians: final_medians 
    }))
}


fn get_uint64_column<'a>(batch: &'a RecordBatch, name: &str) -> Result<&'a UInt64Array, StatusCode> {
    batch
        .column(batch.schema().index_of(name).map_err(|e| {
            error!("Failed to get {} column index: {}", name, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
        .as_any()
        .downcast_ref::<UInt64Array>()
        .ok_or_else(|| {
            error!("Failed to cast {} column to UInt64Array", name);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

pub async fn get_max_lvr(
    State(state): State<Arc<AppState>>,
    Query(params): Query<MaxLVRQuery>,
) -> Result<Json<MaxLVRResponse>, StatusCode> {
    let pool_address = params.pool_address.to_lowercase();
    let markout_time = params.markout_time;
    
    info!(
        "Received max LVR request - Pool: {}, Markout Time: {}", 
        pool_address, markout_time
    );

    // Validate pool address
    let valid_pools = get_valid_pools();
    if !valid_pools.contains(&pool_address) {
        warn!("Invalid pool address requested: {}", pool_address);
        return Err(StatusCode::BAD_REQUEST);
    }

    // If markout time is brontes, we need special handling
    if markout_time == "brontes" {
        return handle_brontes_max_lvr(&state, &pool_address).await;
    }

    // Regular non-brontes handling
    let checkpoint_pattern = format!("{}_{}.parquet", pool_address, markout_time);
    debug!("Looking for checkpoint file matching pattern: {}", checkpoint_pattern);
    
    let max_lvr_data = get_checkpoint_max_lvr(&state, &checkpoint_pattern).await?;

    match max_lvr_data {
        Some((block_number, lvr_cents)) => {
            let pool_name = get_pool_name(&pool_address);
            
            info!(
                "Returning max LVR for {} ({}) - Block: {}, Value: {} cents (${:.2})",
                pool_name,
                pool_address,
                block_number,
                lvr_cents,
                lvr_cents as f64 / 100.0
            );

            Ok(Json(MaxLVRResponse {
                block_number,
                lvr_cents,
                pool_name,
            }))
        }
        None => {
            warn!(
                "No max LVR data found for pool {} with markout time {}",
                pool_address,
                markout_time
            );
            Err(StatusCode::NOT_FOUND)
        }
    }
}

async fn handle_brontes_max_lvr(
    state: &Arc<AppState>,
    pool_address: &str,
) -> Result<Json<MaxLVRResponse>, StatusCode> {
    // First get all theoretical maximums
    let theoretical_maxes = get_theoretical_maximums(state, pool_address).await?;
    if theoretical_maxes.is_empty() {
        warn!("No theoretical maximums found for pool {}", pool_address);
        return Err(StatusCode::NOT_FOUND);
    }

    // Get the minimum theoretical maximum
    let min_theoretical_max = theoretical_maxes.values().min().unwrap();
    debug!(
        "Minimum theoretical maximum for pool {}: {} cents (${:.2})",
        pool_address,
        min_theoretical_max,
        *min_theoretical_max as f64 / 100.0
    );

    // Get brontes maximum from checkpoint
    let checkpoint_pattern = format!("{}_{}.parquet", pool_address, "brontes");
    let brontes_max = get_checkpoint_max_lvr(state, &checkpoint_pattern).await?;

    match brontes_max {
        Some((block, value)) if value <= *min_theoretical_max => {
            // Brontes value is valid, return it
            return Ok(Json(MaxLVRResponse {
                block_number: block,
                lvr_cents: value,
                pool_name: get_pool_name(pool_address),
            }));
        }
        _ => {
            // Need to search through interval files
            debug!("Searching intervals for valid maximum LVR");
            return find_valid_max_from_intervals(state, pool_address, *min_theoretical_max).await;
        }
    }
}

async fn get_theoretical_maximums(
    state: &Arc<AppState>,
    pool_address: &str,
) -> Result<HashMap<String, u64>, StatusCode> {
    let mut maximums = HashMap::new();
    let checkpoints_path = object_store::path::Path::from("checkpoints");
    let mut checkpoint_files = state.store.list(Some(&checkpoints_path));

    while let Some(meta_result) = checkpoint_files.next().await {
        let meta = meta_result.map_err(|e| {
            error!("Failed to get file metadata: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        
        let file_path = meta.location.to_string();
        // Skip brontes checkpoint
        if file_path.to_lowercase().ends_with("_brontes.parquet") {
            continue;
        }

        // Only process files for our pool
        if !file_path.to_lowercase().contains(&pool_address.to_lowercase()) {
            continue;
        }

        if let Some((_, max_value)) = get_checkpoint_max_lvr(state, &file_path).await? {
            let markout = file_path
                .split('_')
                .last()
                .and_then(|s| s.strip_suffix(".parquet"))
                .unwrap_or("unknown");
            
            maximums.insert(markout.to_string(), max_value);
        }
    }

    Ok(maximums)
}

async fn get_checkpoint_max_lvr(
    state: &Arc<AppState>,
    file_pattern: &str,
) -> Result<Option<(u64, u64)>, StatusCode> {
    let checkpoints_path = object_store::path::Path::from("checkpoints");
    let mut checkpoint_files = state.store.list(Some(&checkpoints_path));
    
    while let Some(meta_result) = checkpoint_files.next().await {
        let meta = meta_result.map_err(|e| {
            error!("Failed to get file metadata: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        
        let file_path = meta.location.to_string();
        if !file_path.to_lowercase().ends_with(&file_pattern.to_lowercase()) {
            continue;
        }

        debug!("Found matching checkpoint file: {}", file_path);

        let bytes = state.store.get(&meta.location)
            .await
            .map_err(|e| {
                error!("Failed to read checkpoint file: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .bytes()
            .await
            .map_err(|e| {
                error!("Failed to get file bytes: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        let record_reader = ParquetRecordBatchReader::try_new(bytes, 1)
            .map_err(|e| {
                error!("Failed to create Parquet reader: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        for batch_result in record_reader {
            let batch = batch_result.map_err(|e| {
                error!("Failed to read batch: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            let value = get_column_value::<UInt64Array>(&batch, "max_lvr_value")?;
            let block = get_column_value::<UInt64Array>(&batch, "max_lvr_block")?;

            if value > 0 {
                return Ok(Some((block, value)));
            }
            break;
        }
    }

    Ok(None)
}

async fn find_valid_max_from_intervals(
    state: &Arc<AppState>,
    pool_address: &str,
    max_allowed: u64,
) -> Result<Json<MaxLVRResponse>, StatusCode> {
    let mut max_valid_lvr = 0u64;
    let mut max_valid_block = 0u64;
    
    let intervals_path = object_store::path::Path::from("intervals");
    let mut interval_files = state.store.list(Some(&intervals_path));

    while let Some(meta_result) = interval_files.next().await {
        let meta = meta_result.map_err(|e| {
            error!("Failed to get file metadata: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let bytes = state.store.get(&meta.location)
            .await
            .map_err(|e| {
                error!("Failed to read file: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .bytes()
            .await
            .map_err(|e| {
                error!("Failed to get bytes: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        let record_reader = ParquetRecordBatchReader::try_new(bytes, 1024)
            .map_err(|e| {
                error!("Failed to create Parquet reader: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        for batch_result in record_reader {
            let batch = batch_result.map_err(|e| {
                error!("Failed to read batch: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            let pool_addresses = get_string_column(&batch, "pair_address")?;
            let markout_times = get_string_column(&batch, "markout_time")?;
            let max_lvr_cents = get_uint64_column(&batch, "max_lvr_cents")?;
            let interval_ids = get_uint64_column(&batch, "interval_id")?;

            for i in 0..batch.num_rows() {
                if pool_addresses.value(i).to_lowercase() != pool_address {
                    continue;
                }
                
                if markout_times.value(i) != "brontes" {
                    continue;
                }

                let lvr_value = max_lvr_cents.value(i);
                if lvr_value > max_valid_lvr && lvr_value <= max_allowed {
                    max_valid_lvr = lvr_value;
                    // Calculate block number from interval
                    let file_start = meta.location
                        .to_string()
                        .split("intervals/")
                        .nth(1)
                        .and_then(|name| name.trim_end_matches(".parquet").split('_').next())
                        .and_then(|num| num.parse::<u64>().ok())
                        .unwrap_or(0);
                    
                    max_valid_block = file_start + (interval_ids.value(i) * BLOCKS_PER_INTERVAL);
                }
            }
        }
    }

    if max_valid_lvr > 0 {
        Ok(Json(MaxLVRResponse {
            block_number: max_valid_block,
            lvr_cents: max_valid_lvr,
            pool_name: get_pool_name(pool_address),
        }))
    } else {
        warn!(
            "No valid max LVR found for pool {} below threshold {} cents",
            pool_address,
            max_allowed
        );
        Err(StatusCode::NOT_FOUND)
    }
}

fn get_pool_name(pool_address: &str) -> String {
    POOL_NAMES
        .iter()
        .find(|(addr, _)| addr.to_lowercase() == pool_address)
        .map(|(_, name)| name.to_string())
        .unwrap_or_else(|| pool_address.to_string())
}

// Helper function to get column values with proper type handling
fn get_column_value<A: Array + 'static>(
    batch: &RecordBatch, 
    column_name: &str
) -> Result<u64, StatusCode> {
    let idx = batch.schema().index_of(column_name).map_err(|e| {
        error!("Failed to find {} column: {}", column_name, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let column = batch.column(idx);
    match column.data_type() {
        DataType::UInt64 => {
            column.as_any()
                .downcast_ref::<UInt64Array>()
                .map(|arr| arr.value(0))
                .ok_or_else(|| {
                    error!("Failed to cast {} as UInt64Array", column_name);
                    StatusCode::INTERNAL_SERVER_ERROR
                })
        }
        DataType::Int64 => {
            column.as_any()
                .downcast_ref::<Int64Array>()
                .map(|arr| arr.value(0) as u64)
                .ok_or_else(|| {
                    error!("Failed to cast {} as Int64Array", column_name);
                    StatusCode::INTERNAL_SERVER_ERROR
                })
        }
        _ => {
            error!("Unexpected type for {}: {:?}", column_name, column.data_type());
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}


pub async fn get_lvr_histogram(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HistogramQuery>,
) -> Result<Json<HistogramResponse>, StatusCode> {
    let pool_address = params.pool_address.to_lowercase();
    let markout_time = params.markout_time;
    
    info!(
        "Received histogram request - Pool: {}, Markout Time: {}", 
        pool_address, markout_time
    );

    // Check if pool is valid
    let valid_pools = get_valid_pools();
    if !valid_pools.contains(&pool_address) {
        warn!("Invalid pool address requested: {}", pool_address);
        return Err(StatusCode::BAD_REQUEST);
    }

    // Use lowercase for file pattern
    let checkpoint_pattern = format!("{}_{}.parquet", pool_address, markout_time);
    
    // List checkpoint files
    let checkpoints_path = object_store::path::Path::from("checkpoints");
    let mut checkpoint_files = state.store.list(Some(&checkpoints_path));
    
    while let Some(meta_result) = checkpoint_files.next().await {
        let meta = meta_result.map_err(|e| {
            error!("Failed to get file metadata: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR;
        })?;
        
        let file_path = meta.location.to_string();
        
        // Convert file path to lowercase for comparison
        if !file_path.to_lowercase().ends_with(&checkpoint_pattern) {
            continue;
        }

        let bytes = state.store.get(&meta.location)
            .await
            .map_err(|e| {
                error!("Failed to read checkpoint file: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .bytes()
            .await
            .map_err(|e| {
                error!("Failed to get file bytes: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        let record_reader = ParquetRecordBatchReader::try_new(bytes, 1)
            .map_err(|e| {
                error!("Failed to create Parquet reader: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        for batch_result in record_reader {
            let batch = batch_result.map_err(|e| {
                error!("Failed to read batch: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            // Get all non-zero bucket values
            let bucket_0_10 = get_bucket_value(&batch, "total_bucket_0_10")?;
            let bucket_10_100 = get_bucket_value(&batch, "total_bucket_10_100")?;
            let bucket_100_500 = get_bucket_value(&batch, "total_bucket_100_500")?;
            let bucket_1000_3000 = get_bucket_value(&batch, "total_bucket_1000_3000")?;
            let bucket_3000_10000 = get_bucket_value(&batch, "total_bucket_3000_10000")?;
            let bucket_10000_30000 = get_bucket_value(&batch, "total_bucket_10000_30000")?;
            let bucket_30000_plus = get_bucket_value(&batch, "total_bucket_30000_plus")?;

            // Create bucket objects
            let buckets = vec![
                HistogramBucket {
                    range_start: 0.01, // Start just above 0
                    range_end: Some(10.0),
                    count: bucket_0_10,
                    label: "$0.01-$10".to_string(),
                },
                HistogramBucket {
                    range_start: 10.0,
                    range_end: Some(100.0),
                    count: bucket_10_100,
                    label: "$10-$100".to_string(),
                },
                HistogramBucket {
                    range_start: 100.0,
                    range_end: Some(500.0),
                    count: bucket_100_500,
                    label: "$100-$500".to_string(),
                },
                HistogramBucket {
                    range_start: 1000.0,
                    range_end: Some(3000.0),
                    count: bucket_1000_3000,
                    label: "$1K-$3K".to_string(),
                },
                HistogramBucket {
                    range_start: 3000.0,
                    range_end: Some(10000.0),
                    count: bucket_3000_10000,
                    label: "$3K-$10K".to_string(),
                },
                HistogramBucket {
                    range_start: 10000.0,
                    range_end: Some(30000.0),
                    count: bucket_10000_30000,
                    label: "$10K-$30K".to_string(),
                },
                HistogramBucket {
                    range_start: 30000.0,
                    range_end: None,
                    count: bucket_30000_plus,
                    label: "$30K+".to_string(),
                },
            ];

            // Calculate total non-zero observations
            let total_non_zero_observations: u64 = buckets.iter().map(|b| b.count).sum();

            // Get pool name
            let pool_name = POOL_NAMES
                .iter()
                .find(|(addr, _)| addr.to_lowercase() == pool_address)
                .map(|(_, name)| name.to_string())
                .unwrap_or_else(|| pool_address.clone());

            return Ok(Json(HistogramResponse {
                pool_name,
                pool_address,
                buckets,
                total_observations: total_non_zero_observations, // Only count non-zero values
            }));
        }
    }

    // If we get here, we didn't find the file
    Err(StatusCode::NOT_FOUND)
}

fn get_bucket_value(batch: &arrow::record_batch::RecordBatch, column_name: &str) -> Result<u64, StatusCode> {
    let idx = batch.schema().index_of(column_name).map_err(|e| {
        error!("Failed to find {} column: {}", column_name, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let column = batch.column(idx);
    match column.data_type() {
        DataType::UInt64 => column.as_any().downcast_ref::<UInt64Array>()
            .map(|arr| arr.value(0))
            .ok_or_else(|| {
                error!("Failed to cast {} as UInt64Array", column_name);
                StatusCode::INTERNAL_SERVER_ERROR
            }),
        DataType::Int64 => column.as_any().downcast_ref::<Int64Array>()
            .map(|arr| arr.value(0) as u64)
            .ok_or_else(|| {
                error!("Failed to cast {} as Int64Array", column_name);
                StatusCode::INTERNAL_SERVER_ERROR
            }),
        _ => {
            error!("Unexpected type for {}: {:?}", column_name, column.data_type());
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_non_zero_proportion(
    State(state): State<Arc<AppState>>,
    Query(params): Query<NonZeroProportionQuery>,
) -> Result<Json<NonZeroProportionResponse>, StatusCode> {
    let pool_address = params.pool_address.to_lowercase();
    let markout_time = params.markout_time;
    
    info!(
        "Received non-zero proportion request - Pool: {}, Markout Time: {}", 
        pool_address, markout_time
    );

    // Validate pool address
    let valid_pools = get_valid_pools();
    if !valid_pools.contains(&pool_address) {
        warn!("Invalid pool address requested: {}", pool_address);
        return Err(StatusCode::BAD_REQUEST);
    }

    let checkpoint_pattern = format!("{}_{}.parquet", pool_address, markout_time);
    debug!("Looking for checkpoint file matching pattern: {}", checkpoint_pattern);
    
    let checkpoints_path = object_store::path::Path::from("checkpoints");
    let mut checkpoint_files = state.store.list(Some(&checkpoints_path));
    
    while let Some(meta_result) = checkpoint_files.next().await {
        let meta = meta_result.map_err(|e| {
            error!("Failed to get file metadata: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        
        let file_path = meta.location.to_string();
        if !file_path.to_lowercase().ends_with(&checkpoint_pattern) {
            continue;
        }

        debug!("Found matching checkpoint file: {}", file_path);

        let bytes = state.store.get(&meta.location)
            .await
            .map_err(|e| {
                error!("Failed to read checkpoint file: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .bytes()
            .await
            .map_err(|e| {
                error!("Failed to get file bytes: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        let record_reader = ParquetRecordBatchReader::try_new(bytes, 1)
            .map_err(|e| {
                error!("Failed to create Parquet reader: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        for batch_result in record_reader {
            let batch = batch_result.map_err(|e| {
                error!("Failed to read batch: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            let non_zero_proportions = get_float64_column(&batch, "non_zero_proportion")?;

            if batch.num_rows() > 0 {
                let non_zero_proportion = non_zero_proportions.value(0);
                let pool_name = POOL_NAMES
                    .iter()
                    .find(|(addr, _)| addr.to_lowercase() == pool_address)
                    .map(|(_, name)| name.to_string())
                    .unwrap_or_else(|| pool_address.clone());

                info!(
                    "Found non-zero proportion for {} ({}): {:.2}%",
                    pool_name,
                    pool_address,
                    non_zero_proportion * 100.0
                );

                return Ok(Json(NonZeroProportionResponse {
                    pool_name,
                    pool_address,
                    non_zero_proportion,
                }));
            }
        }
    }

    warn!(
        "No checkpoint data found for pool {} with markout time {}",
        pool_address,
        markout_time
    );
    Err(StatusCode::NOT_FOUND)
}