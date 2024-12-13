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
    pub pool: Option<String>,
}


#[derive(Debug, Serialize)]
pub struct RunningTotal {
    pub block_number: u64,
    pub markout: String,
    pub pool_name: Option<String>, 
    pub pool_address: Option<String>,
    pub running_total_cents: u64,
}

#[derive(Debug, Serialize)]
pub struct IntervalAPIData {
    pub total: u64,
    pub file_path: String,
}


#[derive(Debug, Deserialize)]
pub struct BoxplotLVRQuery {
    pub markout_time: String,
    pub start_block: Option<u64>,
    pub end_block: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct PoolBoxplotData {
    pub pool_name: String,
    pub pool_address: String,
    pub percentile_25_cents: u64,
    pub median_cents: u64,
    pub percentile_75_cents: u64,
    pub max_lvr_cents: u64,
    pub max_lvr_block: u64,
}

#[derive(Debug, Serialize)]
pub struct BoxplotLVRResponse {
    pub markout_time: String,
    pub pool_data: Vec<PoolBoxplotData>,
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
pub struct MaxLVRQuery {
    pub pool_address: String,
    pub markout_time: String,
}

#[derive(Debug, Serialize)]
pub struct MaxLVRResponse {
    pub block_number: u64,
    pub lvr_cents: u64,
    pub pool_name: String,
}
#[derive(Debug, Deserialize)]
pub struct HistogramQuery {
    pub pool_address: String,
    pub markout_time: String,
}

#[derive(Debug, Serialize)]
pub struct HistogramBucket {
    pub range_start: f64,
    pub range_end: Option<f64>,
    pub count: u64,
    pub label: String,
}

#[derive(Debug, Serialize)]
pub struct HistogramResponse {
    pub pool_name: String,
    pub pool_address: String,
    pub buckets: Vec<HistogramBucket>,
    pub total_observations: u64,
}

#[derive(Debug, Deserialize)]
pub struct NonZeroProportionQuery {
    pub pool_address: String,
    pub markout_time: String,
}

#[derive(Debug, Serialize)]
pub struct NonZeroProportionResponse {
    pub pool_name: String,
    pub pool_address: String,
    pub non_zero_proportion: f64,
}

#[derive(Debug, Deserialize)]
pub struct PercentileBandQuery {
    pub start_block: Option<u64>,
    pub end_block: Option<u64>,
    pub pool_address: Option<String>,
    pub markout_time: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PercentileDataPoint {
    pub block_number: u64,
    pub percentile_25_cents: u64,
    pub median_cents: u64,
    pub percentile_75_cents: u64
}

#[derive(Debug, Serialize)]
pub struct PercentileBandResponse {
    pub pool_name: String,
    pub pool_address: String,
    pub markout_time: String,
    pub data_points: Vec<PercentileDataPoint>,
}


#[derive(Debug)]
pub struct AggregatedStats {
    pub percentile_25: u64,
    pub median: u64,
    pub percentile_75: u64,
    pub count: u64,
}