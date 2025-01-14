use axum::{
    extract::{State, Query},
    response::Json,
    http::StatusCode,
};
use crate::{
    AppState,
    api::handlers::common::{get_uint64_column, get_string_column},
    PoolQuartileData, QuartilePlotResponse, QuartilePlotQuery
};
use tracing::{error, info, warn};
use std::sync::Arc;
use parquet::arrow::arrow_reader::ParquetRecordBatchReader;

pub async fn get_quartile_plot(
    State(state): State<Arc<AppState>>,
    Query(params): Query<QuartilePlotQuery>,
) -> Result<Json<QuartilePlotResponse>, StatusCode> {
    let markout_time = params.markout_time.unwrap_or_else(|| String::from("brontes"));
    
    info!(
        "Fetching precomputed quartile plot data - Markout Time: {}", 
        markout_time
    );

    // Read from precomputed file
    let precomputed_path = object_store::path::Path::from("precomputed/distributions/quartile_plots.parquet");
    
    let bytes = state.store.get(&precomputed_path)
        .await
        .map_err(|e| {
            error!("Failed to read precomputed quartile plot data: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .bytes()
        .await
        .map_err(|e| {
            error!("Failed to get bytes from precomputed quartile plot data: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let reader = ParquetRecordBatchReader::try_new(bytes, 1024)
        .map_err(|e| {
            error!("Failed to create Parquet reader: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut pool_data = Vec::new();

    for batch_result in reader {
        let batch = batch_result.map_err(|e| {
            error!("Failed to read batch: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let pool_addresses = get_string_column(&batch, "pool_address")?;
        let pool_names = get_string_column(&batch, "pool_name")?;
        let markout_times = get_string_column(&batch, "markout_time")?;
        let min_nonzero = get_uint64_column(&batch, "min_nonzero_cents")?;
        let percentile_25 = get_uint64_column(&batch, "percentile_25_cents")?;
        let median = get_uint64_column(&batch, "median_cents")?;
        let percentile_75 = get_uint64_column(&batch, "percentile_75_cents")?;

        for i in 0..batch.num_rows() {
            // Filter by markout time
            if markout_times.value(i) != markout_time {
                continue;
            }

            pool_data.push(PoolQuartileData {
                pool_name: pool_names.value(i).to_string(),
                pool_address: pool_addresses.value(i).to_string(),
                min_nonzero_cents: min_nonzero.value(i),
                percentile_25_cents: percentile_25.value(i),
                median_cents: median.value(i),
                percentile_75_cents: percentile_75.value(i),
            });
        }
    }

    if pool_data.is_empty() {
        warn!(
            "No quartile plot data found for markout time {}", 
            markout_time
        );
        return Err(StatusCode::NOT_FOUND);
    }

    // Sort by median value descending
    pool_data.sort_by(|a, b| b.median_cents.cmp(&a.median_cents));

    info!(
        "Returning quartile plot data for {} pools with markout time {}", 
        pool_data.len(),
        markout_time
    );

    Ok(Json(QuartilePlotResponse {
        markout_time,
        pool_data,
    }))
}