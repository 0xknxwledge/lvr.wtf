use axum::{
    extract::{State, Query},
    response::Json,
    http::StatusCode,
};
use crate::{AppState, 
    TimeRangeQuery, RunningTotal, IntervalAPIData, 
    MERGE_BLOCK, POOL_NAMES, api::handlers::common::{get_uint64_column, get_valid_pools, 
    BLOCKS_PER_INTERVAL, FINAL_INTERVAL_FILE, FINAL_PARTIAL_BLOCKS,
    calculate_block_number, get_string_column}};
use tracing::{error, debug, info, warn};
use futures::StreamExt;
use std::{sync::Arc, collections::HashMap};
use parquet::arrow::arrow_reader::ParquetRecordBatchReader;
use arrow::array::Array;


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
    
    info!("Reading interval data from: {:?}", intervals_path);
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