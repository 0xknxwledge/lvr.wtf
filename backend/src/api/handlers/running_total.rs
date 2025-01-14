use axum::{
    extract::{State, Query},
    response::Json,
    http::StatusCode,
};
use crate::{AppState, 
    TimeRangeQuery, RunningTotal, IntervalAPIData, 
    MERGE_BLOCK, POOL_NAMES, api::handlers::common::{get_uint64_column, get_valid_pools, 
    BLOCKS_PER_INTERVAL, FINAL_INTERVAL_FILE, FINAL_PARTIAL_BLOCKS,
    get_string_column}};
use tracing::{error, debug, info, warn};
use std::{sync::Arc, collections::HashMap};
use parquet::arrow::arrow_reader::ParquetRecordBatchReader;


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

    // Read from precomputed file
    let precomputed_path = object_store::path::Path::from("precomputed/running_totals/totals.parquet");
    
    let bytes = state.store.get(&precomputed_path)
        .await
        .map_err(|e| {
            error!("Failed to read precomputed running totals: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .bytes()
        .await
        .map_err(|e| {
            error!("Failed to get bytes from precomputed running totals: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let reader = ParquetRecordBatchReader::try_new(bytes, 1024)
        .map_err(|e| {
            error!("Failed to create Parquet reader: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut results = Vec::new();

    for batch_result in reader {
        let batch = batch_result.map_err(|e| {
            error!("Failed to read batch: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let block_numbers = get_uint64_column(&batch, "block_number")?;
        let markout_times = get_string_column(&batch, "markout_time")?;
        let pool_addresses = get_string_column(&batch, "pool_address")?;
        let running_totals = get_uint64_column(&batch, "running_total_cents")?;

        for i in 0..batch.num_rows() {
            let block_number = block_numbers.value(i);
            
            // Skip if outside requested range
            if block_number < start_block || block_number > end_block {
                continue;
            }

            let pool_address = pool_addresses.value(i).to_lowercase();
            
            // Filter by pool if specified
            if !is_aggregate {
                if let Some(ref requested_pool) = params.pool {
                    if requested_pool.to_lowercase() != pool_address {
                        continue;
                    }
                }
            }

            // Apply markout time filter if specified
            if let Some(ref filter) = params.markout_time {
                if filter != &markout_times.value(i) {
                    continue;
                }
            }

            results.push(RunningTotal {
                block_number,
                markout: markout_times.value(i).to_string(),
                pool_name: None, // We can add pool name lookup if needed
                pool_address: Some(pool_address),
                running_total_cents: running_totals.value(i),
            });
        }
    }

    // Sort by block number and markout time
    results.sort_by(|a, b| {
        a.block_number
            .cmp(&b.block_number)
            .then_with(|| a.markout.to_lowercase().cmp(&b.markout.to_lowercase()))
            .then(a.pool_name.cmp(&b.pool_name))
    });

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