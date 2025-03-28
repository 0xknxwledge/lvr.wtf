use axum::{
    extract::{State, Query},
    response::Json,
    http::StatusCode,
};
use crate::{AppState, 
    MERGE_BLOCK, POOL_ADDRESSES,
    PercentileBandQuery, PercentileBandResponse, PercentileDataPoint,
    api::handlers::common::{get_uint64_column, get_valid_pools, get_string_column, get_float64_column}};
use tracing::{error, info, warn};
use std::sync::Arc;
use parquet::arrow::arrow_reader::ParquetRecordBatchReader;
use object_store::path::Path;

pub async fn get_percentile_band(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PercentileBandQuery>,
) -> Result<Json<PercentileBandResponse>, StatusCode> {
    let start_block = params.start_block.unwrap_or(*MERGE_BLOCK - 1);
    let end_block = params.end_block.unwrap_or(20_000_000);
    let markout_time = params.markout_time.unwrap_or_else(|| String::from("brontes"));

    // Determine pool to analyze
    let pool_filter = if let Some(pool_address) = params.pool_address.clone() {
        let pool_address = pool_address.to_lowercase();
        if !get_valid_pools().contains(&pool_address) {
            warn!("Invalid pool address provided: {}", pool_address);
            return Err(StatusCode::BAD_REQUEST);
        }
        pool_address
    } else {
        POOL_ADDRESSES[0].to_lowercase()
    };

    info!(
        "Analyzing percentile distribution for pool {} (Blocks {} to {}, Markout: {})", 
        pool_filter, start_block, end_block, markout_time
    );

    let bytes = state.store.get(&Path::from("precomputed/distributions/percentile_bands.parquet"))
        .await
        .map_err(|e| {
            error!("Failed to read precomputed percentile distribution data: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .bytes()
        .await
        .map_err(|e| {
            error!("Failed to get bytes from precomputed percentile data: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let reader = ParquetRecordBatchReader::try_new(bytes, 1024)
        .map_err(|e| {
            error!("Failed to create Parquet reader: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut data_points = Vec::new();
    let mut pool_name = String::new();
    let mut max_median = 0f64;
    let mut min_median = f64::MAX;

    for batch_result in reader {
        let batch = batch_result.map_err(|e| {
            error!("Failed to read batch: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let pool_addresses = get_string_column(&batch, "pool_address")?;
        let pool_names = get_string_column(&batch, "pool_name")?;
        let markout_times = get_string_column(&batch, "markout_time")?;
        let start_blocks = get_uint64_column(&batch, "start_block")?;
        let end_blocks = get_uint64_column(&batch, "end_block")?;
        let total_lvr = get_float64_column(&batch, "total_lvr_dollars")?;
        let percentile_25 = get_float64_column(&batch, "percentile_25_dollars")?;
        let median = get_float64_column(&batch, "median_dollars")?;
        let percentile_75 = get_float64_column(&batch, "percentile_75_dollars")?;

        for i in 0..batch.num_rows() {
            let current_pool = pool_addresses.value(i).to_lowercase();
            let interval_start = start_blocks.value(i);
            let interval_end = end_blocks.value(i);
            
            // Skip if interval is entirely outside requested range
            if interval_end < start_block || interval_start > end_block {
                continue;
            }

            if current_pool != pool_filter || markout_times.value(i) != markout_time {
                continue;
            }

            if pool_name.is_empty() {
                pool_name = pool_names.value(i).to_string();
            }

            let median_value = median.value(i);
            max_median = max_median.max(median_value);
            min_median = min_median.min(median_value);

            data_points.push(PercentileDataPoint {
                start_block: interval_start,
                end_block: interval_end,
                total_lvr_dollars: total_lvr.value(i),
                percentile_25_dollars: percentile_25.value(i),
                median_dollars: median_value,
                percentile_75_dollars: percentile_75.value(i),
            });
        }
    }

    if data_points.is_empty() {
        warn!(
            "No percentile distribution data found for pool {} with markout time {}",
            pool_filter,
            markout_time
        );
        return Err(StatusCode::NOT_FOUND);
    }

    // Sort chronologically by start block
    data_points.sort_by_key(|point| point.start_block);

    info!(
        "Retrieved {} distribution points for {}. Median range: ${:.2} to ${:.2}",
        data_points.len(),
        pool_name,
        min_median,
        max_median
    );

    Ok(Json(PercentileBandResponse {
        pool_name,
        pool_address: pool_filter,
        markout_time,
        data_points,
    }))
}