use axum::{
    extract::{State, Query},
    response::Json,
    http::StatusCode,
};
use crate::{AppState, 
    MaxLVRResponse, MaxLVRQuery, MaxLVRPoolData,
    api::handlers::common::{get_uint64_column, 
    BLOCKS_PER_INTERVAL, get_string_column, get_column_value}};
use tracing::{error, debug, info, warn};
use futures::StreamExt;
use std::{sync::Arc, collections::HashMap};
use parquet::arrow::arrow_reader::ParquetRecordBatchReader;
use arrow::array::UInt64Array;
use object_store::ObjectStore;
use anyhow::Context;
use object_store::path::Path;

pub async fn get_max_lvr(
    State(state): State<Arc<AppState>>,
    Query(params): Query<MaxLVRQuery>,
) -> Result<Json<MaxLVRResponse>, StatusCode> {
    let markout_time = params.markout_time;
    
    info!("Fetching maximum LVR values for markout_time: {}", markout_time);

    // Read from precomputed file
    let bytes = state.store.get(&Path::from("precomputed/pool_metrics/max_lvr.parquet"))
        .await
        .map_err(|e| {
            error!("Failed to read precomputed max LVR data: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .bytes()
        .await
        .map_err(|e| {
            error!("Failed to get bytes from precomputed max LVR data: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let reader = ParquetRecordBatchReader::try_new(bytes, 1024)
        .map_err(|e| {
            error!("Failed to create Parquet reader: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut pool_data = Vec::new();
    let mut highest_lvr = 0u64;
    let mut earliest_max = u64::MAX;
    let mut latest_max = 0u64;

    for batch_result in reader {
        let batch = batch_result.map_err(|e| {
            error!("Failed to read batch: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let pool_addresses = get_string_column(&batch, "pool_address")?;
        let pool_names = get_string_column(&batch, "pool_name")?;
        let markout_times = get_string_column(&batch, "markout_time")?;
        let block_numbers = get_uint64_column(&batch, "block_number")?;
        let max_lvr_cents = get_uint64_column(&batch, "max_lvr_cents")?;

        for i in 0..batch.num_rows() {
            // Skip non-matching markout times early
            if markout_times.value(i) != markout_time {
                continue;
            }

            let lvr = max_lvr_cents.value(i);
            let block = block_numbers.value(i);

            // Track statistics
            highest_lvr = highest_lvr.max(lvr);
            if lvr > 0 {
                earliest_max = earliest_max.min(block);
                latest_max = latest_max.max(block);
            }

            pool_data.push(MaxLVRPoolData {
                pool_address: pool_addresses.value(i).to_string(),
                pool_name: pool_names.value(i).to_string(),
                block_number: block,
                lvr_cents: lvr,
            });
        }
    }

    // Sort by LVR value descending for consistent ordering
    pool_data.sort_by(|a, b| b.lvr_cents.cmp(&a.lvr_cents));

    if pool_data.is_empty() {
        warn!(
            "No max LVR data found for markout_time: {}. This might indicate missing data.", 
            markout_time
        );
    } else {
        info!(
            "Retrieved max LVR data for {} pools. Highest value: ${:.2} (Block range: {} to {})", 
            pool_data.len(),
            highest_lvr as f64 / 100.0,
            if earliest_max == u64::MAX { 0 } else { earliest_max },
            latest_max
        );
    }

    Ok(Json(MaxLVRResponse { pools: pool_data }))
}

pub async fn find_valid_max_from_intervals(
    store: &Arc<dyn ObjectStore>,
    pool_address: &str,
    max_allowed: u64,
) -> anyhow::Result<Option<(u64, u64)>> {
    let mut max_valid_lvr = 0u64;
    let mut max_valid_block = 0u64;
    
    let intervals_path = object_store::path::Path::from("intervals");
    let mut interval_files = store.list(Some(&intervals_path));

    while let Some(meta_result) = interval_files.next().await {
        let meta = meta_result.context("Failed to get file metadata")?;

        let bytes = store.get(&meta.location)
            .await
            .context("Failed to read file")?
            .bytes()
            .await
            .context("Failed to get bytes")?;

        let record_reader = ParquetRecordBatchReader::try_new(bytes, 1024)
            .context("Failed to create Parquet reader")?;

        for batch_result in record_reader {
            let batch = batch_result.context("Failed to read batch")?;

            let pool_addresses = get_string_column(&batch, "pair_address")
                .map_err(|e| anyhow::anyhow!("Failed to get pair_address column: {}", e))?;
            let markout_times = get_string_column(&batch, "markout_time")
                .map_err(|e| anyhow::anyhow!("Failed to get markout_time column: {}", e))?;
            let max_lvr_cents = get_uint64_column(&batch, "max_lvr_cents")
                .map_err(|e| anyhow::anyhow!("Failed to get max_lvr_cents column: {}", e))?;
            let interval_ids = get_uint64_column(&batch, "interval_id")
                .map_err(|e| anyhow::anyhow!("Failed to get interval_id column: {}", e))?;

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

pub async fn get_theoretical_maximums(
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

pub async fn get_checkpoint_max_lvr(
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