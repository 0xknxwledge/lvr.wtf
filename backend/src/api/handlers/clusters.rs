use axum::{
    extract::{State, Query},
    response::Json,
    http::StatusCode,
};
use std::{sync::Arc, collections::HashMap};
use tracing::{error, info, debug, warn};
use futures::StreamExt;
use parquet::arrow::arrow_reader::ParquetRecordBatchReader;
use crate::{
    AppState,
    api::handlers::common::{get_uint64_column, get_string_column},
    STABLE_POOLS, WBTC_WETH_POOLS, USDC_WETH_POOLS, USDT_WETH_POOLS, INTERVAL_RANGES,
    DAI_WETH_POOLS, USDC_WBTC_POOLS, ALTCOIN_WETH_POOLS,
    ClusterPieResponse, ClusterQuery, ClusterTotal,
    ClusterHistogramBucket, ClusterHistogramData, ClusterHistogramQuery, ClusterHistogramResponse,
    MonthlyClusterQuery, MonthlyData, ClusterMonthlyResponse
};


fn get_cluster_name(pool_address: &str) -> Option<&'static str> {
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
        "Fetching cluster proportion for markout_time: {}", 
        markout_time
    );

    let mut cluster_totals: HashMap<String, u64> = HashMap::new();
    let mut files_processed = 0;
    
    let checkpoints_path = object_store::path::Path::from("checkpoints");
    let mut checkpoint_files = state.store.list(Some(&checkpoints_path));
    
    while let Some(meta_result) = checkpoint_files.next().await {
        files_processed += 1;
        let meta = meta_result.map_err(|e| {
            error!("Failed to get file metadata: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        
        let file_path = meta.location.to_string();
        
        // Only process files for the specified markout time
        if !file_path.ends_with(&format!("_{}.parquet", markout_time)) {
            continue;
        }

        let bytes = state.store.get(&meta.location)
            .await
            .map_err(|e| {
                error!("Failed to read checkpoint file: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .bytes()
            .await
            .map_err(|e| {
                error!("Failed to get file bytes: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        let record_reader = ParquetRecordBatchReader::try_new(bytes, 1)
            .map_err(|e| {
                error!("Failed to create Parquet reader: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        for batch_result in record_reader {
            let batch = batch_result.map_err(|e| {
                error!("Failed to read batch: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            let pair_addresses = get_string_column(&batch, "pair_address")?;
            let running_totals = get_uint64_column(&batch, "running_total")?;

            // Process each row
            for i in 0..batch.num_rows() {
                let pool_address = pair_addresses.value(i);
                let running_total = running_totals.value(i);

                // Get the cluster name for this pool
                if let Some(cluster_name) = get_cluster_name(pool_address) {
                    cluster_totals
                        .entry(cluster_name.to_string())
                        .and_modify(|total| *total = total.saturating_add(running_total))
                        .or_insert(running_total);
                }
            }
        }
    }

    info!(
        "Processed {} checkpoint files, found {} clusters",
        files_processed,
        cluster_totals.len()
    );

    // Calculate total LVR across all clusters
    let total_lvr_cents: u64 = cluster_totals.values().sum();

    // Convert to response format with percentages
    let clusters: Vec<ClusterTotal> = cluster_totals
    .into_iter()
    .map(|(name, total_lvr_cents)| ClusterTotal {
        name,
        total_lvr_cents,
    })
    .collect();

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
        "Fetching cluster histogram data for markout_time: {}", 
        markout_time
    );

    let mut cluster_data: HashMap<String, Vec<u64>> = HashMap::new();
    let checkpoints_path = object_store::path::Path::from("checkpoints");
    let mut checkpoint_files = state.store.list(Some(&checkpoints_path));
    
    while let Some(meta_result) = checkpoint_files.next().await {
        let meta = meta_result.map_err(|e| {
            error!("Failed to get file metadata: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        
        let file_path = meta.location.to_string();
        if !file_path.ends_with(&format!("_{}.parquet", markout_time)) {
            continue;
        }

        let bytes = state.store.get(&meta.location)
            .await
            .map_err(|e| {
                error!("Failed to read checkpoint file: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .bytes()
            .await
            .map_err(|e| {
                error!("Failed to get file bytes: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        let record_reader = ParquetRecordBatchReader::try_new(bytes, 1)
            .map_err(|e| {
                error!("Failed to create Parquet reader: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        for batch_result in record_reader {
            let batch = batch_result.map_err(|e| {
                error!("Failed to read batch: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            let idx = batch.schema().index_of("pair_address").map_err(|e| {
                error!("Failed to find pair_address column: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
            let pair_addresses = batch.column(idx);
            let pair_addresses = pair_addresses
                .as_any()
                .downcast_ref::<arrow::array::StringArray>()
                .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

            // Get all bucket columns
            let bucket_names = [
                "total_bucket_0_10",
                "total_bucket_10_100",
                "total_bucket_100_500",
                "total_bucket_1000_3000",
                "total_bucket_3000_10000",
                "total_bucket_10000_30000",
                "total_bucket_30000_plus",
            ];

            let mut bucket_columns = Vec::new();
            for name in &bucket_names {
                let idx = batch.schema().index_of(name).map_err(|e| {
                    error!("Failed to find {} column: {}", name, e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
                let column = batch.column(idx);
                let column = column
                    .as_any()
                    .downcast_ref::<arrow::array::UInt64Array>()
                    .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
                bucket_columns.push(column);
            }

            // Process each row
            for row in 0..batch.num_rows() {
                let pool_address = pair_addresses.value(row);
                
                // Get cluster name
                if let Some(cluster_name) = get_cluster_name(&pool_address.to_lowercase()) {
                    let bucket_values: Vec<u64> = bucket_columns
                        .iter()
                        .map(|col| col.value(row))
                        .collect();

                    cluster_data
                        .entry(cluster_name.to_string())
                        .and_modify(|buckets| {
                            for (i, &value) in bucket_values.iter().enumerate() {
                                buckets[i] = buckets[i].saturating_add(value);
                            }
                        })
                        .or_insert_with(|| bucket_values);
                }
            }
        }
    }

    // Convert to response format
    let clusters: Vec<ClusterHistogramData> = cluster_data
        .into_iter()
        .map(|(name, bucket_counts)| {
            let buckets = vec![
                ClusterHistogramBucket {
                    range_start: 0.01,
                    range_end: Some(10.0),
                    count: bucket_counts[0],
                    label: "$0.01-$10".to_string(),
                },
                ClusterHistogramBucket {
                    range_start: 10.0,
                    range_end: Some(100.0),
                    count: bucket_counts[1],
                    label: "$10-$100".to_string(),
                },
                ClusterHistogramBucket {
                    range_start: 100.0,
                    range_end: Some(500.0),
                    count: bucket_counts[2],
                    label: "$100-$500".to_string(),
                },
                ClusterHistogramBucket {
                    range_start: 1000.0,
                    range_end: Some(3000.0),
                    count: bucket_counts[3],
                    label: "$1K-$3K".to_string(),
                },
                ClusterHistogramBucket {
                    range_start: 3000.0,
                    range_end: Some(10000.0),
                    count: bucket_counts[4],
                    label: "$3K-$10K".to_string(),
                },
                ClusterHistogramBucket {
                    range_start: 10000.0,
                    range_end: Some(30000.0),
                    count: bucket_counts[5],
                    label: "$10K-$30K".to_string(),
                },
                ClusterHistogramBucket {
                    range_start: 30000.0,
                    range_end: None,
                    count: bucket_counts[6],
                    label: "$30K+".to_string(),
                },
            ];

            let total_observations: u64 = buckets.iter().map(|b| b.count).sum();

            ClusterHistogramData {
                name,
                buckets,
                total_observations,
            }
        })
        .collect();

    info!(
        "Returning histogram data for {} clusters with markout time {}",
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
        "Fetching monthly cluster totals for markout_time: {}", 
        markout_time
    );

    let mut monthly_data: HashMap<u64, HashMap<String, u64>> = HashMap::new();
    let mut all_clusters = std::collections::HashSet::new();
    let mut files_processed = 0;
    
    let intervals_path = object_store::path::Path::from("intervals");
    let mut interval_files = state.store.list(Some(&intervals_path));
    
    while let Some(meta_result) = interval_files.next().await {
        files_processed += 1;
        let meta = meta_result.map_err(|e| {
            error!("Failed to get file metadata: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let file_path = meta.location.to_string();
        
        // Extract start block from file name
        let start_block = file_path
            .split('/')
            .last()
            .and_then(|name| name.split('_').next())
            .and_then(|num| num.parse::<u64>().ok())
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

        // Skip if we don't have a time range for this start block
        if !INTERVAL_RANGES.contains_key(&start_block) {
            continue;
        }

        debug!("Processing interval file: {} for block {}", file_path, start_block);

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
            let pair_addresses = get_string_column(&batch, "pair_address")?;
            let total_lvr_cents = get_uint64_column(&batch, "total_lvr_cents")?;
            let non_zero_counts = get_uint64_column(&batch, "non_zero_count")?;

            for i in 0..batch.num_rows() {
                if markout_times.value(i) != markout_time {
                    continue;
                }

                // Only process intervals that had activity
                if non_zero_counts.value(i) == 0 {
                    continue;
                }

                let pool_address = pair_addresses.value(i).to_lowercase();
                if let Some(cluster_name) = get_cluster_name(&pool_address) {
                    let lvr_cents = total_lvr_cents.value(i);
                    
                    monthly_data
                        .entry(start_block)
                        .or_default()
                        .entry(cluster_name.to_string())
                        .and_modify(|total| *total = total.saturating_add(lvr_cents))
                        .or_insert(lvr_cents);

                    all_clusters.insert(cluster_name.to_string());

                    debug!(
                        "Added {} cents to {} cluster for block {}", 
                        lvr_cents, cluster_name, start_block
                    );
                }
            }
        }
    }

    debug!(
        "Processed {} files, found {} unique clusters", 
        files_processed, 
        all_clusters.len()
    );

    // Convert to response format
    let mut clusters: Vec<String> = all_clusters.into_iter().collect();
    clusters.sort();

    let mut monthly_result: Vec<MonthlyData> = monthly_data
        .into_iter()
        .filter_map(|(start_block, cluster_totals)| {
            INTERVAL_RANGES.get(&start_block).map(|&time_range| {
                let total_lvr_cents = cluster_totals.values().sum();
                MonthlyData {
                    time_range: time_range.to_string(),
                    cluster_totals,
                    total_lvr_cents,
                }
            })
        })
        .collect();

    // Sort by start block (which gives us chronological order)
    monthly_result.sort_by_key(|data| {
        INTERVAL_RANGES
            .iter()
            .find(|(_, &range)| range == data.time_range)
            .map(|(block, _)| *block)
            .unwrap_or(0)
    });

    info!(
        "Returning monthly data for {} intervals across {} clusters", 
        monthly_result.len(),
        clusters.len()
    );

    if monthly_result.is_empty() {
        warn!(
            "No monthly data found for markout_time: {}. Processed {} files.", 
            markout_time,
            files_processed
        );
    }

    Ok(Json(ClusterMonthlyResponse {
        monthly_data: monthly_result,
        clusters,
    }))
}