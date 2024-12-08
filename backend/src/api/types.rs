use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub version: &'static str,
    pub timestamp: String,
}

#[derive(Debug, Deserialize)]
pub struct TimeRangeQuery {
    pub start_block: Option<u64>,
    pub end_block: Option<u64>,
    pub markout_time: Option<String>,
    pub aggregate: Option<bool>, 
}

#[derive(Debug, Serialize)]
pub struct RunningTotal {
    pub block_number: u64,
    pub markout: String,
    pub pool_name: Option<String>, 
    pub running_total_cents: u64,
}

#[derive(Debug, Serialize)]
pub struct IntervalAPIData {
    pub total: u64,
    pub file_path: String,
}

#[derive(Debug, Serialize)]
pub struct LVRRatioResponse {
    /// Vector of ratios for each markout time
    pub ratios: Vec<MarkoutRatio>,
}

#[derive(Debug, Serialize)]
pub struct MarkoutRatio {
    pub markout_time: String,
    pub ratio: f64,
    pub realized_lvr_cents: u64,
    pub theoretical_lvr_cents: u64,
}

#[derive(Debug, Deserialize)]
pub struct LVRRatioQuery {
    pub start_block: Option<u64>,
    pub end_block: Option<u64>,
    /// Optional pool address for filtering data
    pub pool_address: Option<String>,
}

#[derive(Debug)]
pub struct LVRTotals {
    pub realized: u64,
    pub theoretical: HashMap<String, u64>,
}

#[derive(Debug, Deserialize)]
pub struct PoolTotalsQuery {
    pub markout_time: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PoolTotal {
    pub pool_name: String,
    pub pool_address: String,
    pub total_lvr_cents: u64,
}

#[derive(Debug, Serialize)]
pub struct PoolTotalsResponse {
    pub totals: Vec<PoolTotal>,
}

#[derive(Debug, Deserialize)]
pub struct MedianLVRQuery {
    pub markout_time: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PoolMedianLVR {
    pub pool_name: String,
    pub pool_address: String,
    pub median_lvr_cents: u64,
}

#[derive(Debug, Serialize)]
pub struct MedianLVRResponse {
    pub medians: Vec<PoolMedianLVR>,
}