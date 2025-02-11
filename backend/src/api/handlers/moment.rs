use axum::{
    extract::{State, Query},
    response::Json,
    http::StatusCode,
};
use std::sync::Arc;
use parquet::arrow::arrow_reader::ParquetRecordBatchReader;
use tracing::{error, info, warn};
use crate::{
    AppState,
    api::handlers::common::{get_string_column, get_float64_column, get_valid_pools},
    DistributionQuery, DistributionResponse,
};
use object_store::path::Path;

pub async fn get_distribution_metrics(
    State(state): State<Arc<AppState>>,
    Query(params): Query<DistributionQuery>,
) -> Result<Json<DistributionResponse>, StatusCode> {
    let pool_address = params.pool_address.to_lowercase();
    let markout_time = params.markout_time;
    
    // Validate pool address early
    let valid_pools = get_valid_pools();
    if !valid_pools.contains(&pool_address) {
        warn!("Invalid pool address requested: {}", pool_address);
        return Err(StatusCode::BAD_REQUEST);
    }

    info!(
        "Fetching distribution metrics for pool: {} (markout_time: {})", 
        pool_address, markout_time
    );

    // Read from precomputed file
    let bytes = state.store.get(&Path::from("precomputed/distributions/metrics.parquet"))
        .await
        .map_err(|e| {
            error!("Failed to read precomputed distribution metrics: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .bytes()
        .await
        .map_err(|e| {
            error!("Failed to get bytes from precomputed metrics data: {}", e);
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

        let pool_addresses = get_string_column(&batch, "pool_address")
            .map_err(|e| {
                error!("Failed to get pool_address column: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        let pool_names = get_string_column(&batch, "pool_name")
            .map_err(|e| {
                error!("Failed to get pool_name column: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        let markout_times = get_string_column(&batch, "markout_time")
            .map_err(|e| {
                error!("Failed to get markout_time column: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        let means = get_float64_column(&batch, "mean")
            .map_err(|e| {
                error!("Failed to get mean column: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        let std_devs = get_float64_column(&batch, "std_dev")
            .map_err(|e| {
                error!("Failed to get std_dev column: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        let skewness = get_float64_column(&batch, "skewness")
            .map_err(|e| {
                error!("Failed to get skewness column: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        let kurtosis = get_float64_column(&batch, "kurtosis")
            .map_err(|e| {
                error!("Failed to get kurtosis column: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        for i in 0..batch.num_rows() {
            if pool_addresses.value(i).to_lowercase() == pool_address && 
               markout_times.value(i) == markout_time {
                
                info!(
                    "Found distribution metrics for {}: mean={:.4}, std_dev={:.4}, skewness={:.4}, kurtosis={:.4}", 
                    pool_names.value(i),
                    means.value(i),
                    std_devs.value(i),
                    skewness.value(i),
                    kurtosis.value(i)
                );

                return Ok(Json(DistributionResponse {
                    pool_name: pool_names.value(i).to_string(),
                    pool_address: pool_address.clone(),
                    markout_time: markout_time.clone(),
                    mean: means.value(i),
                    std_dev: std_devs.value(i),
                    skewness: skewness.value(i),
                    kurtosis: kurtosis.value(i)
                }));
            }
        }
    }

    warn!(
        "No distribution metrics found for pool {} with markout time {}",
        pool_address,
        markout_time
    );
    Err(StatusCode::NOT_FOUND)
}