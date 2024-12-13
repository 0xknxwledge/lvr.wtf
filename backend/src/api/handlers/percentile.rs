use axum::{
    extract::{State, Query},
    response::Json,
    http::StatusCode,
};
use crate::{AppState, 
    MERGE_BLOCK, POOL_ADDRESSES,
    PercentileBandQuery, PercentileBandResponse, PercentileDataPoint,
    api::handlers::common::{get_uint64_column, get_valid_pools,get_string_column, get_pool_name, calculate_block_number}};
use tracing::{error, debug, info, warn};
use futures::StreamExt;
use std::sync::Arc;
use parquet::arrow::arrow_reader::ParquetRecordBatchReader;


pub async fn get_percentile_band(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PercentileBandQuery>,
) -> Result<Json<PercentileBandResponse>, StatusCode> {
    let start_block = params.start_block.unwrap_or(*MERGE_BLOCK);
    let end_block = params.end_block.unwrap_or(20_000_000);
    let markout_time = params.markout_time.unwrap_or_else(|| String::from("brontes"));
    
    info!(
        "Fetching percentile band data for blocks {} to {}, markout_time: {}", 
        start_block, end_block, markout_time
    );
    
    // Validate pool address if provided
    let pool_filter = if let Some(pool_address) = params.pool_address {
        let pool_address = pool_address.to_lowercase();
        if !get_valid_pools().contains(&pool_address) {
            warn!("Invalid pool address requested: {}", pool_address);
            return Err(StatusCode::BAD_REQUEST);
        }
        Some(pool_address)
    } else {
        None
    };

    let mut data_points = Vec::new();

    
    let intervals_path = object_store::path::Path::from("intervals");
    let mut interval_files = state.store.list(Some(&intervals_path));

    while let Some(meta_result) = interval_files.next().await {
        let meta = meta_result.map_err(|e| {
            error!("Failed to get file metadata: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let file_path = meta.location.to_string();


        // Skip files outside our range
        if let Some(file_name) = file_path.split('/').last() {
            let parts: Vec<&str> = file_name.split('_').collect();
            if parts.len() == 2 {
                if let (Ok(file_start), Ok(file_end)) = (
                    parts[0].parse::<u64>(),
                    parts[1].trim_end_matches(".parquet").parse::<u64>()
                ) {
                    if file_start > end_block || file_end < start_block {
                        continue;
                    }
                }
            }
        }

        debug!("Processing file: {}", file_path);

        let bytes = state.store.get(&meta.location)
            .await
            .map_err(|e| {
                error!("Failed to read file content: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .bytes()
            .await
            .map_err(|e| {
                error!("Failed to get file bytes: {}", e);
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

            // Get all required columns
            let interval_ids = get_uint64_column(&batch, "interval_id")?;
            let markout_times = get_string_column(&batch, "markout_time")?;
            let pool_addresses = get_string_column(&batch, "pair_address")?;
            let median_lvrs = get_uint64_column(&batch, "median_lvr_cents")?;
            let p25_lvrs = get_uint64_column(&batch, "percentile_25_cents")?;
            let p75_lvrs = get_uint64_column(&batch, "percentile_75_cents")?;

            for i in 0..batch.num_rows() {
                // Apply filters
                if markout_times.value(i) != markout_time {
                    continue;
                }

                let pool_address = pool_addresses.value(i).to_lowercase();
                if let Some(ref filter_address) = pool_filter {
                    if &pool_address != filter_address {
                        continue;
                    }
                } else if !get_valid_pools().contains(&pool_address) {
                    continue;
                }

                // Calculate block number for this interval
                let block_number = calculate_block_number(
                    start_block,
                    interval_ids.value(i),
                    &file_path
                );

                if block_number >= start_block && block_number <= end_block {
                    data_points.push(PercentileDataPoint {
                        block_number,
                        percentile_25_cents: p25_lvrs.value(i),
                        median_cents: median_lvrs.value(i),
                        percentile_75_cents: p75_lvrs.value(i),
                    });
                }
            }
        }
    }

    // Sort data points by block number
    data_points.sort_by_key(|point| point.block_number);

    // Get pool information for response
    let pool_address = pool_filter.unwrap_or_else(|| {
        // If no pool was specified, use the first one we found
        data_points.first()
            .map(|_| POOL_ADDRESSES[0].to_lowercase())
            .unwrap_or_default()
    });

    let pool_name = get_pool_name(&pool_address);
    info!(
        "Returning {} data points for pool {} with markout time {}",
        data_points.len(),
        pool_name,
        markout_time
    );

    Ok(Json(PercentileBandResponse {
        pool_name,
        pool_address,
        markout_time,
        data_points,
    }))
}