use axum::{
    extract::{State, Query},
    response::Json,
    http::StatusCode,
};
use crate::{AppState, 
    HistogramBucket, HistogramResponse, HistogramQuery,
    api::handlers::common::{ get_pool_name, get_valid_pools}};
use tracing::{error, info, warn};
use futures::StreamExt;
use std::sync::Arc;
use parquet::arrow::arrow_reader::ParquetRecordBatchReader;
use arrow::array::{Int64Array,UInt64Array};
use arrow::datatypes::DataType;

pub async fn get_lvr_histogram(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HistogramQuery>,
) -> Result<Json<HistogramResponse>, StatusCode> {
    let pool_address = params.pool_address.to_lowercase();
    let markout_time = params.markout_time;
    
    info!(
        "Received histogram request - Pool: {}, Markout Time: {}", 
        pool_address, markout_time
    );

    // Check if pool is valid
    let valid_pools = get_valid_pools();
    if !valid_pools.contains(&pool_address) {
        warn!("Invalid pool address requested: {}", pool_address);
        return Err(StatusCode::BAD_REQUEST);
    }

    // Use lowercase for file pattern
    let checkpoint_pattern = format!("{}_{}.parquet", pool_address, markout_time);
    
    // List checkpoint files
    let checkpoints_path = object_store::path::Path::from("checkpoints");
    let mut checkpoint_files = state.store.list(Some(&checkpoints_path));
    
    while let Some(meta_result) = checkpoint_files.next().await {
        let meta = meta_result.map_err(|e| {
            error!("Failed to get file metadata: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR;
        })?;
        
        let file_path = meta.location.to_string();
        
        // Convert file path to lowercase for comparison
        if !file_path.to_lowercase().ends_with(&checkpoint_pattern) {
            continue;
        }

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

            // Get all non-zero bucket values
            let bucket_0_10 = get_bucket_value(&batch, "total_bucket_0_10")?;
            let bucket_10_100 = get_bucket_value(&batch, "total_bucket_10_100")?;
            let bucket_100_500 = get_bucket_value(&batch, "total_bucket_100_500")?;
            let bucket_1000_3000 = get_bucket_value(&batch, "total_bucket_1000_3000")?;
            let bucket_3000_10000 = get_bucket_value(&batch, "total_bucket_3000_10000")?;
            let bucket_10000_30000 = get_bucket_value(&batch, "total_bucket_10000_30000")?;
            let bucket_30000_plus = get_bucket_value(&batch, "total_bucket_30000_plus")?;

            // Create bucket objects
            let buckets = vec![
                HistogramBucket {
                    range_start: 0.01, // Start just above 0
                    range_end: Some(10.0),
                    count: bucket_0_10,
                    label: "$0.01-$10".to_string(),
                },
                HistogramBucket {
                    range_start: 10.0,
                    range_end: Some(100.0),
                    count: bucket_10_100,
                    label: "$10-$100".to_string(),
                },
                HistogramBucket {
                    range_start: 100.0,
                    range_end: Some(500.0),
                    count: bucket_100_500,
                    label: "$100-$500".to_string(),
                },
                HistogramBucket {
                    range_start: 1000.0,
                    range_end: Some(3000.0),
                    count: bucket_1000_3000,
                    label: "$1K-$3K".to_string(),
                },
                HistogramBucket {
                    range_start: 3000.0,
                    range_end: Some(10000.0),
                    count: bucket_3000_10000,
                    label: "$3K-$10K".to_string(),
                },
                HistogramBucket {
                    range_start: 10000.0,
                    range_end: Some(30000.0),
                    count: bucket_10000_30000,
                    label: "$10K-$30K".to_string(),
                },
                HistogramBucket {
                    range_start: 30000.0,
                    range_end: None,
                    count: bucket_30000_plus,
                    label: "$30K+".to_string(),
                },
            ];

            // Calculate total non-zero observations
            let total_non_zero_observations: u64 = buckets.iter().map(|b| b.count).sum();

            // Get pool name
            let pool_name = get_pool_name(&pool_address);

            return Ok(Json(HistogramResponse {
                pool_name,
                pool_address,
                buckets,
                total_observations: total_non_zero_observations, // Only count non-zero values
            }));
        }
    }

    // If we get here, we didn't find the file
    Err(StatusCode::NOT_FOUND)
}

fn get_bucket_value(batch: &arrow::record_batch::RecordBatch, column_name: &str) -> Result<u64, StatusCode> {
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