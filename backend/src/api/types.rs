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
    pub markout_time: String,
}

#[derive(Debug, Serialize)]
pub struct MaxLVRPoolData {
    pub pool_name: String,
    pub pool_address: String,
    pub block_number: u64,
    pub lvr_cents: u64,
}

#[derive(Debug, Serialize)]
pub struct MaxLVRResponse {
    pub pools: Vec<MaxLVRPoolData>,
}


#[derive(Debug, Deserialize)]
pub struct HistogramQuery {
    pub pool_address: String,
    pub markout_time: String,
}

#[derive(Debug, Serialize, Clone)]
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
    pub total_blocks: u64,
    pub non_zero_blocks: u64,
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
    pub start_block: u64,
    pub end_block: u64,
    pub total_lvr_dollars: f64,
    pub percentile_25_dollars: f64,
    pub median_dollars: f64,
    pub percentile_75_dollars: f64
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

#[derive(Debug, Deserialize)]
pub struct ClusterQuery {
    pub markout_time: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ClusterTotal {
    pub name: String,
    pub total_lvr_cents: u64,
}

#[derive(Debug, Serialize)]
pub struct ClusterPieResponse {
    pub clusters: Vec<ClusterTotal>,
    pub total_lvr_cents: u64,
}

#[derive(Debug, Deserialize)]
pub struct ClusterHistogramQuery {
    pub markout_time: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ClusterHistogramBucket {
    pub range_start: f64,
    pub range_end: Option<f64>,
    pub count: u64,
    pub label: String,
}

#[derive(Debug, Serialize)]
pub struct ClusterHistogramData {
    pub name: String,
    pub buckets: Vec<ClusterHistogramBucket>,
    pub total_observations: u64,
}

#[derive(Debug, Serialize)]
pub struct ClusterHistogramResponse {
    pub clusters: Vec<ClusterHistogramData>,
}

#[derive(Debug, Deserialize)]
pub struct MonthlyClusterQuery {
    pub markout_time: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MonthlyData {
    pub time_range: String,
    pub cluster_totals: HashMap<String, u64>,
    pub total_lvr_cents: u64,
}

#[derive(Debug, Serialize)]
pub struct ClusterMonthlyResponse {
    pub monthly_data: Vec<MonthlyData>,
    pub clusters: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ClusterNonZeroQuery {
    pub markout_time: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ClusterNonZero {
    pub name: String,
    pub total_observations: u64,
    pub non_zero_observations: u64,
    pub non_zero_proportion: f64,
}

#[derive(Debug, Serialize)]
pub struct ClusterNonZeroResponse {
    pub clusters: Vec<ClusterNonZero>,
}

#[derive(Debug, Deserialize)]
pub struct QuartilePlotQuery {
    pub pool_address: String,
    pub markout_time: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct QuartilePlotResponse {
    pub markout_time: String,
    pub pool_name: String,
    pub pool_address: String,
    pub percentile_25_cents: u64,
    pub median_cents: u64,
    pub percentile_75_cents: u64,
}

#[derive(Debug, Deserialize)]
pub struct DistributionQuery {
    pub pool_address: String,
    pub markout_time: String,
}

#[derive(Debug, Serialize)]
pub struct DistributionResponse {
    pub pool_name: String,
    pub pool_address: String,
    pub markout_time: String,
    pub mean: f64,
    pub std_dev: f64,
    pub skewness: f64,
    pub kurtosis: f64
}

#[derive(Debug, Serialize)]
pub struct TotalLVRResponse {
    pub markout_totals: Vec<MarkoutTotal>,
}

#[derive(Debug, Serialize)]
pub struct MarkoutTotal {
    pub markout_time: String,
    pub total_dollars: f64,
}