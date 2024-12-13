use axum::{
    extract::{State, Query},
    response::Json,
    http::StatusCode,
};
use crate::{AppState, 
    PoolTotalsQuery, PoolTotalsResponse, PoolTotal,
    POOL_NAMES, api::handlers::common::{get_valid_pools, get_string_column, get_float64_column}};
use tracing::{error, debug, info, warn};
use futures::StreamExt;
use std::sync::Arc;
use parquet::arrow::arrow_reader::ParquetRecordBatchReader;
use arrow::array::{UInt64Array, Int64Array,Array};
use arrow::datatypes::DataType;

pub async fn get_pool_totals(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PoolTotalsQuery>,
) -> Result<Json<PoolTotalsResponse>, StatusCode> {
    let markout_time = params.markout_time.unwrap_or_else(|| String::from("brontes"));
    
    info!("Fetching pool totals for markout_time: {}", markout_time);
    let valid_pools = get_valid_pools();
    debug!("Number of valid pools: {}", valid_pools.len());
    
    let mut pool_totals = Vec::new();
    let mut files_processed = 0;
    let mut files_matched = 0;
    
    let checkpoints_path = object_store::path::Path::from("checkpoints");
    let mut checkpoint_files = state.store.list(Some(&checkpoints_path));
    
    while let Some(meta_result) = checkpoint_files.next().await {
        files_processed += 1;
        let meta = meta_result.map_err(|e| {
            error!("Failed to get file metadata: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        
        let file_path = meta.location.to_string();
        
        if !file_path.ends_with(&format!("_{}.parquet", markout_time)) {
            continue;
        }
        
        files_matched += 1;
        debug!("Processing checkpoint file: {}", file_path);

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

            // Get the column index and inspect its type
            let running_total_idx = batch.schema().index_of("running_total").map_err(|e| {
                error!("Failed to get running_total column index: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            let running_total = {
                let column = batch.column(running_total_idx);
                match column.data_type() {
                    DataType::Int64 => {
                        column.as_any()
                            .downcast_ref::<Int64Array>()
                            .map(|arr| arr.value(0))
                            .ok_or_else(|| {
                                error!("Failed to cast running_total as Int64Array");
                                StatusCode::INTERNAL_SERVER_ERROR
                            })?
                    },
                    DataType::UInt64 => {
                        column.as_any()
                            .downcast_ref::<UInt64Array>()
                            .map(|arr| arr.value(0) as i64)
                            .ok_or_else(|| {
                                error!("Failed to cast running_total as UInt64Array");
                                StatusCode::INTERNAL_SERVER_ERROR
                            })?
                    },
                    other => {
                        error!("Unexpected type for running_total: {:?}", other);
                        return Err(StatusCode::INTERNAL_SERVER_ERROR);
                    }
                }
            };

            let pair_addresses = get_string_column(&batch, "pair_address")?;
            let non_zero_proportions = get_float64_column(&batch, "non_zero_proportion")?;

            if batch.num_rows() == 0 {
                warn!("Empty batch in file: {}", file_path);
                continue;
            }

            let pair_address = pair_addresses.value(0).to_lowercase();
            if !valid_pools.contains(&pair_address) {
                debug!("Skipping invalid pool address: {}", pair_address);
                continue;
            }

            let non_zero_proportion = non_zero_proportions.value(0);

            // Only include pools with actual activity
            if non_zero_proportion > 0.0 {
                let pool_name = POOL_NAMES
                    .iter()
                    .find(|(addr, _)| addr.to_lowercase() == pair_address)
                    .map(|(_, name)| name.to_string())
                    .unwrap_or_else(|| pair_address.clone());

                debug!(
                    "Adding pool {} with total {} cents (non-zero proportion: {:.2}%)", 
                    pool_name, 
                    running_total,
                    non_zero_proportion * 100.0
                );

                pool_totals.push(PoolTotal {
                    pool_name,
                    pool_address: pair_address,
                    total_lvr_cents: running_total.unsigned_abs(),
                });
            }
        }
    }

    // Sort by total_lvr_cents descending
    pool_totals.sort_by(|a, b| b.total_lvr_cents.cmp(&a.total_lvr_cents));

    info!(
        "Process summary: Processed {} files, matched {} files, found {} active pools with data for markout time {}", 
        files_processed, 
        files_matched, 
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