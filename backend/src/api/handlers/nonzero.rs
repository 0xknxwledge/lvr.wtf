use axum::{
    extract::{State, Query},
    response::Json,
    http::StatusCode,
};
use crate::{api::handlers::common::{get_pool_name, get_valid_pools, get_uint64_column}, 
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

            // Get the zero bucket count
            let zero_bucket = get_uint64_column(&batch, "total_bucket_0")?;
            
            // Get counts from all non-zero buckets
            let non_zero_buckets = [
                "total_bucket_0_10",
                "total_bucket_10_100",
                "total_bucket_100_500",
                "total_bucket_500_3000",
                "total_bucket_3000_10000",
                "total_bucket_10000_30000",
                "total_bucket_30000_plus",
            ];

            if batch.num_rows() > 0 {
                let mut non_zero_blocks = 0u64;
                
                // Sum up all non-zero buckets
                for bucket_name in &non_zero_buckets {
                    let bucket = get_uint64_column(&batch, bucket_name)?;
                    non_zero_blocks += bucket.value(0);
                }

                let zero_blocks = zero_bucket.value(0);
                let total_blocks = zero_blocks + non_zero_blocks;
                let non_zero_proportion = if total_blocks > 0 {
                    non_zero_blocks as f64 / total_blocks as f64
                } else {
                    0.0
                };

                let pool_name = get_pool_name(&pool_address);

                info!(
                    "Found stats for {} ({}): {:.2}% non-zero ({} out of {} blocks)",
                    pool_name,
                    pool_address,
                    non_zero_proportion * 100.0,
                    non_zero_blocks,
                    total_blocks
                );

                return Ok(Json(NonZeroProportionResponse {
                    pool_name,
                    pool_address,
                    non_zero_proportion,
                    total_blocks,
                    non_zero_blocks,
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