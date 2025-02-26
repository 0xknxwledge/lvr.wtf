use axum::{
    extract::State,
    response::Json,
    http::StatusCode,
};
use crate::{AppState, api::handlers::common::get_string_column, TotalLVRResponse, MarkoutTotal};
use tracing::{error, info};
use std::sync::Arc;
use parquet::arrow::arrow_reader::ParquetRecordBatchReader;
use object_store::path::Path;


pub async fn get_total_lvr(
    State(state): State<Arc<AppState>>,
) -> Result<Json<TotalLVRResponse>, StatusCode> {
    info!("Fetching latest LVR totals across all markout times (excluding Brontes)");
    
    // Read from precomputed aggregate file
    let bytes = state.store.get(&Path::from("precomputed/running_totals/aggregate.parquet"))
        .await
        .map_err(|e| {
            error!("Failed to read precomputed aggregate running totals: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .bytes()
        .await
        .map_err(|e| {
            error!("Failed to get bytes from precomputed aggregate data: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let reader = ParquetRecordBatchReader::try_new(bytes, 1024)
        .map_err(|e| {
            error!("Failed to create Parquet reader: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Track the latest block number for each markout time
    let mut latest_blocks: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
    
    // First pass: find the latest block for each markout time
    for batch_result in reader {
        let batch = batch_result.map_err(|e| {
            error!("Failed to read batch: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let block_numbers = crate::api::handlers::common::get_uint64_column(&batch, "block_number")?;
        let markout_times = get_string_column(&batch, "markout_time")?;

        for i in 0..batch.num_rows() {
            let markout_time = markout_times.value(i).to_string();
            
            // Skip Brontes
            if markout_time.to_lowercase() == "brontes" {
                continue;
            }
            
            let block_number = block_numbers.value(i);
            
            // Update latest block number for this markout time
            latest_blocks
                .entry(markout_time)
                .and_modify(|latest| *latest = std::cmp::max(*latest, block_number))
                .or_insert(block_number);
        }
    }

    // Now read the file again to get the total for each markout time at its latest block
    let bytes = state.store.get(&Path::from("precomputed/running_totals/aggregate.parquet"))
        .await
        .map_err(|e| {
            error!("Failed to read precomputed aggregate running totals: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .bytes()
        .await
        .map_err(|e| {
            error!("Failed to get bytes from precomputed aggregate data: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let reader = ParquetRecordBatchReader::try_new(bytes, 1024)
        .map_err(|e| {
            error!("Failed to create Parquet reader: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut markout_totals = Vec::new();

    for batch_result in reader {
        let batch = batch_result.map_err(|e| {
            error!("Failed to read batch: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let block_numbers = crate::api::handlers::common::get_uint64_column(&batch, "block_number")?;
        let markout_times = get_string_column(&batch, "markout_time")?;
        let running_totals = crate::api::handlers::common::get_uint64_column(&batch, "running_total_cents")?;

        for i in 0..batch.num_rows() {
            let markout_time = markout_times.value(i).to_string();
            
            // Skip Brontes
            if markout_time.to_lowercase() == "brontes" {
                continue;
            }
            
            let block_number = block_numbers.value(i);
            
            // Check if this is the latest block for this markout time
            if let Some(&latest) = latest_blocks.get(&markout_time) {
                if block_number == latest {
                    let total_cents = running_totals.value(i);
                    let total_dollars = total_cents as f64 / 100.0;
                    
                    markout_totals.push(MarkoutTotal {
                        markout_time: markout_time.clone(),
                        total_dollars,
                    });
                }
            }
        }
    }

    // Sort by markout time for consistent presentation
    markout_totals.sort_by(|a, b| a.markout_time.cmp(&b.markout_time));

    info!(
        "Successfully retrieved latest LVR totals for {} markout times (excluding Brontes)",
        markout_totals.len()
    );

    Ok(Json(TotalLVRResponse {
        markout_totals,
    }))
}