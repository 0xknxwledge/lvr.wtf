use axum::{
    extract::{State, Query},
    response::Json,
    http::StatusCode,
};
use crate::{api::handlers::common::{get_pool_name, get_valid_pools, get_float64_column}, 
    AppState, NonZeroProportionQuery, NonZeroProportionResponse};
use tracing::{error, debug, info, warn};
use futures::StreamExt;
use std::sync::Arc;
use parquet::arrow::arrow_reader::ParquetRecordBatchReader;
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
                let pool_name = get_pool_name(&pool_address);

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