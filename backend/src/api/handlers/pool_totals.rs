use axum::{
    extract::{State, Query},
    response::Json,
    http::StatusCode,
};
use crate::{AppState, 
    PoolTotalsQuery, PoolTotalsResponse, PoolTotal,
    api::handlers::common::{get_uint64_column, get_string_column}};
use tracing::{error, info, warn};
use std::sync::Arc;
use parquet::arrow::arrow_reader::ParquetRecordBatchReader;

pub async fn get_pool_totals(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PoolTotalsQuery>,
) -> Result<Json<PoolTotalsResponse>, StatusCode> {
    let markout_time = params.markout_time.unwrap_or_else(|| String::from("brontes"));
    
    info!("Fetching precomputed pool totals for markout_time: {}", markout_time);

    // Read from precomputed file
    let precomputed_path = object_store::path::Path::from("precomputed/pool_metrics/totals.parquet");
    
    let bytes = state.store.get(&precomputed_path)
        .await
        .map_err(|e| {
            error!("Failed to read precomputed pool totals: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .bytes()
        .await
        .map_err(|e| {
            error!("Failed to get bytes from precomputed pool totals: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let reader = ParquetRecordBatchReader::try_new(bytes, 1024)
        .map_err(|e| {
            error!("Failed to create Parquet reader: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut pool_totals = Vec::new();

    for batch_result in reader {
        let batch = batch_result.map_err(|e| {
            error!("Failed to read batch: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let pool_addresses = get_string_column(&batch, "pool_address")?;
        let pool_names = get_string_column(&batch, "pool_name")?;
        let markout_times = get_string_column(&batch, "markout_time")?;
        let total_lvr_cents = get_uint64_column(&batch, "total_lvr_cents")?;
        let non_zero_blocks = get_uint64_column(&batch, "non_zero_blocks")?;

        for i in 0..batch.num_rows() {
            // Filter by markout time
            if markout_times.value(i) != markout_time {
                continue;
            }

            // Only include pools with activity
            if non_zero_blocks.value(i) > 0 {
                pool_totals.push(PoolTotal {
                    pool_name: pool_names.value(i).to_string(),
                    pool_address: pool_addresses.value(i).to_string(),
                    total_lvr_cents: total_lvr_cents.value(i),
                });
            }
        }
    }

    // Sort by total_lvr_cents descending
    pool_totals.sort_by(|a, b| b.total_lvr_cents.cmp(&a.total_lvr_cents));

    info!(
        "Found {} active pools with data for markout time {}", 
        pool_totals.len(),
        markout_time
    );

    if pool_totals.is_empty() {
        warn!(
            "No pool totals found for markout_time: {}. This might indicate missing data or no activity.", 
            markout_time
        );
    }

    Ok(Json(PoolTotalsResponse { totals: pool_totals }))
}