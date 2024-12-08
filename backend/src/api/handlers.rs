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
    LVRRatioQuery, LVRRatioResponse, 
    MarkoutRatio, LVRTotals};
use tracing::{error, debug, info, warn};
use futures::StreamExt;
use std::{sync::Arc, collections::{HashSet, HashMap}};
use parquet::arrow::arrow_reader::ParquetRecordBatchReader;
use arrow::array::{StringArray, UInt64Array, Int64Array, Array};


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

// Helper structs and functions
struct BatchColumns<'a> {
    interval_ids: &'a UInt64Array,
    markout_times: &'a StringArray,
    total_lvr_cents: &'a Int64Array,
    pool_addresses: &'a StringArray,
}

fn get_batch_columns(batch: &arrow::record_batch::RecordBatch) -> Result<BatchColumns, StatusCode> {
    let interval_ids = batch
        .column(batch.schema().index_of("interval_id").map_err(|e| {
            error!("Failed to get interval_id column index: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
        .as_any()
        .downcast_ref::<UInt64Array>()
        .ok_or_else(|| StatusCode::INTERNAL_SERVER_ERROR)?;

    let markout_times = batch
        .column(batch.schema().index_of("markout_time").map_err(|e| {
            error!("Failed to get markout_time column index: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total_lvr_cents = batch
        .column(batch.schema().index_of("total_lvr_cents").map_err(|e| {
            error!("Failed to get total_lvr_cents column index: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
        .as_any()
        .downcast_ref::<Int64Array>()
        .ok_or_else(|| StatusCode::INTERNAL_SERVER_ERROR)?;

    let pool_addresses = batch
        .column(batch.schema().index_of("pair_address").map_err(|e| {
            error!("Failed to get pair_address column index: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(BatchColumns {
        interval_ids,
        markout_times,
        total_lvr_cents,
        pool_addresses,
    })
}

pub async fn get_running_total(
    State(state): State<Arc<AppState>>,
    Query(params): Query<TimeRangeQuery>,
) -> Result<Json<Vec<RunningTotal>>, StatusCode> {
    let start_block = params.start_block.unwrap_or(*MERGE_BLOCK);
    let end_block = params.end_block.unwrap_or(20_000_000);
    let is_aggregate = params.aggregate.unwrap_or(false);
    
    debug!("Fetching running total for blocks {} to {}", start_block, end_block);
    
    let valid_pools = get_valid_pools();
    let mut interval_totals: HashMap<(u64, u64, String, Option<String>), IntervalAPIData> = HashMap::new();
    let intervals_path = object_store::path::Path::from("intervals");
    
    info!("Attempting to read from path: {:?}", intervals_path);
    let mut interval_files = state.store.list(Some(&intervals_path));

    while let Some(meta_result) = interval_files.next().await {
        let meta = meta_result.map_err(|e| {
            error!("Failed to get file metadata: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let file_path = meta.location.to_string();
        
        // Skip files outside our range
        if let Some(file_name) = file_path.split('/').last() {
            let parts: Vec<&str> = file_name.split('_').collect();
            if parts.len() == 2 {
                if let (Ok(file_start), Ok(file_end)) = (
                    parts[0].parse::<u64>(),
                    parts[1].trim_end_matches(".parquet").parse::<u64>()
                ) {
                    if file_start > end_block || file_end < start_block {
                        continue;
                    }
                }
            }
        }

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

            let columns = get_batch_columns(&batch)?;
            
            for i in 0..batch.num_rows() {
                if columns.total_lvr_cents.is_null(i) {
                    continue;
                }

                let pool_address = columns.pool_addresses.value(i).to_lowercase();
                
                // Skip if pool is not in our valid set
                if !valid_pools.contains(&pool_address) {
                    continue;
                }

                let interval_id = columns.interval_ids.value(i);
                let markout_time = columns.markout_times.value(i).to_string();
                let lvr_cents = if columns.total_lvr_cents.value(i) < 0 {
                    0u64
                } else {
                    columns.total_lvr_cents.value(i) as u64
                };

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
    
    // Convert the HashMap into a sorted Vec for chronological processing
    let mut sorted_entries: Vec<_> = interval_totals.into_iter().collect();
    sorted_entries.sort_by(|a, b| {
        // Sort by block number (calculated from file_start and interval_id)
        let block_a = a.0.0 + (a.0.1 * BLOCKS_PER_INTERVAL);
        let block_b = b.0.0 + (b.0.1 * BLOCKS_PER_INTERVAL);
        block_a.cmp(&block_b)
    });

    // Process each entry chronologically
    for ((file_start, interval_id, markout, pool_opt), data) in sorted_entries {
        // Calculate actual block number, handling the special case for final interval
        let block_number = if data.file_path.ends_with(FINAL_INTERVAL_FILE) && interval_id == 19 {
            file_start + (interval_id * BLOCKS_PER_INTERVAL) + FINAL_PARTIAL_BLOCKS
        } else {
            file_start + (interval_id * BLOCKS_PER_INTERVAL)
        };

        // Only process if within requested block range
        if block_number >= start_block && block_number <= end_block {
            if is_aggregate {
                // Handle aggregated totals (across all pools)
                let current_total = last_totals
                    .entry(markout.clone())
                    .and_modify(|total| *total = total.saturating_add(data.total))
                    .or_insert(data.total);

                // Only add new entry if total changed or first entry for this markout
                if results.last().map_or(true, |last: &RunningTotal| 
                    last.markout != markout || last.running_total_cents != *current_total
                ) {
                    results.push(RunningTotal {
                        block_number,
                        markout,
                        pool_name: None,
                        running_total_cents: *current_total,
                    });
                }
            } else if let Some(pool_address) = pool_opt {
                // Handle individual pool totals
                let pool_name = POOL_NAMES
                .iter()
                .find(|(addr, _)| addr.to_lowercase() == pool_address)
                .map(|(_, name)| name.to_string())
                .unwrap_or_else(|| pool_address.clone());

                // Create unique key for this pool/markout combination
                let key = format!("{}_{}", markout, pool_name);
                let current_total = last_totals
                    .entry(key.clone())
                    .and_modify(|total| *total = total.saturating_add(data.total))
                    .or_insert(data.total);

                // Only add new entry if total changed or first entry for this pool/markout
                if results.last().map_or(true, |last: &RunningTotal| 
                    last.markout != markout || 
                    last.pool_name.as_ref() != Some(&pool_name) ||
                    last.running_total_cents != *current_total
                ) {
                    results.push(RunningTotal {
                        block_number,
                        markout,
                        pool_name: Some(pool_name),
                        running_total_cents: *current_total,
                    });
                }
            }
        }
    }

    // Sort final results by block number, then markout time (case-insensitive), then pool name
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

            let columns = get_batch_columns(&batch)?;

            for i in 0..batch.num_rows() {
                if !columns.total_lvr_cents.is_null(i) {
                    let pool_address = columns.pool_addresses.value(i).to_lowercase();
                    
                    // Skip if pool is not in our valid set
                    if !valid_pools.contains(&pool_address) {
                        continue;
                    }

                    let markout_time = columns.markout_times.value(i);
                    let lvr_cents = columns.total_lvr_cents.value(i);
                    
                    let unsigned_cents = if lvr_cents < 0 {
                        0
                    } else {
                        lvr_cents as u64
                    };

                    if markout_time == "brontes" {
                        totals.realized = totals.realized.saturating_add(unsigned_cents);
                    } else {
                        totals.theoretical
                            .entry(markout_time.to_string())
                            .and_modify(|e| *e = e.saturating_add(unsigned_cents))
                            .or_insert(unsigned_cents);
                    }
                }
            }
        }
    }

    let ratios = calculate_lvr_ratios(totals);
    
    info!("Returning {} LVR ratios", ratios.len());
    Ok(Json(LVRRatioResponse { ratios }))
}

fn calculate_lvr_ratios(totals: LVRTotals) -> Vec<MarkoutRatio> {
    let mut ratios = Vec::new();
    
    // Only calculate ratios if we have realized LVR data
    if totals.realized > 0 {
        for (markout_time, theoretical_lvr) in totals.theoretical {
            // Only include ratios where we have theoretical data
            if theoretical_lvr > 0 {
                // Calculate the ratio as a percentage
                let ratio = (totals.realized as f64 / theoretical_lvr as f64) * 100.0;
                
                // Create ratio entry
                ratios.push(MarkoutRatio {
                    markout_time,
                    // Cap ratio at 100% to prevent unrealistic values
                    ratio: ratio.min(100.0),
                    realized_lvr_cents: totals.realized,
                    theoretical_lvr_cents: theoretical_lvr,
                });
            }
        }
    }

    // Sort by markout time for consistent ordering
    ratios.sort_by(|a, b| {
        // Handle special case for "brontes" markout time
        if a.markout_time == "brontes" {
            std::cmp::Ordering::Greater
        } else if b.markout_time == "brontes" {
            std::cmp::Ordering::Less
        } else {
            // Try to parse markout times as floats for numerical sorting
            match (
                a.markout_time.parse::<f64>(),
                b.markout_time.parse::<f64>()
            ) {
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
    let markout_time = if params.markout_time.is_none() {
        String::from("brontes")
    } else {
        params.markout_time.unwrap()
    };
    
    info!("Fetching pool totals for markout_time: {}", markout_time);
    let valid_pools = get_valid_pools();
    info!("Number of valid pools: {}", valid_pools.len());
    let mut pool_totals = Vec::new();
    
    // List all checkpoint files
    let checkpoints_path = object_store::path::Path::from("checkpoints");
    info!("Searching for checkpoint files in: {:?}", checkpoints_path);
    let mut checkpoint_files = state.store.list(Some(&checkpoints_path));
    
    let mut files_checked = 0;
    let mut files_matched = 0;
    
    while let Some(meta_result) = checkpoint_files.next().await {
        files_checked += 1;
        let meta = meta_result.map_err(|e| {
            error!("Failed to get file metadata: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        
        let file_path = meta.location.to_string();
        debug!("Checking checkpoint file: {}", file_path);
        
        // More robust filename extraction
        let file_name = file_path.split('/').last().unwrap_or("");
        if file_name.is_empty() {
            warn!("Skipping file with invalid path: {}", file_path);
            continue;
        }

        if !file_name.contains(&format!("_{}", markout_time)) {
            debug!("Skipping file {} as it doesn't match markout_time {}", file_name, markout_time);
            continue;
        }
        
        files_matched += 1;
        debug!("Processing matching file: {}", file_path);
        
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

            // Get column indices with proper error handling
            let pair_address_idx = match batch.schema().index_of("pair_address") {
                Ok(idx) => idx,
                Err(e) => {
                    error!("Failed to find pair_address column: {}", e);
                    continue;
                }
            };

            let running_total_idx = match batch.schema().index_of("running_total") {
                Ok(idx) => idx,
                Err(e) => {
                    error!("Failed to find running_total column: {}", e);
                    continue;
                }
            };

            // Safe column access
            let pair_address_array = match batch.column(pair_address_idx).as_any().downcast_ref::<StringArray>() {
                Some(array) => array,
                None => {
                    error!("Failed to cast pair_address column to StringArray");
                    continue;
                }
            };

            let running_total_array = match batch.column(running_total_idx).as_any().downcast_ref::<Int64Array>() {
                Some(array) => array,
                None => {
                    error!("Failed to cast running_total column to Int64Array");
                    continue;
                }
            };

            // Safe value access
            if pair_address_array.is_empty() || running_total_array.is_empty() {
                warn!("Empty arrays in batch, skipping");
                continue;
            }

            let pair_address = pair_address_array.value(0).to_lowercase();
            debug!("Found pool address in file: {}", pair_address);

            // Skip if pool is not in valid set
            if !valid_pools.contains(&pair_address) {
                debug!("Skipping invalid pool address: {}", pair_address);
                continue;
            }

            let running_total = running_total_array.value(0);

            // Get pool name from our constants
            let pool_name = POOL_NAMES
            .iter()
            .find(|(addr, _)| addr.to_lowercase() == pair_address)
            .map(|(_, name)| name.to_string())
            .unwrap_or_else(|| pair_address.clone());

            debug!("Adding pool {} with total {}", pool_name, running_total);
            pool_totals.push(PoolTotal {
                pool_name,
                pool_address: pair_address,
                total_lvr_cents: running_total as u64,
            });
        }
    }

    // Sort by total_lvr_cents descending
    pool_totals.sort_by(|a, b| b.total_lvr_cents.cmp(&a.total_lvr_cents));

    info!(
        "Process summary: Checked {} files, matched {} files, found {} pools with data. Markout time: {}", 
        files_checked, 
        files_matched, 
        pool_totals.len(),
        markout_time
    );

    if pool_totals.is_empty() {
        warn!(
            "No pool totals found for markout_time: {}. This might indicate missing checkpoint files or filtering issues.", 
            markout_time
        );
    }

    Ok(Json(PoolTotalsResponse { totals: pool_totals }))
}

// Add to handlers.rs

pub async fn get_pool_medians(
    State(state): State<Arc<AppState>>,
    Query(params): Query<MedianLVRQuery>,
) -> Result<Json<MedianLVRResponse>, StatusCode> {
    let markout_time = params.markout_time.unwrap_or_else(|| String::from("brontes"));
    
    info!("Fetching pool medians for markout_time: {}", markout_time);
    let valid_pools = get_valid_pools();
    
    // Track medians for each pool
    let mut pool_medians: HashMap<String, Vec<u64>> = HashMap::new();
    
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

            // Get column indices
            let markout_times = batch
                .column(batch.schema().index_of("markout_time").unwrap())
                .as_any()
                .downcast_ref::<StringArray>()
                .unwrap();

            let pool_addresses = batch
                .column(batch.schema().index_of("pair_address").unwrap())
                .as_any()
                .downcast_ref::<StringArray>()
                .unwrap();

            let median_lvrs = batch
                .column(batch.schema().index_of("median_lvr_cents").unwrap())
                .as_any()
                .downcast_ref::<Int64Array>()
                .unwrap();

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
                if median_lvr > 0 {  // Only include non-zero values
                    pool_medians
                        .entry(pool_address)
                        .or_default()
                        .push(median_lvr as u64);
                }
            }
        }
    }

    // Calculate final medians for each pool
    let mut final_medians = Vec::new();
    for (pool_address, medians) in pool_medians {
        if !medians.is_empty() {
            // Sort medians to find the overall median
            let mut sorted_medians = medians;
            sorted_medians.sort_unstable();
            let overall_median = if sorted_medians.len() % 2 == 0 {
                let mid = sorted_medians.len() / 2;
                (sorted_medians[mid - 1] + sorted_medians[mid]) / 2
            } else {
                sorted_medians[sorted_medians.len() / 2]
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

    Ok(Json(MedianLVRResponse { 
        medians: final_medians 
    }))
}