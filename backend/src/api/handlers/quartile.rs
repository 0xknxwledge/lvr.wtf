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
use object_store::path::Path;

pub async fn get_quartile_plot(
    State(state): State<Arc<AppState>>,
    Query(params): Query<QuartilePlotQuery>,
) -> Result<Json<QuartilePlotResponse>, StatusCode> {
    let markout_time = params.markout_time.unwrap_or_else(|| String::from("brontes"));
    
    info!(
        "Analyzing distribution metrics across pools for markout time: {}", 
        markout_time
    );

    // Read from precomputed file
    let bytes = state.store.get(&Path::from("precomputed/distributions/quartile_plots.parquet"))
        .await
        .map_err(|e| {
            error!("Failed to read precomputed distribution metrics: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .bytes()
        .await
        .map_err(|e| {
            error!("Failed to get bytes from precomputed distribution data: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let reader = ParquetRecordBatchReader::try_new(bytes, 1024)
        .map_err(|e| {
            error!("Failed to create Parquet reader: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut pool_data = Vec::new();
    let mut highest_median = 0u64;
    let mut lowest_median = u64::MAX;
    let mut widest_iqr = 0u64;
    let mut widest_iqr_pool = String::new();

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
            // Early filter by markout time
            if markout_times.value(i) != markout_time {
                continue;
            }

            let median_value = median.value(i);
            let p75_value = percentile_75.value(i);
            let p25_value = percentile_25.value(i);
            
            // Track distribution statistics
            highest_median = highest_median.max(median_value);
            lowest_median = lowest_median.min(median_value);
            
            let iqr = p75_value.saturating_sub(p25_value);
            if iqr > widest_iqr {
                widest_iqr = iqr;
                widest_iqr_pool = pool_names.value(i).to_string();
            }

            pool_data.push(PoolQuartileData {
                pool_name: pool_names.value(i).to_string(),
                pool_address: pool_addresses.value(i).to_string(),
                min_nonzero_cents: min_nonzero.value(i),
                percentile_25_cents: p25_value,
                median_cents: median_value,
                percentile_75_cents: p75_value,
            });
        }
    }

    if pool_data.is_empty() {
        warn!(
            "No distribution data found for markout time {}", 
            markout_time
        );
        return Err(StatusCode::NOT_FOUND);
    }

    // Sort by median value descending for consistent ordering
    pool_data.sort_by(|a, b| b.median_cents.cmp(&a.median_cents));

    info!(
        "Analyzed {} pools for {}. Median range: ${:.2} to ${:.2}. Widest IQR: ${:.2} ({})", 
        pool_data.len(),
        markout_time,
        lowest_median as f64 / 100.0,
        highest_median as f64 / 100.0,
        widest_iqr as f64 / 100.0,
        widest_iqr_pool
    );

    Ok(Json(QuartilePlotResponse {
        markout_time,
        pool_data,
    }))
}