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
        "Fetching precomputed cluster proportion data for markout_time: {}", 
        markout_time
    );

    // Read from precomputed file
    let precomputed_path = object_store::path::Path::from("precomputed/clusters/proportions.parquet");
    
    let bytes = state.store.get(&precomputed_path)
        .await
        .map_err(|e| {
            error!("Failed to read precomputed cluster proportion data: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .bytes()
        .await
        .map_err(|e| {
            error!("Failed to get bytes from precomputed cluster proportion data: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let reader = ParquetRecordBatchReader::try_new(bytes, 1024)
        .map_err(|e| {
            error!("Failed to create Parquet reader: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut clusters = Vec::new();
    let mut total_lvr_cents = 0u64;

    for batch_result in reader {
        let batch = batch_result.map_err(|e| {
            error!("Failed to read batch: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let cluster_names = get_string_column(&batch, "cluster_name")?;
        let markout_times = get_string_column(&batch, "markout_time")?;
        let lvr_cents = get_uint64_column(&batch, "total_lvr_cents")?;

        for i in 0..batch.num_rows() {
            // Filter by markout time
            if markout_times.value(i) != markout_time {
                continue;
            }

            let cluster_total = lvr_cents.value(i);
            total_lvr_cents = total_lvr_cents.saturating_add(cluster_total);

            clusters.push(ClusterTotal {
                name: cluster_names.value(i).to_string(),
                total_lvr_cents: cluster_total,
            });
        }
    }

    info!(
        "Found {} clusters with total LVR of {} cents for markout time {}", 
        clusters.len(),
        total_lvr_cents,
        markout_time
    );

    if clusters.is_empty() {
        warn!(
            "No cluster proportion data found for markout_time: {}", 
            markout_time
        );
    }

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
        "Fetching precomputed cluster histogram data for markout_time: {}", 
        markout_time
    );

    // Read from precomputed file
    let precomputed_path = object_store::path::Path::from("precomputed/clusters/histograms.parquet");
    
    let bytes = state.store.get(&precomputed_path)
        .await
        .map_err(|e| {
            error!("Failed to read precomputed cluster histogram data: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .bytes()
        .await
        .map_err(|e| {
            error!("Failed to get bytes from precomputed cluster histogram data: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let reader = ParquetRecordBatchReader::try_new(bytes, 1024)
        .map_err(|e| {
            error!("Failed to create Parquet reader: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Map to collect buckets for each cluster
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
            // Filter by markout time
            if markout_times.value(i) != markout_time {
                continue;
            }

            let cluster_name = cluster_names.value(i).to_string();
            let bucket = ClusterHistogramBucket {
                range_start: bucket_starts.value(i),
                range_end: if bucket_ends.is_null(i) { None } else { Some(bucket_ends.value(i)) },
                count: counts.value(i),
                label: labels.value(i).to_string(),
            };

            cluster_data
                .entry(cluster_name)
                .and_modify(|(buckets, total)| {
                    buckets.push(bucket.clone());
                    *total = total.saturating_add(bucket.count);
                })
                .or_insert_with(|| (vec![bucket], counts.value(i)));
        }
    }

    // Convert to response format
    let clusters: Vec<ClusterHistogramData> = cluster_data
        .into_iter()
        .map(|(name, (buckets, total_observations))| ClusterHistogramData {
            name,
            buckets,
            total_observations,
        })
        .collect();

    info!(
        "Returning histogram data for {} clusters with markout time {}",
        clusters.len(),
        markout_time
    );

    if clusters.is_empty() {
        warn!(
            "No cluster histogram data found for markout_time: {}", 
            markout_time
        );
    }

    Ok(Json(ClusterHistogramResponse { clusters }))
}

pub async fn get_monthly_cluster_totals(
    State(state): State<Arc<AppState>>,
    Query(params): Query<MonthlyClusterQuery>,
) -> Result<Json<ClusterMonthlyResponse>, StatusCode> {
    let markout_time = params.markout_time.unwrap_or_else(|| String::from("brontes"));
    
    info!(
        "Fetching precomputed monthly cluster totals for markout_time: {}", 
        markout_time
    );

    // Read from precomputed file
    let precomputed_path = object_store::path::Path::from("precomputed/clusters/monthly_totals.parquet");
    
    let bytes = state.store.get(&precomputed_path)
        .await
        .map_err(|e| {
            error!("Failed to read precomputed monthly cluster totals: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .bytes()
        .await
        .map_err(|e| {
            error!("Failed to get bytes from precomputed monthly cluster totals: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let reader = ParquetRecordBatchReader::try_new(bytes, 1024)
        .map_err(|e| {
            error!("Failed to create Parquet reader: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut monthly_result = Vec::new();
    let mut all_clusters = std::collections::HashSet::new();

    for batch_result in reader {
        let batch = batch_result.map_err(|e| {
            error!("Failed to read batch: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let time_ranges = get_string_column(&batch, "time_range")?;
        let cluster_names = get_string_column(&batch, "cluster_name")?;
        let markout_times = get_string_column(&batch, "markout_time")?;
        let total_lvr = get_uint64_column(&batch, "total_lvr_cents")?;

        // Keep track of totals for each time range
        let mut time_range_data: HashMap<String, (HashMap<String, u64>, u64)> = HashMap::new();

        for i in 0..batch.num_rows() {
            // Filter by markout time
            if markout_times.value(i) != markout_time {
                continue;
            }

            let time_range = time_ranges.value(i).to_string();
            let cluster_name = cluster_names.value(i).to_string();
            let lvr_cents = total_lvr.value(i);

            time_range_data
                .entry(time_range)
                .and_modify(|(totals, sum)| {
                    totals.insert(cluster_name.clone(), lvr_cents);
                    *sum = sum.saturating_add(lvr_cents);
                })
                .or_insert_with(|| {
                    let mut totals = HashMap::new();
                    totals.insert(cluster_name.clone(), lvr_cents);
                    (totals, lvr_cents)
                });

            all_clusters.insert(cluster_name);
        }

        // Convert time range data to MonthlyData
        for (time_range, (cluster_totals, total_lvr_cents)) in time_range_data {
            monthly_result.push(MonthlyData {
                time_range,
                cluster_totals,
                total_lvr_cents,
            });
        }
    }

    // Sort monthly data chronologically by time range
    monthly_result.sort_by_key(|data| {
        INTERVAL_RANGES
            .iter()
            .find(|(_, &range)| range == data.time_range)
            .map(|(block, _)| *block)
            .unwrap_or(0)
    });

    // Convert all_clusters to sorted Vec
    let mut clusters: Vec<String> = all_clusters.into_iter().collect();
    clusters.sort();

    info!(
        "Returning monthly data for {} intervals across {} clusters", 
        monthly_result.len(),
        clusters.len()
    );

    if monthly_result.is_empty() {
        warn!(
            "No monthly cluster data found for markout_time: {}", 
            markout_time
        );
    }

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
        "Fetching precomputed cluster non-zero proportions for markout_time: {}", 
        markout_time
    );

    // Read from precomputed file
    let precomputed_path = object_store::path::Path::from("precomputed/clusters/non_zero.parquet");
    
    let bytes = state.store.get(&precomputed_path)
        .await
        .map_err(|e| {
            error!("Failed to read precomputed cluster non-zero data: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .bytes()
        .await
        .map_err(|e| {
            error!("Failed to get bytes from precomputed cluster non-zero data: {}", e);
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
        let total_observations = get_uint64_column(&batch, "total_observations")?;
        let non_zero_observations = get_uint64_column(&batch, "non_zero_observations")?;
        let non_zero_proportions = get_float64_column(&batch, "non_zero_proportion")?;

        for i in 0..batch.num_rows() {
            // Filter by markout time
            if markout_times.value(i) != markout_time {
                continue;
            }

            clusters.push(ClusterNonZero {
                name: cluster_names.value(i).to_string(),
                total_observations: total_observations.value(i),
                non_zero_observations: non_zero_observations.value(i),
                non_zero_proportion: non_zero_proportions.value(i),
            });
        }
    }

    info!(
        "Returning non-zero proportions for {} clusters with markout time {}", 
        clusters.len(), 
        markout_time
    );

    if clusters.is_empty() {
        warn!(
            "No cluster non-zero proportion data found for markout_time: {}", 
            markout_time
        );
    }

    Ok(Json(ClusterNonZeroResponse { clusters }))
}