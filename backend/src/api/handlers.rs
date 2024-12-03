use axum::{
    extract::{State, Query},
    response::{Json, IntoResponse},
    http::StatusCode,
};
use time::OffsetDateTime;
use crate::{AppState, 
    TimeRangeQuery, RunningTotal, IntervalAPIData, 
    HealthResponse, MERGE_BLOCK,
    LVRRatioQuery, LVRRatioResponse, 
    MarkoutRatio, LVRTotals};
use tracing::{error, debug, info};
use futures::StreamExt;
use std::{sync::Arc, collections::HashMap};
use parquet::arrow::arrow_reader::ParquetRecordBatchReader;
use arrow::array::{StringArray, UInt64Array, Int64Array, Array};


const BLOCKS_PER_INTERVAL: u64 = 7200;
const FINAL_PARTIAL_BLOCKS: u64 = 5808;
const FINAL_INTERVAL_FILE: &str = "19857392_20000000.parquet";


pub async fn health_check() -> impl IntoResponse {
    let response = HealthResponse {
        status: "OK",
        version: env!("CARGO_PKG_VERSION"),
        timestamp: OffsetDateTime::now_utc().to_string(),
    };

    (StatusCode::OK, Json(response))
}

pub async fn get_running_total(
    State(state): State<Arc<AppState>>,
    Query(params): Query<TimeRangeQuery>,
) -> Result<Json<Vec<RunningTotal>>, StatusCode> {
    let start_block = params.start_block.unwrap_or(*MERGE_BLOCK);
    let end_block = params.end_block.unwrap_or(u64::MAX);
    
    debug!("Fetching running total for blocks {} to {}", start_block, end_block);
    
    let mut interval_totals: HashMap<(u64, String), IntervalAPIData> = HashMap::new();
    let intervals_path = object_store::path::Path::from("intervals");
    info!("Attempting to read from path: {:?}", intervals_path);
    let mut interval_files = state.store.list(Some(&intervals_path));

    while let Some(meta_result) = interval_files.next().await {
        let meta = meta_result.map_err(|e| {
            error!("Failed to get file metadata: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let file_path = meta.location.to_string();
        
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

            // Get column indices with Arrow error handling
            let interval_id_idx = batch.schema().index_of("interval_id").map_err(|e| {
                error!("Failed to get interval_id column index: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            let markout_time_idx = batch.schema().index_of("markout_time").map_err(|e| {
                error!("Failed to get markout_time column index: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            let total_lvr_cents_idx = batch.schema().index_of("total_lvr_cents").map_err(|e| {
                error!("Failed to get total_lvr_cents column index: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            // Safely cast columns
            let interval_ids = batch
                .column(interval_id_idx)
                .as_any()
                .downcast_ref::<UInt64Array>()
                .ok_or_else(|| {
                    error!("Failed to cast interval_id column to UInt64Array");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;

            let markout_times = batch
                .column(markout_time_idx)
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or_else(|| {
                    error!("Failed to cast markout_time column to StringArray");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;

                let total_lvr_cents = batch
                .column(total_lvr_cents_idx)
                .as_any()
                .downcast_ref::<Int64Array>() // Changed from UInt64Array to Int64Array
                .ok_or_else(|| {
                    error!("Failed to cast total_lvr_cents column to Int64Array");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;

                for i in 0..batch.num_rows() {
                    if !total_lvr_cents.is_null(i) {
                        let interval_id = interval_ids.value(i);
                        let markout_time = markout_times.value(i).to_string();
                        let lvr_cents = total_lvr_cents.value(i);
            
                        // Convert to unsigned if needed while handling negative values
                        let unsigned_cents = if lvr_cents < 0 {
                            0
                        } else {
                            lvr_cents as u64
                        };
            
                        interval_totals
                            .entry((interval_id, markout_time))
                            .and_modify(|data| data.total += unsigned_cents)
                            .or_insert(IntervalAPIData {
                                total: unsigned_cents,
                                file_path: file_path.clone(),
                            });
                    }
                }
        }
    }

    let mut running_totals: HashMap<String, Vec<(u64, u64)>> = HashMap::new();
    
    for ((interval_id, markout), interval_data) in interval_totals {
        let is_final_interval = interval_data.file_path.ends_with(FINAL_INTERVAL_FILE);
        
        let block_number = if interval_id == 19 && is_final_interval {
            start_block + (interval_id * BLOCKS_PER_INTERVAL) + FINAL_PARTIAL_BLOCKS
        } else {
            start_block + ((interval_id + 1) * BLOCKS_PER_INTERVAL)
        };

        if block_number >= start_block && block_number <= end_block {
            running_totals
                .entry(markout)
                .or_default()
                .push((block_number, interval_data.total));
        }
    }

    let mut results = Vec::new();
    for (markout, totals) in running_totals {
        let mut sorted_totals = totals;
        sorted_totals.sort_by_key(|&(block, _)| block);
        
        let mut running_sum = 0u64;
        for (block_number, cents) in sorted_totals {
            running_sum += cents;
            results.push(RunningTotal {
                block_number,
                markout: markout.clone(),
                running_total_cents: running_sum,
            });
        }
    }

    results.sort_by(|a, b| {
        a.block_number.cmp(&b.block_number)
            .then(a.markout.cmp(&b.markout))
    });

    info!(
        "Returning {} running total data points",
        results.len()
    );

    Ok(Json(results))
}

pub async fn get_lvr_ratios(
    State(state): State<Arc<AppState>>,
    Query(params): Query<LVRRatioQuery>,
) -> Result<Json<LVRRatioResponse>, StatusCode> {
    info!("Fetching LVR ratios with params: {:?}", params);
    
    let mut totals = LVRTotals {
        realized: 0,
        theoretical: HashMap::new(),
    };

    let intervals_path = object_store::path::Path::from("intervals");
    info!("Attempting to read from path: {:?}", intervals_path);
    let mut interval_files = state.store.list(Some(&intervals_path));

    // Process each interval file
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

        // Process each batch in the file
        for batch_result in record_reader {
            let batch = batch_result.map_err(|e| {
                error!("Failed to read batch: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            let markout_times = batch
                .column(batch.schema().index_of("markout_time").unwrap())
                .as_any()
                .downcast_ref::<StringArray>()
                .unwrap();

                let total_lvr_cents = batch
                .column(batch.schema().index_of("total_lvr_cents").map_err(|e| {
                    error!("Failed to get total_lvr_cents column index: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?)
                .as_any()
                .downcast_ref::<Int64Array>() // Changed from UInt64Array to Int64Array
                .ok_or_else(|| {
                    error!("Failed to cast total_lvr_cents column to Int64Array");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;

            // Process each row in the batch
            for i in 0..batch.num_rows() {
                if !total_lvr_cents.is_null(i) {
                    let markout_time = markout_times.value(i);
                    let lvr_cents = total_lvr_cents.value(i);
                    
                    // Convert to unsigned while handling negative values
                    let unsigned_cents = if lvr_cents < 0 {
                        0
                    } else {
                        lvr_cents as u64
                    };

                    if markout_time == "brontes" {
                        totals.realized = totals.realized.saturating_add(unsigned_cents);
                    } else {
                        totals.theoretical
                            .entry(markout_time.to_string())
                            .and_modify(|e| *e = e.saturating_add(unsigned_cents))
                            .or_insert(unsigned_cents);
                    }
                }
            }
        }
    }

    // Calculate ratios for each markout time
    let mut ratios = Vec::new();
    
    if totals.realized > 0 {
        for (markout_time, theoretical_lvr) in totals.theoretical {
            if theoretical_lvr > 0 {
                let ratio = (totals.realized as f64 / theoretical_lvr as f64) * 100.0;
                
                ratios.push(MarkoutRatio {
                    markout_time,
                    ratio: ratio.min(100.0), // Cap at 100%
                    realized_lvr_cents: totals.realized,
                    theoretical_lvr_cents: theoretical_lvr,
                });
            }
        }
    }

    // Sort by markout time for consistent ordering
    ratios.sort_by(|a, b| a.markout_time.cmp(&b.markout_time));

    info!("Returning {} LVR ratios", ratios.len());

    Ok(Json(LVRRatioResponse { ratios }))
}