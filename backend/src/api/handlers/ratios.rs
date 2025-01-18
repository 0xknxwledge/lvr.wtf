use axum::{
    extract::{State, Query},
    response::Json,
    http::StatusCode,
};
use crate::{AppState, MERGE_BLOCK,
    LVRRatioQuery, LVRRatioResponse, LVRTotals, MarkoutRatio,
    api::handlers::common::{get_uint64_column, get_float64_column, get_valid_pools,
    get_string_column}};
use tracing::{error, debug, info, warn};
use std::sync::Arc;
use parquet::arrow::arrow_reader::ParquetRecordBatchReader;
use arrow::array::Array;
use object_store::path::Path;

pub async fn get_lvr_ratios(
    State(state): State<Arc<AppState>>,
    Query(params): Query<LVRRatioQuery>,
) -> Result<Json<LVRRatioResponse>, StatusCode> {
    let start_block = params.start_block.unwrap_or(*MERGE_BLOCK);
    let end_block = params.end_block.unwrap_or(20_000_000);
    
    // Validate pool address if provided
    if let Some(ref pool_address) = params.pool_address {
        let valid_pools = get_valid_pools();
        if !valid_pools.contains(&pool_address.to_lowercase()) {
            warn!("Invalid pool address provided: {}", pool_address);
            return Err(StatusCode::BAD_REQUEST);
        }
    }
    
    info!("Fetching LVR ratios - Block range: {} to {}{}", 
        start_block, 
        end_block,
        params.pool_address.as_ref().map_or(String::new(), |p| format!(", pool: {}", p))
    );

    // Read from precomputed file
    let bytes = state.store.get(&Path::from("precomputed/ratios/lvr_ratios.parquet"))
        .await
        .map_err(|e| {
            error!("Failed to read precomputed LVR ratios: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .bytes()
        .await
        .map_err(|e| {
            error!("Failed to get bytes from precomputed LVR ratios: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let reader = ParquetRecordBatchReader::try_new(bytes, 1024)
        .map_err(|e| {
            error!("Failed to create Parquet reader: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut ratios = Vec::new();
    let mut total_theoretical = 0u64;
    let mut total_realized = 0u64;

    for batch_result in reader {
        let batch = batch_result.map_err(|e| {
            error!("Failed to read batch: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let markout_times = get_string_column(&batch, "markout_time")?;
        let ratio_values = get_float64_column(&batch, "ratio")?;
        let realized_cents = get_uint64_column(&batch, "realized_lvr_cents")?;
        let theoretical_cents = get_uint64_column(&batch, "theoretical_lvr_cents")?;

        for i in 0..batch.num_rows() {
            // Skip invalid data points
            if !theoretical_cents.is_valid(i) || !realized_cents.is_valid(i) {
                continue;
            }

            let realized = realized_cents.value(i);
            let theoretical = theoretical_cents.value(i);

            // Skip if both values are zero
            if realized == 0 && theoretical == 0 {
                continue;
            }

            // Only add to ratios if it matches our filters
            ratios.push(MarkoutRatio {
                markout_time: markout_times.value(i).to_string(),
                ratio: ratio_values.value(i),
                realized_lvr_cents: realized,
                theoretical_lvr_cents: theoretical,
            });
            
            total_realized = total_realized.saturating_add(realized);
            total_theoretical = total_theoretical.saturating_add(theoretical);
        }
    }

    // Sort ratios by markout time for consistent ordering
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

    let avg_ratio = if total_theoretical > 0 {
        (total_realized as f64 / total_theoretical as f64 * 100.0).min(100.0)
    } else {
        0.0
    };

    info!(
        "Retrieved {} LVR ratios, Average ratio: {:.2}% (Realized: ${:.2}, Theoretical: ${:.2})", 
        ratios.len(),
        avg_ratio,
        total_realized as f64 / 100.0,
        total_theoretical as f64 / 100.0
    );

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