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

pub async fn get_lvr_histogram(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HistogramQuery>,
) -> Result<Json<HistogramResponse>, StatusCode> {
    let pool_address = params.pool_address.to_lowercase();
    let markout_time = params.markout_time;
    
    info!(
        "Fetching precomputed histogram data - Pool: {}, Markout Time: {}", 
        pool_address, markout_time
    );

    // Check if pool is valid
    let valid_pools = get_valid_pools();
    if !valid_pools.contains(&pool_address) {
        warn!("Invalid pool address requested: {}", pool_address);
        return Err(StatusCode::BAD_REQUEST);
    }

    // Read from precomputed file
    let precomputed_path = object_store::path::Path::from("precomputed/distributions/histograms.parquet");
    
    let bytes = state.store.get(&precomputed_path)
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
            // Filter by pool and markout time
            if pool_addresses.value(i).to_lowercase() != pool_address ||
               markout_times.value(i) != markout_time {
                continue;
            }

            // Store pool name on first match
            if pool_name.is_empty() {
                pool_name = pool_names.value(i).to_string();
            }

            let count = counts.value(i);
            total_observations += count;

            buckets.push(HistogramBucket {
                range_start: bucket_starts.value(i),
                range_end: if bucket_ends.is_null(i) { None } else { Some(bucket_ends.value(i)) },
                count,
                label: labels.value(i).to_string(),
            });
        }
    }

    if buckets.is_empty() {
        warn!(
            "No histogram data found for pool {} with markout time {}",
            pool_address,
            markout_time
        );
        return Err(StatusCode::NOT_FOUND);
    }

    info!(
        "Retrieved histogram data with {} buckets and {} total observations", 
        buckets.len(),
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