use arrow::array::{StringArray, UInt64Array, Float64Array, Array, Int64Array};
use arrow::record_batch::RecordBatch;
use axum::http::StatusCode;
use tracing::error;
use std::collections::HashSet;
use crate::{POOL_NAMES, POOL_ADDRESSES};
use arrow::datatypes::DataType;

pub const BLOCKS_PER_INTERVAL: u64 = 7200;
pub const FINAL_PARTIAL_BLOCKS: u64 = 5808;
pub const FINAL_INTERVAL_FILE: &str = "19857392_20000000.parquet";

pub fn get_valid_pools() -> HashSet<String> {
    POOL_ADDRESSES.iter()
        .map(|&addr| addr.to_lowercase())
        .collect()
}

pub fn get_pool_name(pool_address: &str) -> String {
    POOL_NAMES
        .iter()
        .find(|(addr, _)| addr.to_lowercase() == pool_address)
        .map(|(_, name)| name.to_string())
        .unwrap_or_else(|| pool_address.to_string())
}

pub fn calculate_block_number(base_block: u64, interval_id: u64, file_path: &str) -> u64 {
    let file_start = file_path
        .split("intervals/")
        .nth(1)
        .and_then(|name| name.trim_end_matches(".parquet").split('_').next())
        .and_then(|num| num.parse::<u64>().ok())
        .unwrap_or(base_block);

    if file_path.ends_with(FINAL_INTERVAL_FILE) && interval_id == 19 {
        file_start + (interval_id * BLOCKS_PER_INTERVAL) + FINAL_PARTIAL_BLOCKS
    } else {
        file_start + (interval_id * BLOCKS_PER_INTERVAL)
    }
}

pub fn calculate_percentile(sorted_values: &[u64], percentile: f64) -> u64 {
    if sorted_values.is_empty() {
        return 0;
    }
    
    if sorted_values.len() == 1 {
        return sorted_values[0];
    }
    
    let n = sorted_values.len() as f64;
    let rank = (n - 1.0) * percentile;
    let k = rank.floor() as usize;
    let d = rank - k as f64;
    
    if k + 1 >= sorted_values.len() {
        sorted_values[sorted_values.len() - 1]
    } else {
        let lower = sorted_values[k] as f64;
        let upper = sorted_values[k + 1] as f64;
        ((1.0 - d) * lower + d * upper).round() as u64
    }
}

pub fn get_column_value<A: Array + 'static>(
    batch: &RecordBatch, 
    column_name: &str
) -> Result<u64, StatusCode> {
    let idx = batch.schema().index_of(column_name).map_err(|e| {
        error!("Failed to find {} column: {}", column_name, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let column = batch.column(idx);
    match column.data_type() {
        DataType::UInt64 => {
            column.as_any()
                .downcast_ref::<UInt64Array>()
                .map(|arr| arr.value(0))
                .ok_or_else(|| {
                    error!("Failed to cast {} as UInt64Array", column_name);
                    StatusCode::INTERNAL_SERVER_ERROR
                })
        }
        DataType::Int64 => {
            column.as_any()
                .downcast_ref::<Int64Array>()
                .map(|arr| arr.value(0) as u64)
                .ok_or_else(|| {
                    error!("Failed to cast {} as Int64Array", column_name);
                    StatusCode::INTERNAL_SERVER_ERROR
                })
        }
        _ => {
            error!("Unexpected type for {}: {:?}", column_name, column.data_type());
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub fn get_string_column<'a>(batch: &'a RecordBatch, name: &str) -> Result<&'a StringArray, StatusCode> {
    batch
        .column(batch.schema().index_of(name).map_err(|e| {
            error!("Failed to get {} column index: {}", name, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| {
            error!("Failed to cast {} column to StringArray", name);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

pub fn get_uint64_column<'a>(batch: &'a RecordBatch, name: &str) -> Result<&'a UInt64Array, StatusCode> {
    batch
        .column(batch.schema().index_of(name).map_err(|e| {
            error!("Failed to get {} column index: {}", name, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
        .as_any()
        .downcast_ref::<UInt64Array>()
        .ok_or_else(|| {
            error!("Failed to cast {} column to UInt64Array", name);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

pub fn get_float64_column<'a>(batch: &'a RecordBatch, name: &str) -> Result<&'a Float64Array, StatusCode> {
    batch
        .column(batch.schema().index_of(name).map_err(|e| {
            error!("Failed to get {} column index: {}", name, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
        .as_any()
        .downcast_ref::<Float64Array>()
        .ok_or_else(|| {
            error!("Failed to cast {} column to Float64Array", name);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}