use axum::{
    extract::{State, Query},
    response::Json,
    http::StatusCode,
};
use crate::{
    AppState,
    api::handlers::common::{get_uint64_column, get_string_column, get_pool_name, calculate_percentile},
    POOL_ADDRESSES, PoolQuartileData, QuartilePlotResponse, QuartilePlotQuery
};
use tracing::{error, info};
use futures::StreamExt;
use std::{sync::Arc, collections::HashMap};
use parquet::arrow::arrow_reader::ParquetRecordBatchReader;

pub async fn get_quartile_plot(
    State(state): State<Arc<AppState>>,
    Query(params): Query<QuartilePlotQuery>,
) -> Result<Json<QuartilePlotResponse>, StatusCode> {
    let markout_time = params.markout_time.unwrap_or_else(|| String::from("brontes"));
    
    info!(
        "Fetching quartile plot data for markout_time: {}", 
        markout_time
    );

    // Map to collect all non-zero LVR values for each pool
    let mut pool_lvr_values: HashMap<String, Vec<u64>> = HashMap::new();
    
    let intervals_path = object_store::path::Path::from("intervals");
    let mut interval_files = state.store.list(Some(&intervals_path));

    while let Some(meta_result) = interval_files.next().await {
        let meta = meta_result.map_err(|e| {
            error!("Failed to get file metadata: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let bytes = state.store.get(&meta.location)
            .await
            .map_err(|e| {
                error!("Failed to read file: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .bytes()
            .await
            .map_err(|e| {
                error!("Failed to get bytes: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        let record_reader = ParquetRecordBatchReader::try_new(bytes, 1024)
            .map_err(|e| {
                error!("Failed to create Parquet reader: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        for batch_result in record_reader {
            let batch = batch_result.map_err(|e| {
                error!("Failed to read batch: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            let markout_times = get_string_column(&batch, "markout_time")?;
            let pool_addresses = get_string_column(&batch, "pair_address")?;
            let total_lvr_cents = get_uint64_column(&batch, "total_lvr_cents")?;
            let non_zero_counts = get_uint64_column(&batch, "non_zero_count")?;

            for i in 0..batch.num_rows() {
                // Skip if not matching markout time or no activity
                if markout_times.value(i) != markout_time || non_zero_counts.value(i) == 0 {
                    continue;
                }

                let pool_address = pool_addresses.value(i).to_lowercase();
                let lvr_cents = total_lvr_cents.value(i);
                
                // Only include non-zero values
                if lvr_cents > 0 {
                    pool_lvr_values
                        .entry(pool_address)
                        .or_default()
                        .push(lvr_cents);
                }
            }
        }
    }

    // Calculate quartiles and minimum for each pool
    let mut pool_data: Vec<PoolQuartileData> = Vec::new();

    for pool_address in POOL_ADDRESSES.iter() {
        let pool_address = pool_address.to_lowercase();
        if let Some(mut values) = pool_lvr_values.remove(&pool_address) {
            // Need enough data points to calculate meaningful quartiles
            if !values.is_empty() {
                // Sort values for percentile calculation
                values.sort_unstable();
                
                pool_data.push(PoolQuartileData {
                    pool_name: get_pool_name(&pool_address),
                    pool_address: pool_address.clone(),
                    min_nonzero_cents: values[0], // First value after sorting is the minimum
                    percentile_25_cents: calculate_percentile(&values, 0.25),
                    median_cents: calculate_percentile(&values, 0.50),
                    percentile_75_cents: calculate_percentile(&values, 0.75),
                });
            }
        }
    }

    // Sort by median value descending for better visualization
    pool_data.sort_by(|a, b| b.median_cents.cmp(&a.median_cents));

    info!(
        "Returning quartile data for {} pools with markout time {}",
        pool_data.len(),
        markout_time
    );

    Ok(Json(QuartilePlotResponse {
        markout_time,
        pool_data,
    }))
}