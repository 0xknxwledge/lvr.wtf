use axum::{
    extract::{State, Query},
    response::Json,
    http::StatusCode,
};
use crate::{api::handlers::common::{get_float64_column, get_string_column, get_valid_pools, get_uint64_column}, 
    AppState, NonZeroProportionQuery, NonZeroProportionResponse};
use tracing::{error, info, warn};
use std::sync::Arc;
use parquet::arrow::arrow_reader::ParquetRecordBatchReader;
use object_store::path::Path;

pub async fn get_non_zero_proportion(
    State(state): State<Arc<AppState>>,
    Query(params): Query<NonZeroProportionQuery>,
) -> Result<Json<NonZeroProportionResponse>, StatusCode> {
    let pool_address = params.pool_address.to_lowercase();
    let markout_time = params.markout_time;
    
    // Early validation of pool address
    let valid_pools = get_valid_pools();
    if !valid_pools.contains(&pool_address) {
        warn!("Invalid pool address requested: {}", pool_address);
        return Err(StatusCode::BAD_REQUEST);
    }

    info!(
        "Fetching activity metrics for pool: {} (markout_time: {})", 
        pool_address, markout_time
    );

    // Read from precomputed file
    let bytes = state.store.get(&Path::from("precomputed/pool_metrics/non_zero.parquet"))
        .await
        .map_err(|e| {
            error!("Failed to read precomputed activity metrics: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .bytes()
        .await
        .map_err(|e| {
            error!("Failed to get bytes from precomputed activity metrics: {}", e);
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
        let non_zero_proportions = get_float64_column(&batch, "non_zero_proportion")?;

        for i in 0..batch.num_rows() {
            if pool_addresses.value(i).to_lowercase() == pool_address && 
               markout_times.value(i) == markout_time {
                
                let pool_name = pool_names.value(i).to_string();
                let non_zero_count = non_zero_blocks.value(i);
                let total_count = total_blocks.value(i);
                let proportion = non_zero_proportions.value(i);

                info!(
                    "Found activity metrics for {}: {:.2}% active blocks ({} out of {})", 
                    pool_name,
                    proportion * 100.0,
                    non_zero_count,
                    total_count
                );

                return Ok(Json(NonZeroProportionResponse {
                    pool_name,
                    pool_address: pool_address.clone(),
                    non_zero_proportion: proportion,
                    total_blocks: total_count,
                    non_zero_blocks: non_zero_count,
                }));
            }
        }
    }

    warn!(
        "No activity metrics found for pool {} with markout time {}",
        pool_address,
        markout_time
    );
    Err(StatusCode::NOT_FOUND)
}