use axum::{
    extract::{State, Query},
    response::Json,
    http::StatusCode,
};
use crate::{
    AppState,
    api::handlers::common::{get_uint64_column, get_string_column, get_valid_pools},
    QuartilePlotResponse, QuartilePlotQuery
};
use tracing::{error, info, warn};
use std::sync::Arc;
use parquet::arrow::arrow_reader::ParquetRecordBatchReader;
use object_store::path::Path;

pub async fn get_quartile_plot(
    State(state): State<Arc<AppState>>,
    Query(params): Query<QuartilePlotQuery>,
) -> Result<Json<QuartilePlotResponse>, StatusCode> {
    let pool_address = params.pool_address.to_lowercase();
    let markout_time = params.markout_time.unwrap_or_else(|| String::from("brontes"));

    // Validate pool address early
    let valid_pools = get_valid_pools();
    if !valid_pools.contains(&pool_address) {
        warn!("Invalid pool address provided: {}", pool_address);
        return Err(StatusCode::BAD_REQUEST);
    }

    info!(
        "Analyzing distribution metrics for pool {} with markout time: {}", 
        pool_address, markout_time
    );

    // Read from precomputed file
    let bytes = state.store.get(&Path::from("precomputed/distributions/quartile_plots.parquet"))
        .await
        .map_err(|e| {
            error!("Failed to read precomputed quartile metrics: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .bytes()
        .await
        .map_err(|e| {
            error!("Failed to get bytes from precomputed quartile data: {}", e);
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
        let percentile_25 = get_uint64_column(&batch, "percentile_25_cents")?;
        let median = get_uint64_column(&batch, "median_cents")?;
        let percentile_75 = get_uint64_column(&batch, "percentile_75_cents")?;

        for i in 0..batch.num_rows() {
            let current_pool = pool_addresses.value(i).to_lowercase();
            
            // Filter by pool and markout time
            if current_pool != pool_address || markout_times.value(i) != markout_time {
                continue;
            }

            info!(
                "Found quartile data for {} ({}): Q1=${:.2}, Median=${:.2}, Q3=${:.2}", 
                pool_names.value(i),
                markout_time,
                percentile_25.value(i) as f64 / 100.0,
                median.value(i) as f64 / 100.0,
                percentile_75.value(i) as f64 / 100.0
            );

            return Ok(Json(QuartilePlotResponse {
                pool_name: pool_names.value(i).to_string(),
                pool_address: current_pool,
                markout_time,
                percentile_25_cents: percentile_25.value(i),
                median_cents: median.value(i),
                percentile_75_cents: percentile_75.value(i),
            }));
        }
    }

    warn!(
        "No quartile data found for pool {} with markout time {}", 
        pool_address, markout_time
    );
    Err(StatusCode::NOT_FOUND)
}