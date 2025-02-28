use axum::{
    extract::{State, Query},
    response::Json,
    http::StatusCode,
};
use std::{sync::Arc, collections::HashMap};
use tracing::{error, info, warn};
use parquet::arrow::arrow_reader::ParquetRecordBatchReader;
use arrow::array::Array;
use crate::{
    AppState,
    api::handlers::common::{get_uint64_column, get_string_column, get_float64_column},
    STABLE_POOLS, WBTC_WETH_POOLS, USDC_WETH_POOLS, USDT_WETH_POOLS, INTERVAL_RANGES,
    DAI_WETH_POOLS, USDC_WBTC_POOLS, ALTCOIN_WETH_POOLS,
    ClusterPieResponse, ClusterQuery, ClusterTotal,
    ClusterHistogramBucket, ClusterHistogramData, ClusterHistogramQuery, ClusterHistogramResponse,
    MonthlyClusterQuery, MonthlyData, ClusterMonthlyResponse,
    ClusterNonZero, ClusterNonZeroQuery, ClusterNonZeroResponse
};
use object_store::path::Path;


pub fn get_cluster_name(pool_address: &str) -> Option<&'static str> {
    let address = pool_address.to_lowercase();
    if STABLE_POOLS.contains_key(address.as_str()) {
        Some("Stable Pairs")
    } else if WBTC_WETH_POOLS.contains_key(address.as_str()) {
        Some("WBTC-WETH")
    } else if USDC_WETH_POOLS.contains_key(address.as_str()) {
        Some("USDC-WETH")
    } else if USDT_WETH_POOLS.contains_key(address.as_str()) {
        Some("USDT-WETH")
    } else if DAI_WETH_POOLS.contains_key(address.as_str()) {
        Some("DAI-WETH")
    } else if USDC_WBTC_POOLS.contains_key(address.as_str()) {
        Some("USDC-WBTC")
    } else if ALTCOIN_WETH_POOLS.contains_key(address.as_str()) {
        Some("Altcoin-WETH")
    } else {
        None
    }
}

pub async fn get_cluster_proportion(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ClusterQuery>,
) -> Result<Json<ClusterPieResponse>, StatusCode> {
    let markout_time = params.markout_time.unwrap_or_else(|| String::from("brontes"));
    
    info!(
        "Analyzing cluster distribution metrics for markout time: {}", 
        markout_time
    );

    // Read from precomputed file
    let bytes = state.store.get(&Path::from("precomputed/clusters/proportions.parquet"))
        .await
        .map_err(|e| {
            error!("Failed to read precomputed cluster distribution data: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .bytes()
        .await
        .map_err(|e| {
            error!("Failed to get bytes from precomputed cluster data: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let reader = ParquetRecordBatchReader::try_new(bytes, 1024)
        .map_err(|e| {
            error!("Failed to create Parquet reader: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut clusters = Vec::new();
    let mut total_lvr_cents = 0u64;
    let mut largest_cluster_name = String::new();
    let mut largest_cluster_amount = 0u64;

    for batch_result in reader {
        let batch = batch_result.map_err(|e| {
            error!("Failed to read batch: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let cluster_names = get_string_column(&batch, "cluster_name")?;
        let markout_times = get_string_column(&batch, "markout_time")?;
        let lvr_cents = get_uint64_column(&batch, "total_lvr_cents")?;

        for i in 0..batch.num_rows() {
            // Early filter by markout time
            if markout_times.value(i) != markout_time {
                continue;
            }

            let cluster_name = cluster_names.value(i).to_string();
            let cluster_total = lvr_cents.value(i);
            
            // Track largest cluster
            if cluster_total > largest_cluster_amount {
                largest_cluster_amount = cluster_total;
                largest_cluster_name = cluster_name.clone();
            }

            total_lvr_cents = total_lvr_cents.saturating_add(cluster_total);

            clusters.push(ClusterTotal {
                name: cluster_name,
                total_lvr_cents: cluster_total,
            });
        }
    }

    if clusters.is_empty() {
        warn!(
            "No cluster distribution data found for markout time: {}", 
            markout_time
        );
        return Ok(Json(ClusterPieResponse {
            clusters: Vec::new(),
            total_lvr_cents: 0,
        }));
    }

    // Sort clusters by total for consistent presentation
    clusters.sort_by(|a, b| b.total_lvr_cents.cmp(&a.total_lvr_cents));

    let largest_proportion = if total_lvr_cents > 0 {
        (largest_cluster_amount as f64 / total_lvr_cents as f64) * 100.0
    } else {
        0.0
    };

    info!(
        "Analyzed {} clusters. Total volume: ${:.2}. Largest cluster: {} ({:.1}%)", 
        clusters.len(),
        total_lvr_cents as f64 / 100.0,
        largest_cluster_name,
        largest_proportion
    );

    Ok(Json(ClusterPieResponse {
        clusters,
        total_lvr_cents,
    }))
}

pub async fn get_cluster_histogram(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ClusterHistogramQuery>,
) -> Result<Json<ClusterHistogramResponse>, StatusCode> {
    let markout_time = params.markout_time.unwrap_or_else(|| String::from("brontes"));
    
    info!(
        "Analyzing transaction size distribution by cluster for markout time: {}", 
        markout_time
    );

    // Read from precomputed file
    let bytes = state.store.get(&Path::from("precomputed/clusters/histograms.parquet"))
        .await
        .map_err(|e| {
            error!("Failed to read precomputed cluster distribution data: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .bytes()
        .await
        .map_err(|e| {
            error!("Failed to get bytes from precomputed distribution data: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let reader = ParquetRecordBatchReader::try_new(bytes, 1024)
        .map_err(|e| {
            error!("Failed to create Parquet reader: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut cluster_data: HashMap<String, (Vec<ClusterHistogramBucket>, u64)> = HashMap::new();

    for batch_result in reader {
        let batch = batch_result.map_err(|e| {
            error!("Failed to read batch: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let cluster_names = get_string_column(&batch, "cluster_name")?;
        let markout_times = get_string_column(&batch, "markout_time")?;
        let bucket_starts = get_float64_column(&batch, "bucket_range_start")?;
        let bucket_ends = get_float64_column(&batch, "bucket_range_end")?;
        let counts = get_uint64_column(&batch, "count")?;
        let labels = get_string_column(&batch, "label")?;

        for i in 0..batch.num_rows() {
            // Early filter by markout time
            if markout_times.value(i) != markout_time {
                continue;
            }

            let cluster_name = cluster_names.value(i).to_string();
            let count = counts.value(i);

            let bucket = ClusterHistogramBucket {
                range_start: bucket_starts.value(i),
                range_end: if bucket_ends.is_null(i) { None } else { Some(bucket_ends.value(i)) },
                count,
                label: labels.value(i).to_string(),
            };

            cluster_data
                .entry(cluster_name)
                .and_modify(|(buckets, total)| {
                    buckets.push(bucket.clone());
                    *total = total.saturating_add(count);
                })
                .or_insert_with(|| (vec![bucket], count));
        }
    }

    if cluster_data.is_empty() {
        warn!(
            "No distribution data found for markout time: {}", 
            markout_time
        );
        return Ok(Json(ClusterHistogramResponse { clusters: Vec::new() }));
    }

    // Convert to response format and sort buckets
    let mut clusters: Vec<ClusterHistogramData> = cluster_data
        .into_iter()
        .map(|(name, (mut buckets, total_observations))| {
            // Sort buckets by range start for consistent presentation
            buckets.sort_by(|a, b| a.range_start.partial_cmp(&b.range_start)
                .unwrap_or(std::cmp::Ordering::Equal));
            ClusterHistogramData {
                name,
                buckets,
                total_observations,
            }
        })
        .collect();

    // Sort clusters by total observations for consistent ordering
    clusters.sort_by(|a, b| b.total_observations.cmp(&a.total_observations));

    info!(
        "Retrieved distribution data for {} clusters for markout time {}", 
        clusters.len(),
        markout_time
    );

    Ok(Json(ClusterHistogramResponse { clusters }))
}

pub async fn get_monthly_cluster_totals(
    State(state): State<Arc<AppState>>,
    Query(params): Query<MonthlyClusterQuery>,
) -> Result<Json<ClusterMonthlyResponse>, StatusCode> {
    let markout_time = params.markout_time.unwrap_or_else(|| String::from("brontes"));
    
    info!(
        "Analyzing monthly volume distribution across clusters for markout time: {}", 
        markout_time
    );

    // Read from precomputed file
    let bytes = state.store.get(&Path::from("precomputed/clusters/monthly_totals.parquet"))
        .await
        .map_err(|e| {
            error!("Failed to read precomputed monthly distribution data: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .bytes()
        .await
        .map_err(|e| {
            error!("Failed to get bytes from precomputed monthly data: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let reader = ParquetRecordBatchReader::try_new(bytes, 1024)
        .map_err(|e| {
            error!("Failed to create Parquet reader: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut time_range_data: HashMap<String, (HashMap<String, u64>, u64)> = HashMap::new();
    let mut unique_clusters = std::collections::HashSet::new();

    for batch_result in reader {
        let batch = batch_result.map_err(|e| {
            error!("Failed to read batch: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let time_ranges = get_string_column(&batch, "time_range")?;
        let cluster_names = get_string_column(&batch, "cluster_name")?;
        let markout_times = get_string_column(&batch, "markout_time")?;
        let total_lvr = get_uint64_column(&batch, "total_lvr_cents")?;

        for i in 0..batch.num_rows() {
            // Early filter by markout time
            if markout_times.value(i) != markout_time {
                continue;
            }

            let time_range = time_ranges.value(i).to_string();
            let cluster_name = cluster_names.value(i).to_string();
            let lvr_cents = total_lvr.value(i);

            unique_clusters.insert(cluster_name.clone());
            
            time_range_data
                .entry(time_range)
                .and_modify(|(totals, sum)| {
                    totals.insert(cluster_name.clone(), lvr_cents);
                    *sum = sum.saturating_add(lvr_cents);
                })
                .or_insert_with(|| {
                    let mut totals = HashMap::new();
                    totals.insert(cluster_name, lvr_cents);
                    (totals, lvr_cents)
                });
        }
    }

    if time_range_data.is_empty() {
        warn!(
            "No monthly distribution data found for markout time: {}", 
            markout_time
        );
        return Ok(Json(ClusterMonthlyResponse {
            monthly_data: Vec::new(),
            clusters: Vec::new(),
        }));
    }

    // Convert map data to chronologically sorted monthly results
    let mut monthly_result: Vec<MonthlyData> = time_range_data
        .into_iter()
        .map(|(time_range, (cluster_totals, total_lvr_cents))| MonthlyData {
            time_range,
            cluster_totals,
            total_lvr_cents,
        })
        .collect();

    monthly_result.sort_by_key(|data| {
        INTERVAL_RANGES
            .iter()
            .find(|(_, &range)| range == data.time_range)
            .map(|(block, _)| *block)
            .unwrap_or(0)
    });

    // Convert clusters to sorted Vec for consistent presentation
    let mut clusters: Vec<String> = unique_clusters.into_iter().collect();
    clusters.sort();

    info!(
        "Processed volume distribution across {} clusters over {} months", 
        clusters.len(),
        monthly_result.len()
    );

    Ok(Json(ClusterMonthlyResponse {
        monthly_data: monthly_result,
        clusters,
    }))
}

pub async fn get_cluster_non_zero(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ClusterNonZeroQuery>,
) -> Result<Json<ClusterNonZeroResponse>, StatusCode> {
    let markout_time = params.markout_time.unwrap_or_else(|| String::from("brontes"));
    
    info!(
        "Analyzing activity patterns across clusters for markout time: {}", 
        markout_time
    );

    // Read from precomputed file
    let bytes = state.store.get(&Path::from("precomputed/clusters/non_zero.parquet"))
        .await
        .map_err(|e| {
            error!("Failed to read precomputed cluster activity data: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .bytes()
        .await
        .map_err(|e| {
            error!("Failed to get bytes from precomputed activity data: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let reader = ParquetRecordBatchReader::try_new(bytes, 1024)
        .map_err(|e| {
            error!("Failed to create Parquet reader: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut clusters = Vec::new();

    for batch_result in reader {
        let batch = batch_result.map_err(|e| {
            error!("Failed to read batch: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let cluster_names = get_string_column(&batch, "cluster_name")?;
        let markout_times = get_string_column(&batch, "markout_time")?;
        let total_blocks = get_uint64_column(&batch, "total_blocks")?;
        let non_zero_blocks = get_uint64_column(&batch, "non_zero_blocks")?;
        let non_zero_proportions = get_float64_column(&batch, "non_zero_proportion")?;

        for i in 0..batch.num_rows() {
            // Early filter by markout time
            if markout_times.value(i) != markout_time {
                continue;
            }

            clusters.push(ClusterNonZero {
                name: cluster_names.value(i).to_string(),
                total_observations: total_blocks.value(i),
                non_zero_observations: non_zero_blocks.value(i),
                non_zero_proportion: non_zero_proportions.value(i),
            });
        }
    }

    if clusters.is_empty() {
        warn!(
            "No activity data found for markout time: {}", 
            markout_time
        );
        return Ok(Json(ClusterNonZeroResponse { clusters: Vec::new() }));
    }

    // Sort clusters by activity proportion for consistent presentation
    clusters.sort_by(|a, b| b.non_zero_proportion.partial_cmp(&a.non_zero_proportion)
        .unwrap_or(std::cmp::Ordering::Equal));

    info!(
        "Retrieved activity patterns for {} clusters with markout time {}", 
        clusters.len(), 
        markout_time
    );

    Ok(Json(ClusterNonZeroResponse { clusters }))
}