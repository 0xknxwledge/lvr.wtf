use axum::{
    extract::{State, Query},
    response::Json,
    http::StatusCode,
};
use crate::{AppState, 
    MERGE_BLOCK, POOL_ADDRESSES,
    PercentileBandQuery, PercentileBandResponse, PercentileDataPoint,
    api::handlers::common::{get_uint64_column, get_valid_pools, get_string_column, get_pool_name, calculate_percentile}};
use tracing::{error, debug, info, warn};
use futures::StreamExt;
use std::{sync::Arc, collections::HashMap};
use parquet::arrow::arrow_reader::ParquetRecordBatchReader;

pub async fn get_percentile_band(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PercentileBandQuery>,
) -> Result<Json<PercentileBandResponse>, StatusCode> {
    let start_block = params.start_block.unwrap_or(*MERGE_BLOCK);
    let end_block = params.end_block.unwrap_or(20_000_000);
    let markout_time = params.markout_time.unwrap_or_else(|| String::from("brontes"));
    
    info!(
        "Fetching LVR distribution for blocks {} to {}, markout_time: {}", 
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
        // Default to first valid pool if none specified
        Some(POOL_ADDRESSES[0].to_lowercase())
    };

    // Map to collect all LVR values per interval file
    let mut file_lvr_values: HashMap<u64, Vec<u64>> = HashMap::new();
    let intervals_path = object_store::path::Path::from("intervals");
    let mut interval_files = state.store.list(Some(&intervals_path));

    while let Some(meta_result) = interval_files.next().await {
        let meta = meta_result.map_err(|e| {
            error!("Failed to get file metadata: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let file_path = meta.location.to_string();

        // Extract block range from file name
        let (file_start, file_end) = if let Some(file_name) = file_path.split('/').last() {
            let parts: Vec<&str> = file_name.split('_').collect();
            if parts.len() == 2 {
                let start = parts[0].parse::<u64>().unwrap_or(0);
                let end = parts[1].trim_end_matches(".parquet").parse::<u64>().unwrap_or(0);
                (start, end)
            } else {
                (0, 0)
            }
        } else {
            (0, 0)
        };

        // Skip files outside our range
        if file_start > end_block || file_end < start_block {
            continue;
        }

        debug!("Processing interval file: {}", file_path);

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

        let mut interval_values = Vec::new();

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
                // Apply filters
                if markout_times.value(i) != markout_time {
                    continue;
                }

                let pool_address = pool_addresses.value(i).to_lowercase();
                if pool_filter.as_ref().map_or(true, |filter| &pool_address != filter) {
                    continue;
                }

                // Only include intervals with activity
                if non_zero_counts.value(i) > 0 {
                    interval_values.push(total_lvr_cents.value(i));
                }
            }
        }

        // Store values if we found any matching entries
        if !interval_values.is_empty() {
            file_lvr_values.insert(file_start, interval_values);
        }
    }

    // Calculate percentiles for each interval file
    let mut data_points: Vec<PercentileDataPoint> = file_lvr_values
        .into_iter()
        .map(|(block_number, mut values)| {
            // Sort values for percentile calculation
            values.sort_unstable();
            
            PercentileDataPoint {
                block_number,
                percentile_25_cents: calculate_percentile(&values, 0.25),
                median_cents: calculate_percentile(&values, 0.5),
                percentile_75_cents: calculate_percentile(&values, 0.75),
            }
        })
        .collect();

    // Sort data points by block number
    data_points.sort_by_key(|point| point.block_number);

    // Get pool information for response
    let pool_address = pool_filter.unwrap_or_else(|| POOL_ADDRESSES[0].to_lowercase());
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