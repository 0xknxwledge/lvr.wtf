use axum::{
    extract::{State, Query},
    response::Json,
    http::StatusCode,
};
use crate::{AppState, 
    HistogramBucket, HistogramResponse, HistogramQuery,
    api::handlers::common::{get_string_column, get_float64_column, get_uint64_column, get_valid_pools}};
use tracing::{error, info, warn};
use std::sync::Arc;
use parquet::arrow::arrow_reader::ParquetRecordBatchReader;
use arrow::array::{Int64Array,UInt64Array, Array};
use arrow::datatypes::DataType;
use object_store::path::Path;

pub async fn get_lvr_histogram(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HistogramQuery>,
) -> Result<Json<HistogramResponse>, StatusCode> {
    let pool_address = params.pool_address.to_lowercase();
    let markout_time = params.markout_time;
    
    // Validate pool address early
    let valid_pools = get_valid_pools();
    if !valid_pools.contains(&pool_address) {
        warn!("Invalid pool address requested: {}", pool_address);
        return Err(StatusCode::BAD_REQUEST);
    }

    info!(
        "Fetching LVR distribution data for pool: {} (markout_time: {})", 
        pool_address, markout_time
    );

    // Read from precomputed file
    let bytes = state.store.get(&Path::from("precomputed/distributions/histograms.parquet"))
        .await
        .map_err(|e| {
            error!("Failed to read precomputed histogram data: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .bytes()
        .await
        .map_err(|e| {
            error!("Failed to get bytes from precomputed histogram data: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let reader = ParquetRecordBatchReader::try_new(bytes, 1024)
        .map_err(|e| {
            error!("Failed to create Parquet reader: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut buckets = Vec::new();
    let mut total_observations = 0u64;
    let mut pool_name = String::new();
    let mut highest_bucket_count = 0u64;
    let mut mode_bucket = String::new();

    for batch_result in reader {
        let batch = batch_result.map_err(|e| {
            error!("Failed to read batch: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let pool_addresses = get_string_column(&batch, "pool_address")?;
        let pool_names = get_string_column(&batch, "pool_name")?;
        let markout_times = get_string_column(&batch, "markout_time")?;
        let bucket_starts = get_float64_column(&batch, "bucket_range_start")?;
        let bucket_ends = get_float64_column(&batch, "bucket_range_end")?;
        let counts = get_uint64_column(&batch, "count")?;
        let labels = get_string_column(&batch, "label")?;

        for i in 0..batch.num_rows() {
            // Early filtering
            if pool_addresses.value(i).to_lowercase() != pool_address ||
               markout_times.value(i) != markout_time {
                continue;
            }

            // Get or set pool name
            if pool_name.is_empty() {
                pool_name = pool_names.value(i).to_string();
            }

            let count = counts.value(i);
            let label = labels.value(i);
            
            // Track the mode (most frequent) bucket
            if count > highest_bucket_count {
                highest_bucket_count = count;
                mode_bucket = label.to_string();
            }

            total_observations += count;

            buckets.push(HistogramBucket {
                range_start: bucket_starts.value(i),
                range_end: if bucket_ends.is_null(i) { None } else { Some(bucket_ends.value(i)) },
                count,
                label: label.to_string(),
            });
        }
    }

    if buckets.is_empty() {
        warn!(
            "No distribution data found for pool {} with markout time {}",
            pool_address,
            markout_time
        );
        return Err(StatusCode::NOT_FOUND);
    }

    // Sort buckets by range start for consistent ordering
    buckets.sort_by(|a, b| a.range_start.partial_cmp(&b.range_start).unwrap_or(std::cmp::Ordering::Equal));

    info!(
        "Retrieved distribution with {} buckets for {}. Most frequent range: {} ({:.2}% of {} total observations)", 
        buckets.len(),
        pool_name,
        mode_bucket,
        (highest_bucket_count as f64 / total_observations as f64) * 100.0,
        total_observations
    );

    Ok(Json(HistogramResponse {
        pool_name,
        pool_address,
        buckets,
        total_observations,
    }))
}

pub fn get_bucket_value(batch: &arrow::record_batch::RecordBatch, column_name: &str) -> Result<u64, StatusCode> {
    let idx = batch.schema().index_of(column_name).map_err(|e| {
        error!("Failed to find {} column: {}", column_name, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let column = batch.column(idx);
    match column.data_type() {
        DataType::UInt64 => column.as_any().downcast_ref::<UInt64Array>()
            .map(|arr| arr.value(0))
            .ok_or_else(|| {
                error!("Failed to cast {} as UInt64Array", column_name);
                StatusCode::INTERNAL_SERVER_ERROR
            }),
        DataType::Int64 => column.as_any().downcast_ref::<Int64Array>()
            .map(|arr| arr.value(0) as u64)
            .ok_or_else(|| {
                error!("Failed to cast {} as Int64Array", column_name);
                StatusCode::INTERNAL_SERVER_ERROR
            }),
        _ => {
            error!("Unexpected type for {}: {:?}", column_name, column.data_type());
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}