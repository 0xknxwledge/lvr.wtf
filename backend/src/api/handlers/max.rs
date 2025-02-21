use axum::{
    extract::{State, Query},
    response::Json,
    http::StatusCode,
};
use crate::{AppState, 
    MaxLVRResponse, MaxLVRQuery, MaxLVRPoolData,
    api::handlers::common::{get_uint64_column, 
    get_string_column}};
use tracing::{error, info, warn};
use std::sync::Arc;
use parquet::arrow::arrow_reader::ParquetRecordBatchReader;
use object_store::ObjectStore;
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