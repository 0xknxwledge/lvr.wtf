use axum::{
    extract::{State, Query},
    response::Json,
    http::StatusCode,
};
use crate::{AppState, 
    MERGE_BLOCK, POOL_ADDRESSES,
    PercentileBandQuery, PercentileBandResponse, PercentileDataPoint,
    api::handlers::common::{get_uint64_column, get_valid_pools, get_string_column}};
use tracing::{error, info, warn};
use std::sync::Arc;
use parquet::arrow::arrow_reader::ParquetRecordBatchReader;

pub async fn get_percentile_band(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PercentileBandQuery>,
) -> Result<Json<PercentileBandResponse>, StatusCode> {
    let start_block = params.start_block.unwrap_or(*MERGE_BLOCK);
    let end_block = params.end_block.unwrap_or(20_000_000);
    let markout_time = params.markout_time.unwrap_or_else(|| String::from("brontes"));
    
    info!(
        "Fetching precomputed percentile bands - Blocks {} to {}, Markout Time: {}", 
        start_block, end_block, markout_time
    );
    
    // Validate pool address if provided
    let pool_filter = if let Some(pool_address) = params.pool_address {
        let pool_address = pool_address.to_lowercase();
        if !get_valid_pools().contains(&pool_address) {
            warn!("Invalid pool address requested: {}", pool_address);
            return Err(StatusCode::BAD_REQUEST);
        }
        Some(pool_address)
    } else {
        // Default to first valid pool if none specified
        Some(POOL_ADDRESSES[0].to_lowercase())
    };

    // Read from precomputed file
    let precomputed_path = object_store::path::Path::from("precomputed/distributions/percentile_bands.parquet");
    
    let bytes = state.store.get(&precomputed_path)
        .await
        .map_err(|e| {
            error!("Failed to read precomputed percentile bands: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .bytes()
        .await
        .map_err(|e| {
            error!("Failed to get bytes from precomputed percentile bands: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let reader = ParquetRecordBatchReader::try_new(bytes, 1024)
        .map_err(|e| {
            error!("Failed to create Parquet reader: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut data_points = Vec::new();
    let mut pool_name = String::new();
    let mut pool_address = String::new();

    for batch_result in reader {
        let batch = batch_result.map_err(|e| {
            error!("Failed to read batch: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let pool_addresses = get_string_column(&batch, "pool_address")?;
        let pool_names = get_string_column(&batch, "pool_name")?;
        let markout_times = get_string_column(&batch, "markout_time")?;
        let block_numbers = get_uint64_column(&batch, "block_number")?;
        let percentile_25 = get_uint64_column(&batch, "percentile_25_cents")?;
        let median = get_uint64_column(&batch, "median_cents")?;
        let percentile_75 = get_uint64_column(&batch, "percentile_75_cents")?;

        for i in 0..batch.num_rows() {
            // Apply filters
            let current_pool = pool_addresses.value(i).to_lowercase();
            if pool_filter.as_ref().map_or(true, |p| &current_pool != p) {
                continue;
            }

            if markout_times.value(i) != markout_time {
                continue;
            }

            let block_number = block_numbers.value(i);
            if block_number < start_block || block_number > end_block {
                continue;
            }

            // Store pool info on first match
            if pool_name.is_empty() {
                pool_name = pool_names.value(i).to_string();
                pool_address = current_pool.clone();
            }

            data_points.push(PercentileDataPoint {
                block_number,
                percentile_25_cents: percentile_25.value(i),
                median_cents: median.value(i),
                percentile_75_cents: percentile_75.value(i),
            });
        }
    }

    if data_points.is_empty() {
        warn!(
            "No percentile band data found for pool {} with markout time {}",
            pool_filter.as_ref().unwrap_or(&"any".to_string()),
            markout_time
        );
        return Err(StatusCode::NOT_FOUND);
    }

    // Sort by block number
    data_points.sort_by_key(|point| point.block_number);

    info!(
        "Returning {} percentile band data points for pool {} with markout time {}",
        data_points.len(),
        pool_name,
        markout_time
    );

    Ok(Json(PercentileBandResponse {
        pool_name,
        pool_address,
        markout_time,
        data_points,
    }))
}