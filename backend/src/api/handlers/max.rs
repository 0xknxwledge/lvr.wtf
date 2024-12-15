use axum::{
    extract::{State, Query},
    response::Json,
    http::StatusCode,
};
use crate::{AppState, 
    MaxLVRResponse, MaxLVRQuery, MaxLVRPoolData,
    api::handlers::common::{get_uint64_column, get_valid_pools, 
    BLOCKS_PER_INTERVAL, get_string_column, get_pool_name, get_column_value}};
use tracing::{error, debug, info};
use futures::StreamExt;
use std::{sync::Arc, collections::HashMap};
use parquet::arrow::arrow_reader::ParquetRecordBatchReader;
use arrow::array::UInt64Array;

pub async fn get_max_lvr(
    State(state): State<Arc<AppState>>,
    Query(params): Query<MaxLVRQuery>,
) -> Result<Json<MaxLVRResponse>, StatusCode> {
    let markout_time = params.markout_time;
    
    info!("Received max LVR request - Markout Time: {}", markout_time);

    // If markout time is brontes, we need special handling
    if markout_time == "brontes" {
        return handle_brontes_max_lvr(&state).await;
    }

    // Regular non-brontes handling
    let valid_pools = get_valid_pools();
    let mut pool_data = Vec::new();

    for pool_address in valid_pools {
        let checkpoint_pattern = format!("{}_{}.parquet", pool_address, markout_time);
        debug!("Looking for checkpoint file matching pattern: {}", checkpoint_pattern);
        
        if let Some((block_number, lvr_cents)) = get_checkpoint_max_lvr(&state, &checkpoint_pattern).await? {
            let pool_name = get_pool_name(&pool_address);
            
            debug!(
                "Found max LVR for {} ({}) - Block: {}, Value: {} cents (${:.2})",
                pool_name,
                pool_address,
                block_number,
                lvr_cents,
                lvr_cents as f64 / 100.0
            );

            pool_data.push(MaxLVRPoolData {
                pool_name,
                pool_address,
                block_number,
                lvr_cents,
            });
        }
    }

    // Sort by lvr_cents descending
    pool_data.sort_by(|a, b| b.lvr_cents.cmp(&a.lvr_cents));

    info!("Returning max LVR data for {} pools", pool_data.len());
    Ok(Json(MaxLVRResponse { pools: pool_data }))
}

async fn handle_brontes_max_lvr(
    state: &Arc<AppState>,
) -> Result<Json<MaxLVRResponse>, StatusCode> {
    let valid_pools = get_valid_pools();
    let mut pool_data = Vec::new();

    for pool_address in valid_pools {
        // First get all theoretical maximums for this pool
        let theoretical_maxes = get_theoretical_maximums(state, &pool_address).await?;
        if theoretical_maxes.is_empty() {
            continue;
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
                // Brontes value is valid, use it
                pool_data.push(MaxLVRPoolData {
                    pool_name: get_pool_name(&pool_address),
                    pool_address: pool_address.clone(),
                    block_number: block,
                    lvr_cents: value,
                });
            }
            _ => {
                // Need to search through interval files
                debug!("Searching intervals for valid maximum LVR");
                if let Some((block, value)) = find_valid_max_from_intervals(state, &pool_address, *min_theoretical_max).await? {
                    pool_data.push(MaxLVRPoolData {
                        pool_name: get_pool_name(&pool_address),
                        pool_address: pool_address.clone(),
                        block_number: block,
                        lvr_cents: value,
                    });
                }
            }
        }
    }

    // Sort by lvr_cents descending
    pool_data.sort_by(|a, b| b.lvr_cents.cmp(&a.lvr_cents));

    info!("Returning max LVR data for {} pools", pool_data.len());
    Ok(Json(MaxLVRResponse { pools: pool_data }))
}

async fn find_valid_max_from_intervals(
    state: &Arc<AppState>,
    pool_address: &str,
    max_allowed: u64,
) -> Result<Option<(u64, u64)>, StatusCode> {
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
        Ok(Some((max_valid_block, max_valid_lvr)))
    } else {
        debug!(
            "No valid max LVR found for pool {} below threshold {} cents",
            pool_address,
            max_allowed
        );
        Ok(None)
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