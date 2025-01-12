use axum::{
    extract::{State, Query},
    response::Json,
    http::StatusCode,
};
use crate::{AppState, 
    LVRRatioQuery, LVRRatioResponse, LVRTotals, MarkoutRatio,
    api::handlers::common::{get_uint64_column, get_valid_pools,
    get_string_column}};
use tracing::{error, debug, info};
use futures::StreamExt;
use std::{sync::Arc, collections::HashMap};
use parquet::arrow::arrow_reader::ParquetRecordBatchReader;
use arrow::array::Array;


pub async fn get_lvr_ratios(
    State(state): State<Arc<AppState>>,
    Query(params): Query<LVRRatioQuery>,
) -> Result<Json<LVRRatioResponse>, StatusCode> {
    info!("Fetching LVR ratios with params: {:?}", params);
    
    let valid_pools = get_valid_pools();
    let mut totals = LVRTotals {
        realized: 0,
        theoretical: HashMap::new(),
    };

    let mut files_processed = 0;
    let intervals_path = object_store::path::Path::from("intervals");
    let mut interval_files = state.store.list(Some(&intervals_path));

    while let Some(meta_result) = interval_files.next().await {
        files_processed += 1;
        let meta = meta_result.map_err(|e| {
            error!("Failed to get file metadata: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        debug!("Processing interval file {}: {}", files_processed, meta.location);

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
                if total_lvr_cents.is_null(i) || non_zero_counts.value(i) == 0 {
                    continue;
                }

                let pool_address = pool_addresses.value(i).to_lowercase();
                if !valid_pools.contains(&pool_address) {
                    continue;
                }

                let markout_time = markout_times.value(i);
                let lvr_cents = total_lvr_cents.value(i);

                // Only include intervals that had actual activity
                if lvr_cents > 0 {
                    if markout_time == "brontes" {
                        totals.realized = totals.realized.saturating_add(lvr_cents);
                        debug!("Added {} cents to realized total", lvr_cents);
                    } else {
                        totals.theoretical
                            .entry(markout_time.to_string())
                            .and_modify(|e| *e = e.saturating_add(lvr_cents))
                            .or_insert(lvr_cents);
                        debug!("Added {} cents to theoretical total for markout {}", lvr_cents, markout_time);
                    }
                }
            }
        }
    }

    info!(
        "Processed {} files. Found realized total of {} cents and {} theoretical markout times",
        files_processed,
        totals.realized,
        totals.theoretical.len()
    );

    let ratios = calculate_lvr_ratios(totals);
    
    info!("Calculated {} LVR ratios", ratios.len());
    Ok(Json(LVRRatioResponse { ratios }))
}

// calculate_lvr_ratios remains the same
pub fn calculate_lvr_ratios(totals: LVRTotals) -> Vec<MarkoutRatio> {
    let mut ratios = Vec::new();
    
    // Only calculate ratios if we have realized LVR data
    if totals.realized > 0 {
        for (markout_time, theoretical_lvr) in totals.theoretical {
            // Only include ratios where we have theoretical data
            if theoretical_lvr > 0 {
                // Calculate the ratio as a percentage
                let ratio = (totals.realized as f64 / theoretical_lvr as f64) * 100.0;
                let capped_ratio = ratio.min(100.0);
                
                debug!(
                    "Calculated ratio for {}: {:.2}% (realized: {}, theoretical: {})",
                    markout_time, capped_ratio, totals.realized, theoretical_lvr
                );
                
                ratios.push(MarkoutRatio {
                    markout_time,
                    ratio: capped_ratio,
                    realized_lvr_cents: totals.realized,
                    theoretical_lvr_cents: theoretical_lvr,
                });
            }
        }
    }

    // Sort by markout time for consistent ordering
    ratios.sort_by(|a, b| {
        if a.markout_time == "brontes" {
            std::cmp::Ordering::Greater
        } else if b.markout_time == "brontes" {
            std::cmp::Ordering::Less
        } else {
            match (a.markout_time.parse::<f64>(), b.markout_time.parse::<f64>()) {
                (Ok(a_val), Ok(b_val)) => a_val.partial_cmp(&b_val).unwrap_or(std::cmp::Ordering::Equal),
                _ => a.markout_time.cmp(&b.markout_time)
            }
        }
    });

    ratios
}