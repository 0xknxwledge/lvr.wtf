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
    pub pool_address: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RunningTotal {
    pub block_number: u64,
    pub markout: String,
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