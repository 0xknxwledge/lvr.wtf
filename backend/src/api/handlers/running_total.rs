use axum::{
    extract::{State, Query},
    response::Json,
    http::StatusCode,
};
use crate::{AppState, 
    TimeRangeQuery, RunningTotal, 
    MERGE_BLOCK, api::handlers::common::{get_uint64_column, get_valid_pools, get_pool_name,
    get_string_column}};
use tracing::{error, info, warn};
use std::sync::Arc;
use parquet::arrow::arrow_reader::ParquetRecordBatchReader;
use object_store::path::Path;

pub async fn get_running_total(
    State(state): State<Arc<AppState>>,
    Query(params): Query<TimeRangeQuery>,
) -> Result<Json<Vec<RunningTotal>>, StatusCode> {
    let start_block = params.start_block.unwrap_or(*MERGE_BLOCK);
    let end_block = params.end_block.unwrap_or(20_000_000);
    let is_aggregate = params.aggregate.unwrap_or(false);
    
    // Early validation
    if !is_aggregate && params.pool.is_none() {
        warn!("Pool parameter required when not aggregating");
        return Err(StatusCode::BAD_REQUEST);
    }

    // Pool validation when specified
    if let Some(ref pool) = params.pool {
        let valid_pools = get_valid_pools();
        if !valid_pools.contains(&pool.to_lowercase()) {
            warn!("Invalid pool address provided: {}", pool);
            return Err(StatusCode::BAD_REQUEST);
        }
    }
    
    info!(
        "Fetching {} running total for blocks {} to {}{}", 
        if is_aggregate { "aggregated" } else { "individual" },
        start_block, 
        end_block,
        params.pool.as_ref().map_or(String::new(), |p| format!(", pool: {}", p))
    );

    let results = if is_aggregate {
        read_aggregate_running_totals(&state, start_block, end_block, params.markout_time).await?
    } else {
        read_individual_running_totals(&state, start_block, end_block, &params).await?
    };

    info!("Returning {} running total data points", results.len());
    Ok(Json(results))
}

async fn read_aggregate_running_totals(
    state: &Arc<AppState>,
    start_block: u64,
    end_block: u64,
    markout_filter: Option<String>,
) -> Result<Vec<RunningTotal>, StatusCode> {
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

    let mut results = Vec::new();

    for batch_result in reader {
        let batch = batch_result.map_err(|e| {
            error!("Failed to read batch: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let block_numbers = get_uint64_column(&batch, "block_number")?;
        let markout_times = get_string_column(&batch, "markout_time")?;
        let running_totals = get_uint64_column(&batch, "running_total_cents")?;

        for i in 0..batch.num_rows() {
            let block_number = block_numbers.value(i);
            
            // Skip if outside requested range
            if block_number < start_block || block_number > end_block {
                continue;
            }

            let markout_time = markout_times.value(i).to_string();
            
            // Apply markout time filter if specified
            if let Some(ref filter) = markout_filter {
                if filter != &markout_time {
                    continue;
                }
            }

            results.push(RunningTotal {
                block_number,
                markout: markout_time,
                pool_name: None,
                pool_address: None,
                running_total_cents: running_totals.value(i),
            });
        }
    }

    // Sort results by block number and markout time
    results.sort_by(|a, b| {
        a.block_number
            .cmp(&b.block_number)
            .then_with(|| a.markout.to_lowercase().cmp(&b.markout.to_lowercase()))
    });

    Ok(results)
}

async fn read_individual_running_totals(
    state: &Arc<AppState>,
    start_block: u64,
    end_block: u64,
    params: &TimeRangeQuery,
) -> Result<Vec<RunningTotal>, StatusCode> {
    // Read from precomputed individual file
    let bytes = state.store.get(&Path::from("precomputed/running_totals/individual.parquet"))
        .await
        .map_err(|e| {
            error!("Failed to read precomputed individual running totals: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .bytes()
        .await
        .map_err(|e| {
            error!("Failed to get bytes from precomputed individual data: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let reader = ParquetRecordBatchReader::try_new(bytes, 1024)
        .map_err(|e| {
            error!("Failed to create Parquet reader: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut results = Vec::new();

    for batch_result in reader {
        let batch = batch_result.map_err(|e| {
            error!("Failed to read batch: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let block_numbers = get_uint64_column(&batch, "block_number")?;
        let markout_times = get_string_column(&batch, "markout_time")?;
        let pool_addresses = get_string_column(&batch, "pool_address")?;
        let running_totals = get_uint64_column(&batch, "running_total_cents")?;

        for i in 0..batch.num_rows() {
            let block_number = block_numbers.value(i);
            
            // Skip if outside requested range
            if block_number < start_block || block_number > end_block {
                continue;
            }

            let markout_time = markout_times.value(i).to_string();
            let pool_address = pool_addresses.value(i).to_lowercase();

            // Apply markout time filter if specified
            if let Some(ref filter) = params.markout_time {
                if filter != &markout_time {
                    continue;
                }
            }

            // Apply pool filter
            if let Some(ref requested_pool) = params.pool {
                if requested_pool.to_lowercase() != pool_address {
                    continue;
                }
            }

            results.push(RunningTotal {
                block_number,
                markout: markout_time,
                pool_name: Some(get_pool_name(&pool_address)),
                pool_address: Some(pool_address),
                running_total_cents: running_totals.value(i),
            });
        }
    }

    // Sort results by block number, markout time, and pool name
    results.sort_by(|a, b| {
        a.block_number
            .cmp(&b.block_number)
            .then_with(|| a.markout.to_lowercase().cmp(&b.markout.to_lowercase()))
            .then(a.pool_name.cmp(&b.pool_name))
    });

    Ok(results)
}