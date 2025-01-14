use axum::{
    extract::{State, Query},
    response::Json,
    http::StatusCode,
};
use crate::{api::handlers::common::{get_float64_column, get_string_column, get_valid_pools, get_uint64_column}, 
    AppState, NonZeroProportionQuery, NonZeroProportionResponse};
use tracing::{error, debug, info, warn};
use std::sync::Arc;
use parquet::arrow::arrow_reader::ParquetRecordBatchReader;


pub async fn get_non_zero_proportion(
    State(state): State<Arc<AppState>>,
    Query(params): Query<NonZeroProportionQuery>,
) -> Result<Json<NonZeroProportionResponse>, StatusCode> {
    let pool_address = params.pool_address.to_lowercase();
    let markout_time = params.markout_time;
    
    info!(
        "Fetching precomputed non-zero proportion data - Pool: {}, Markout Time: {}", 
        pool_address, markout_time
    );

    // Validate pool address
    let valid_pools = get_valid_pools();
    if !valid_pools.contains(&pool_address) {
        warn!("Invalid pool address requested: {}", pool_address);
        return Err(StatusCode::BAD_REQUEST);
    }

    // Read from precomputed file
    let precomputed_path = object_store::path::Path::from("precomputed/pool_metrics/non_zero.parquet");
    
    let bytes = state.store.get(&precomputed_path)
        .await
        .map_err(|e| {
            error!("Failed to read precomputed non-zero proportion data: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .bytes()
        .await
        .map_err(|e| {
            error!("Failed to get bytes from precomputed non-zero proportion data: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let reader = ParquetRecordBatchReader::try_new(bytes, 1024)
        .map_err(|e| {
            error!("Failed to create Parquet reader: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    for batch_result in reader {
        let batch = batch_result.map_err(|e| {
            error!("Failed to read batch: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let pool_addresses = get_string_column(&batch, "pool_address")?;
        let pool_names = get_string_column(&batch, "pool_name")?;
        let markout_times = get_string_column(&batch, "markout_time")?;
        let non_zero_blocks = get_uint64_column(&batch, "non_zero_blocks")?;
        let total_blocks = get_uint64_column(&batch, "total_blocks")?;
        
        // Already pre-computed in the parquet file
        let non_zero_proportions = get_float64_column(&batch, "non_zero_proportion")?;

        for i in 0..batch.num_rows() {
            if pool_addresses.value(i).to_lowercase() == pool_address && 
               markout_times.value(i) == markout_time {
                
                debug!(
                    "Found non-zero stats for {} ({}): {:.2}% non-zero ({} out of {} blocks)",
                    pool_names.value(i),
                    pool_address,
                    non_zero_proportions.value(i) * 100.0,
                    non_zero_blocks.value(i),
                    total_blocks.value(i)
                );

                return Ok(Json(NonZeroProportionResponse {
                    pool_name: pool_names.value(i).to_string(),
                    pool_address: pool_address.clone(),
                    non_zero_proportion: non_zero_proportions.value(i),
                    total_blocks: total_blocks.value(i),
                    non_zero_blocks: non_zero_blocks.value(i),
                }));
            }
        }
    }

    warn!(
        "No non-zero proportion data found for pool {} with markout time {}",
        pool_address,
        markout_time
    );
    Err(StatusCode::NOT_FOUND)
}