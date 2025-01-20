use arrow::{
    array::{StringArray, UInt64Array, Float64Array, Int64Array},
    record_batch::RecordBatch,
    datatypes::DataType
};
use object_store::{path::Path, ObjectStore};
use parquet::{
    arrow::{ArrowWriter, arrow_reader::ParquetRecordBatchReader},
    basic::Compression,
    file::properties::WriterProperties,
};
use std::sync::Arc;
use anyhow::Context;
use std::collections::HashMap;
use bytes::Bytes;
use tracing::{info, warn, debug};
use futures::StreamExt;
use crate::{
    api::types::*,
    api::handlers::*,
    MERGE_BLOCK, POOL_NAMES, INTERVAL_RANGES,
    common::{BLOCKS_PER_INTERVAL, FINAL_INTERVAL_FILE, FINAL_PARTIAL_BLOCKS, 
        get_string_column, get_uint64_column, get_valid_pools, get_column_value, get_pool_name, calculate_percentile}
};
use arrow::array::Array;


pub struct PrecomputedWriter {
    object_store: Arc<dyn ObjectStore>,
    max_retries: u32,
}

impl PrecomputedWriter {
    pub fn new(object_store: Arc<dyn ObjectStore>) -> Self {
        Self {
            object_store,
            max_retries: 3,
        }
    }

    async fn write_batch_to_store(
        &self,
        path: Path,
        batch: RecordBatch,
    ) -> Result<(), anyhow::Error> {
        let props = WriterProperties::builder()
            .set_compression(Compression::SNAPPY)
            .set_write_batch_size(1024 * 1024)
            .build();

        let mut buffer = Vec::new();
        {
            let mut writer = ArrowWriter::try_new(&mut buffer, batch.schema(), Some(props))?;
            writer.write(&batch)?;
            writer.close()?;
        }

        let mut retries = 0;
        while retries < self.max_retries {
            match self.object_store.put(&path, Bytes::from(buffer.clone()).into()).await {
                Ok(_) => return Ok(()),
                Err(e) if retries < self.max_retries - 1 => {
                    retries += 1;
                    let delay = std::time::Duration::from_secs(2u64.pow(retries));
                    warn!(
                        "Write attempt {} failed for path {}: {}. Retrying in {} seconds...",
                        retries, path, e, delay.as_secs()
                    );
                    tokio::time::sleep(delay).await;
                }
                Err(e) => return Err(e.into()),
            }
        }
        
        Err(anyhow::anyhow!("Failed to write after {} retries", self.max_retries))
    }

    pub async fn write_running_totals(&self) -> Result<(), anyhow::Error> {
        info!("Starting precomputation of running totals (individual and aggregate)");
        
        // Get all data from interval files
        let intervals_path = object_store::path::Path::from("intervals");
        let mut interval_files = self.object_store.list(Some(&intervals_path));
        let valid_pools = get_valid_pools();
            
        let mut interval_data: HashMap<(u64, String, String), u64> = HashMap::new();
        let mut aggregate_data: HashMap<(u64, String), u64> = HashMap::new();
    
        // Process all interval files to collect interval data
        while let Some(meta_result) = interval_files.next().await {
            let meta = meta_result.context("Failed to get file metadata")?;
            let file_path = meta.location.to_string();
            
            // Get file start block from path
            let file_start = file_path
                .split("intervals/")
                .nth(1)
                .and_then(|name| name.trim_end_matches(".parquet").split('_').next())
                .and_then(|num| num.parse::<u64>().ok())
                .unwrap_or(*MERGE_BLOCK);
    
            let bytes = self.object_store.get(&meta.location)
                .await?
                .bytes()
                .await?;
    
            let record_reader = ParquetRecordBatchReader::try_new(bytes, 1024)?;
    
            for batch_result in record_reader {
                let batch = batch_result?;
                
                let interval_ids = get_uint64_column(&batch, "interval_id")
                .map_err(|e| anyhow::anyhow!("Failed to get interval_id column: {}", e))?;
                let markout_times_col = get_string_column(&batch, "markout_time")
                    .map_err(|e| anyhow::anyhow!("Failed to get markout_time column: {}", e))?;
                let pool_addresses_col = get_string_column(&batch, "pair_address")
                    .map_err(|e| anyhow::anyhow!("Failed to get pair_address column: {}", e))?;
                let total_lvr_cents = get_uint64_column(&batch, "total_lvr_cents")
                    .map_err(|e| anyhow::anyhow!("Failed to get total_lvr_cents column: {}", e))?;
                let non_zero_counts = get_uint64_column(&batch, "non_zero_count")
                    .map_err(|e| anyhow::anyhow!("Failed to get non_zero_count column: {}", e))?;
    
                for i in 0..batch.num_rows() {
                    if total_lvr_cents.is_null(i) || non_zero_counts.value(i) == 0 {
                        continue;
                    }
    
                    let pool_address = pool_addresses_col.value(i).to_lowercase();
                    if !valid_pools.contains(&pool_address) {
                        continue;
                    }
    
                    let interval_id = interval_ids.value(i);
                    let markout_time = markout_times_col.value(i).to_string();
                    let lvr_cents = total_lvr_cents.value(i);
    
                    let block_number = if file_path.ends_with(FINAL_INTERVAL_FILE) && interval_id == 19 {
                        file_start + (interval_id * BLOCKS_PER_INTERVAL) + FINAL_PARTIAL_BLOCKS
                    } else {
                        file_start + (interval_id * BLOCKS_PER_INTERVAL)
                    };
    
                    // Update individual pool data
                    interval_data
                        .entry((block_number, markout_time.clone(), pool_address.clone()))
                        .and_modify(|total| *total = total.saturating_add(lvr_cents))
                        .or_insert(lvr_cents);
    
                    // Update aggregate data
                    aggregate_data
                        .entry((block_number, markout_time.clone()))
                        .and_modify(|total| *total = total.saturating_add(lvr_cents))
                        .or_insert(lvr_cents);
                }
            }
        }
    
        // Write individual running totals
        self.write_individual_running_totals(interval_data).await?;
    
        // Write aggregate running totals
        self.write_aggregate_running_totals(aggregate_data).await?;
    
        info!("Successfully wrote precomputed running totals (individual and aggregate)");
        Ok(())
    }
    
    async fn write_individual_running_totals(
        &self,
        interval_data: HashMap<(u64, String, String), u64>
    ) -> Result<(), anyhow::Error> {
        // Create output schema for individual running totals
        let schema = arrow::datatypes::Schema::new(vec![
            arrow::datatypes::Field::new("block_number", arrow::datatypes::DataType::UInt64, false),
            arrow::datatypes::Field::new("markout_time", arrow::datatypes::DataType::Utf8, false),
            arrow::datatypes::Field::new("pool_address", arrow::datatypes::DataType::Utf8, false),
            arrow::datatypes::Field::new("running_total_cents", arrow::datatypes::DataType::UInt64, false),
        ]);
    
        // Track running totals per pool/markout combination
        let mut running_totals: HashMap<(String, String), u64> = HashMap::new();
    
        // Sort data points by block number
        let mut data_points: Vec<_> = interval_data.into_iter().collect();
        data_points.sort_by_key(|((block, _, _), _)| *block);
    
        let mut block_numbers = Vec::new();
        let mut markout_times = Vec::new();
        let mut pool_addresses = Vec::new();
        let mut totals = Vec::new();
    
        // Process sorted data points to maintain monotonicity
        for ((block_number, markout_time, pool_address), interval_total) in data_points {
            let current_total = running_totals
                .entry((pool_address.clone(), markout_time.clone()))
                .and_modify(|total| *total = total.saturating_add(interval_total))
                .or_insert(interval_total);
    
            block_numbers.push(block_number);
            markout_times.push(markout_time);
            pool_addresses.push(pool_address);
            totals.push(*current_total);
        }
    
        // Create and write record batch
        let batch = RecordBatch::try_new(
            Arc::new(schema),
            vec![
                Arc::new(UInt64Array::from(block_numbers)),
                Arc::new(StringArray::from(markout_times)),
                Arc::new(StringArray::from(pool_addresses)),
                Arc::new(UInt64Array::from(totals)),
            ],
        )?;
    
        let output_path = Path::from("precomputed/running_totals/individual.parquet");
        self.write_batch_to_store(output_path, batch).await?;
    
        info!("Successfully wrote precomputed individual running totals");
        Ok(())
    }
    
    async fn write_aggregate_running_totals(
        &self,
        aggregate_data: HashMap<(u64, String), u64>
    ) -> Result<(), anyhow::Error> {
        // Create output schema for aggregate running totals
        let schema = arrow::datatypes::Schema::new(vec![
            arrow::datatypes::Field::new("block_number", arrow::datatypes::DataType::UInt64, false),
            arrow::datatypes::Field::new("markout_time", arrow::datatypes::DataType::Utf8, false),
            arrow::datatypes::Field::new("running_total_cents", arrow::datatypes::DataType::UInt64, false),
        ]);
    
        // Track running totals per markout time
        let mut running_totals: HashMap<String, u64> = HashMap::new();
    
        // Sort data points by block number
        let mut data_points: Vec<_> = aggregate_data.into_iter().collect();
        data_points.sort_by_key(|((block, _), _)| *block);
    
        let mut block_numbers = Vec::new();
        let mut markout_times = Vec::new();
        let mut totals = Vec::new();
    
        // Process sorted data points to maintain monotonicity
        for ((block_number, markout_time), interval_total) in data_points {
            let current_total = running_totals
                .entry(markout_time.clone())
                .and_modify(|total| *total = total.saturating_add(interval_total))
                .or_insert(interval_total);
    
            block_numbers.push(block_number);
            markout_times.push(markout_time);
            totals.push(*current_total);
        }
    
        // Create and write record batch
        let batch = RecordBatch::try_new(
            Arc::new(schema),
            vec![
                Arc::new(UInt64Array::from(block_numbers)),
                Arc::new(StringArray::from(markout_times)),
                Arc::new(UInt64Array::from(totals)),
            ],
        )?;
    
        let output_path = Path::from("precomputed/running_totals/aggregate.parquet");
        self.write_batch_to_store(output_path, batch).await?;
    
        info!("Successfully wrote precomputed aggregate running totals");
        Ok(())
    }

    pub async fn write_lvr_ratios(&self) -> Result<(), anyhow::Error> {
        info!("Starting precomputation of LVR ratios");
        
        // Create schema for LVR ratios
        let schema = arrow::datatypes::Schema::new(vec![
            arrow::datatypes::Field::new("markout_time", arrow::datatypes::DataType::Utf8, false),
            arrow::datatypes::Field::new("ratio", arrow::datatypes::DataType::Float64, false),
            arrow::datatypes::Field::new("realized_lvr_cents", arrow::datatypes::DataType::UInt64, false),
            arrow::datatypes::Field::new("theoretical_lvr_cents", arrow::datatypes::DataType::UInt64, false),
        ]);

        // Initialize totals structure
        let mut totals = LVRTotals {
            realized: 0,
            theoretical: HashMap::new(),
        };

        // Process all interval files
        let intervals_path = object_store::path::Path::from("intervals");
        let mut interval_files = self.object_store.list(Some(&intervals_path));
        let valid_pools = get_valid_pools();

        while let Some(meta_result) = interval_files.next().await {
            let meta = meta_result.context("Failed to get file metadata")?;
            let bytes = self.object_store.get(&meta.location)
                .await?
                .bytes()
                .await?;

            let record_reader = ParquetRecordBatchReader::try_new(bytes, 1024)?;

            for batch_result in record_reader {
                let batch = batch_result?;
                
                let markout_times = get_string_column(&batch, "markout_time")
                    .map_err(|e| anyhow::anyhow!("Failed to get markout_time column: {}", e))?;
                let pool_addresses = get_string_column(&batch, "pair_address")
                    .map_err(|e| anyhow::anyhow!("Failed to get pair_address column: {}", e))?;
                let total_lvr_cents = get_uint64_column(&batch, "total_lvr_cents")
                    .map_err(|e| anyhow::anyhow!("Failed to get total_lvr_cents column: {}", e))?;
                let non_zero_counts = get_uint64_column(&batch, "non_zero_count")
                    .map_err(|e| anyhow::anyhow!("Failed to get non_zero_count column: {}", e))?;

                for i in 0..batch.num_rows() {
                    if total_lvr_cents.is_null(i) || non_zero_counts.value(i) == 0 {
                        continue;
                    }

                    let pool_address = pool_addresses.value(i).to_lowercase();
                    if !valid_pools.contains(&pool_address) {
                        continue;
                    }

                    let markout_time = markout_times.value(i);
                    let lvr_cents = total_lvr_cents.value(i);

                    if lvr_cents > 0 {
                        if markout_time == "brontes" {
                            totals.realized = totals.realized.saturating_add(lvr_cents);
                        } else {
                            totals.theoretical
                                .entry(markout_time.to_string())
                                .and_modify(|e| *e = e.saturating_add(lvr_cents))
                                .or_insert(lvr_cents);
                        }
                    }
                }
            }
        }

        // Calculate ratios
        let ratios = ratios::calculate_lvr_ratios(totals);

        // Prepare arrays for the record batch
        let markout_times: Vec<String> = ratios.iter().map(|r| r.markout_time.clone()).collect();
        let ratio_values: Vec<f64> = ratios.iter().map(|r| r.ratio).collect();
        let realized_cents: Vec<u64> = ratios.iter().map(|r| r.realized_lvr_cents).collect();
        let theoretical_cents: Vec<u64> = ratios.iter().map(|r| r.theoretical_lvr_cents).collect();

        // Create record batch
        let batch = RecordBatch::try_new(
            Arc::new(schema),
            vec![
                Arc::new(StringArray::from(markout_times)),
                Arc::new(Float64Array::from(ratio_values)),
                Arc::new(UInt64Array::from(realized_cents)),
                Arc::new(UInt64Array::from(theoretical_cents)),
            ],
        )?;

        // Write to output file
        let output_path = Path::from("precomputed/ratios/lvr_ratios.parquet");
        self.write_batch_to_store(output_path, batch).await?;

        info!("Successfully wrote precomputed LVR ratios");
        Ok(())
    }

    pub async fn write_pool_totals(&self) -> Result<(), anyhow::Error> {
        info!("Starting precomputation of pool totals");
        
        // Create schema for pool totals
        let schema = arrow::datatypes::Schema::new(vec![
            arrow::datatypes::Field::new("pool_address", arrow::datatypes::DataType::Utf8, false),
            arrow::datatypes::Field::new("pool_name", arrow::datatypes::DataType::Utf8, false),
            arrow::datatypes::Field::new("markout_time", arrow::datatypes::DataType::Utf8, false),
            arrow::datatypes::Field::new("total_lvr_cents", arrow::datatypes::DataType::UInt64, false),
            arrow::datatypes::Field::new("non_zero_blocks", arrow::datatypes::DataType::UInt64, false),
            arrow::datatypes::Field::new("total_blocks", arrow::datatypes::DataType::UInt64, false),
        ]);

        // Prepare vectors for collecting data
        let mut pool_addresses = Vec::new();
        let mut pool_names = Vec::new();
        let mut markout_times = Vec::new();
        let mut total_lvr_cents = Vec::new();
        let mut non_zero_blocks = Vec::new();
        let mut total_blocks = Vec::new();

        let valid_pools = get_valid_pools();
        let checkpoints_path = object_store::path::Path::from("checkpoints");
        let mut checkpoint_files = self.object_store.list(Some(&checkpoints_path));
        
        while let Some(meta_result) = checkpoint_files.next().await {
            let meta = meta_result.context("Failed to get file metadata")?;
            let file_path = meta.location.to_string();
            
            let bytes = self.object_store.get(&meta.location)
                .await?
                .bytes()
                .await?;

            let record_reader = ParquetRecordBatchReader::try_new(bytes, 1)?;

            for batch_result in record_reader {
                let batch = batch_result?;

                // Get running total with dynamic type handling
                let running_total_idx = batch.schema().index_of("running_total")?;
                let running_total = {
                    let column = batch.column(running_total_idx);
                    match column.data_type() {
                        DataType::Int64 => {
                            column.as_any()
                                .downcast_ref::<Int64Array>()
                                .map(|arr| arr.value(0))
                                .context("Failed to cast running_total as Int64Array")?
                        },
                        DataType::UInt64 => {
                            column.as_any()
                                .downcast_ref::<UInt64Array>()
                                .map(|arr| arr.value(0) as i64)
                                .context("Failed to cast running_total as UInt64Array")?
                        },
                        other => return Err(anyhow::anyhow!("Unexpected type for running_total: {:?}", other))
                    }
                };

                let pair_addresses = get_string_column(&batch, "pair_address")
                    .map_err(|e| anyhow::anyhow!("Failed to get pair_address column: {}", e))?;
                    
                // Get additional metrics
                let total_bucket_0 = get_uint64_column(&batch, "total_bucket_0")
                    .map_err(|e| anyhow::anyhow!("Failed to get total_bucket_0 column: {}", e))?;
                
                let non_zero_buckets = [
                    "total_bucket_0_10",
                    "total_bucket_10_100",
                    "total_bucket_100_500",
                    "total_bucket_500_1000",
                    "total_bucket_1000_10000",
                    "total_bucket_10000_plus",
                ];

                if batch.num_rows() > 0 {
                    let pair_address = pair_addresses.value(0).to_lowercase();
                    if !valid_pools.contains(&pair_address) {
                        continue;
                    }

                    // Calculate non_zero and total blocks
                    let mut non_zero_count = 0u64;
                    for bucket_name in &non_zero_buckets {
                        let bucket = get_uint64_column(&batch, bucket_name)
                            .map_err(|e| anyhow::anyhow!("Failed to get {} column: {}", bucket_name, e))?;
                        non_zero_count += bucket.value(0);
                    }

                    let zero_count = total_bucket_0.value(0);
                    let total_count = zero_count + non_zero_count;

                    if total_count > 0 {
                        // Extract markout time from file path
                        let markout_time = file_path
                            .split('_')
                            .last()
                            .and_then(|s| s.strip_suffix(".parquet"))
                            .context("Failed to extract markout time from file path")?;

                        let pool_name = POOL_NAMES
                            .iter()
                            .find(|(addr, _)| addr.to_lowercase() == pair_address)
                            .map(|(_, name)| name.to_string())
                            .unwrap_or_else(|| pair_address.clone());

                        pool_addresses.push(pair_address);
                        pool_names.push(pool_name);
                        markout_times.push(markout_time.to_string());
                        total_lvr_cents.push(running_total.unsigned_abs());
                        non_zero_blocks.push(non_zero_count);
                        total_blocks.push(total_count);
                    }
                }
            }
        }

        // Create record batch
        let batch = RecordBatch::try_new(
            Arc::new(schema),
            vec![
                Arc::new(StringArray::from(pool_addresses)),
                Arc::new(StringArray::from(pool_names)),
                Arc::new(StringArray::from(markout_times)),
                Arc::new(UInt64Array::from(total_lvr_cents)),
                Arc::new(UInt64Array::from(non_zero_blocks)),
                Arc::new(UInt64Array::from(total_blocks)),
            ],
        )?;

        // Write to output file
        let output_path = Path::from("precomputed/pool_metrics/totals.parquet");
        self.write_batch_to_store(output_path, batch).await?;

        info!("Successfully wrote precomputed pool totals");
        Ok(())
    }

    pub async fn write_max_lvr(&self) -> Result<(), anyhow::Error> {
        info!("Starting precomputation of max LVR values");
        
        let schema = arrow::datatypes::Schema::new(vec![
            arrow::datatypes::Field::new("pool_address", arrow::datatypes::DataType::Utf8, false),
            arrow::datatypes::Field::new("pool_name", arrow::datatypes::DataType::Utf8, false),
            arrow::datatypes::Field::new("markout_time", arrow::datatypes::DataType::Utf8, false),
            arrow::datatypes::Field::new("block_number", arrow::datatypes::DataType::UInt64, false),
            arrow::datatypes::Field::new("max_lvr_cents", arrow::datatypes::DataType::UInt64, false),
        ]);
    
        let mut pool_addresses = Vec::new();
        let mut pool_names = Vec::new();
        let mut markout_times = Vec::new();
        let mut block_numbers = Vec::new();
        let mut max_lvr_cents = Vec::new();
    
        let valid_pools = get_valid_pools();
        let mut theoretical_maximums: HashMap<String, HashMap<String, u64>> = HashMap::new();
    
        // First, get theoretical maximums for brontes validation
        for pool_address in &valid_pools {
            let mut pool_maximums = HashMap::new();
            let checkpoints_path = object_store::path::Path::from("checkpoints");
            let mut checkpoint_files = self.object_store.list(Some(&checkpoints_path));
    
            while let Some(meta_result) = checkpoint_files.next().await {
                let meta = meta_result.context("Failed to get file metadata")?;
                let file_path = meta.location.to_string();
    
                if !file_path.to_lowercase().contains(&pool_address.to_lowercase()) 
                   || file_path.to_lowercase().ends_with("_brontes.parquet") {
                    continue;
                }
    
                let bytes = self.object_store.get(&meta.location).await?.bytes().await?;
                let record_reader = ParquetRecordBatchReader::try_new(bytes, 1)?;
    
                for batch_result in record_reader {
                    let batch = batch_result?;
                    let value = get_column_value::<UInt64Array>(&batch, "max_lvr_value")
                        .map_err(|e| anyhow::anyhow!("Failed to get max_lvr_value: {}", e))?;
                    
                    if value > 0 {
                        let markout = file_path
                            .split('_')
                            .last()
                            .and_then(|s| s.strip_suffix(".parquet"))
                            .context("Failed to extract markout time")?;
                        
                        pool_maximums.insert(markout.to_string(), value);
                    }
                }
            }
    
            if !pool_maximums.is_empty() {
                theoretical_maximums.insert(pool_address.to_string(), pool_maximums);
            }
        }
    
        // Process regular markout times
        let checkpoints_path = object_store::path::Path::from("checkpoints");
        let mut checkpoint_files = self.object_store.list(Some(&checkpoints_path));
    
        while let Some(meta_result) = checkpoint_files.next().await {
            let meta = meta_result.context("Failed to get file metadata")?;
            let file_path = meta.location.to_string();
            
            let pool_address = file_path
                .split('/')
                .last()
                .and_then(|s| s.split('_').next())
                .context("Failed to extract pool address")?
                .to_lowercase();
    
            if !valid_pools.contains(&pool_address) {
                continue;
            }
    
            let markout_time = file_path
                .split('_')
                .last()
                .and_then(|s| s.strip_suffix(".parquet"))
                .context("Failed to extract markout time")?;
    
            let bytes = self.object_store.get(&meta.location).await?.bytes().await?;
            let record_reader = ParquetRecordBatchReader::try_new(bytes, 1)?;
    
            for batch_result in record_reader {
                let batch = batch_result?;
                let value = get_column_value::<UInt64Array>(&batch, "max_lvr_value")
                    .map_err(|e| anyhow::anyhow!("Failed to get max_lvr_value: {}", e))?;
                let block = get_column_value::<UInt64Array>(&batch, "max_lvr_block")
                    .map_err(|e| anyhow::anyhow!("Failed to get max_lvr_block: {}", e))?;
    
                if value > 0 {
                    // For brontes, validate against theoretical maximums
                    if markout_time == "brontes" {
                        if let Some(pool_maxes) = theoretical_maximums.get(&pool_address) {
                            let min_theoretical_max = pool_maxes.values().min()
                                .context("No theoretical maximum found")?;
                            
                            if value > *min_theoretical_max {
                                // Search through intervals for valid maximum
                                if let Some((valid_block, valid_value)) = max::find_valid_max_from_intervals(
                                    &self.object_store,
                                    &pool_address,
                                    *min_theoretical_max
                                ).await.map_err(|e| anyhow::anyhow!("Error finding valid max: {}", e))? {
                                    let pool_name = get_pool_name(&pool_address);
                                    pool_addresses.push(pool_address.clone());
                                    pool_names.push(pool_name);
                                    markout_times.push(markout_time.to_string());
                                    block_numbers.push(valid_block);
                                    max_lvr_cents.push(valid_value);
                                }
                                continue;
                            }
                        }
                    }
    
                    let pool_name = get_pool_name(&pool_address);
                    pool_addresses.push(pool_address.clone());
                    pool_names.push(pool_name);
                    markout_times.push(markout_time.to_string());
                    block_numbers.push(block);
                    max_lvr_cents.push(value);
                }
            }
        }
    
        // Create record batch
        let batch = RecordBatch::try_new(
            Arc::new(schema),
            vec![
                Arc::new(StringArray::from(pool_addresses)),
                Arc::new(StringArray::from(pool_names)),
                Arc::new(StringArray::from(markout_times)),
                Arc::new(UInt64Array::from(block_numbers)),
                Arc::new(UInt64Array::from(max_lvr_cents)),
            ],
        )?;
    
        // Write to output file
        let output_path = Path::from("precomputed/pool_metrics/max_lvr.parquet");
        self.write_batch_to_store(output_path, batch).await?;
    
        info!("Successfully wrote precomputed max LVR values");
        Ok(())
    }

    pub async fn write_non_zero_proportions(&self) -> Result<(), anyhow::Error> {
        info!("Starting precomputation of non-zero proportions");
        
        let schema = arrow::datatypes::Schema::new(vec![
            arrow::datatypes::Field::new("pool_address", arrow::datatypes::DataType::Utf8, false),
            arrow::datatypes::Field::new("pool_name", arrow::datatypes::DataType::Utf8, false),
            arrow::datatypes::Field::new("markout_time", arrow::datatypes::DataType::Utf8, false),
            arrow::datatypes::Field::new("non_zero_blocks", arrow::datatypes::DataType::UInt64, false),
            arrow::datatypes::Field::new("total_blocks", arrow::datatypes::DataType::UInt64, false),
            arrow::datatypes::Field::new("non_zero_proportion", arrow::datatypes::DataType::Float64, false),
        ]);

        let mut pool_addresses = Vec::new();
        let mut pool_names = Vec::new();
        let mut markout_times = Vec::new();
        let mut non_zero_blocks_vec = Vec::new();
        let mut total_blocks_vec = Vec::new();
        let mut proportions = Vec::new();

        let valid_pools = get_valid_pools();
        let checkpoints_path = object_store::path::Path::from("checkpoints");
        let mut checkpoint_files = self.object_store.list(Some(&checkpoints_path));

        // Process all checkpoint files
        while let Some(meta_result) = checkpoint_files.next().await {
            let meta = meta_result.context("Failed to get file metadata")?;
            let file_path = meta.location.to_string();

            let bytes = self.object_store.get(&meta.location).await?.bytes().await?;
            let record_reader = ParquetRecordBatchReader::try_new(bytes, 1)?;

            for batch_result in record_reader {
                let batch = batch_result?;

                if batch.num_rows() == 0 {
                    continue;
                }

                // Get pool address and validate
                let pair_addresses = get_string_column(&batch, "pair_address")
                    .map_err(|e| anyhow::anyhow!("Failed to get pair_address column: {}", e))?;
                let pool_address = pair_addresses.value(0).to_lowercase();
                
                if !valid_pools.contains(&pool_address) {
                    continue;
                }

                // Calculate total blocks from buckets
                let zero_bucket = get_uint64_column(&batch, "total_bucket_0")
                    .map_err(|e| anyhow::anyhow!("Failed to get total_bucket_0 column: {}", e))?;
                
                let non_zero_buckets = [
                    "total_bucket_0_10",
                    "total_bucket_10_100",
                    "total_bucket_100_500",
                    "total_bucket_500_1000",
                    "total_bucket_1000_10000",
                    "total_bucket_10000_plus",
                ];

                let mut non_zero_count = 0u64;
                for bucket_name in &non_zero_buckets {
                    let bucket = get_uint64_column(&batch, bucket_name)
                        .map_err(|e| anyhow::anyhow!("Failed to get {} column: {}", bucket_name, e))?;
                    non_zero_count += bucket.value(0);
                }

                let zero_count = zero_bucket.value(0);
                let total_count = zero_count + non_zero_count;

                if total_count > 0 {
                    // Extract markout time from file path
                    let markout_time = file_path
                        .split('_')
                        .last()
                        .and_then(|s| s.strip_suffix(".parquet"))
                        .context("Failed to extract markout time")?;

                    let proportion = if total_count > 0 {
                        non_zero_count as f64 / total_count as f64
                    } else {
                        0.0
                    };

                    let pool_name = POOL_NAMES
                        .iter()
                        .find(|(addr, _)| addr.to_lowercase() == pool_address)
                        .map(|(_, name)| name.to_string())
                        .unwrap_or_else(|| pool_address.clone());

                    pool_addresses.push(pool_address);
                    pool_names.push(pool_name);
                    markout_times.push(markout_time.to_string());
                    non_zero_blocks_vec.push(non_zero_count);
                    total_blocks_vec.push(total_count);
                    proportions.push(proportion);
                }
            }
        }

        // Create record batch
        let batch = RecordBatch::try_new(
            Arc::new(schema),
            vec![
                Arc::new(StringArray::from(pool_addresses)),
                Arc::new(StringArray::from(pool_names)),
                Arc::new(StringArray::from(markout_times)),
                Arc::new(UInt64Array::from(non_zero_blocks_vec)),
                Arc::new(UInt64Array::from(total_blocks_vec)),
                Arc::new(Float64Array::from(proportions)),
            ],
        )?;

        // Write to output file
        let output_path = Path::from("precomputed/pool_metrics/non_zero.parquet");
        self.write_batch_to_store(output_path, batch).await?;

        info!("Successfully wrote precomputed non-zero proportions");
        Ok(())
    }

    pub async fn write_histograms(&self) -> Result<(), anyhow::Error> {
        info!("Starting precomputation of histogram distributions");
        
        let schema = arrow::datatypes::Schema::new(vec![
            arrow::datatypes::Field::new("pool_address", arrow::datatypes::DataType::Utf8, false),
            arrow::datatypes::Field::new("pool_name", arrow::datatypes::DataType::Utf8, false),
            arrow::datatypes::Field::new("markout_time", arrow::datatypes::DataType::Utf8, false),
            arrow::datatypes::Field::new("bucket_range_start", arrow::datatypes::DataType::Float64, false),
            arrow::datatypes::Field::new("bucket_range_end", arrow::datatypes::DataType::Float64, true),
            arrow::datatypes::Field::new("count", arrow::datatypes::DataType::UInt64, false),
            arrow::datatypes::Field::new("label", arrow::datatypes::DataType::Utf8, false),
        ]);

        let mut pool_addresses = Vec::new();
        let mut pool_names = Vec::new();
        let mut markout_times = Vec::new();
        let mut bucket_starts = Vec::new();
        let mut bucket_ends = Vec::new();
        let mut counts = Vec::new();
        let mut labels = Vec::new();

        let valid_pools = get_valid_pools();
        let checkpoints_path = object_store::path::Path::from("checkpoints");
        let mut checkpoint_files = self.object_store.list(Some(&checkpoints_path));

        while let Some(meta_result) = checkpoint_files.next().await {
            let meta = meta_result.context("Failed to get file metadata")?;
            let file_path = meta.location.to_string();

            // Extract pool address and markout time from file path
            let pool_address = file_path
                .split('/')
                .last()
                .and_then(|s| s.split('_').next())
                .context("Failed to extract pool address")?
                .to_lowercase();

            if !valid_pools.contains(&pool_address) {
                continue;
            }

            let markout_time = file_path
                .split('_')
                .last()
                .and_then(|s| s.strip_suffix(".parquet"))
                .context("Failed to extract markout time")?;

            let bytes = self.object_store.get(&meta.location).await?.bytes().await?;
            let record_reader = ParquetRecordBatchReader::try_new(bytes, 1)?;

            for batch_result in record_reader {
                let batch = batch_result?;

                // Define bucket configurations
                let bucket_configs = vec![
                    (0.01, Some(10.0), "total_bucket_0_10", "$0.01-$10"),
                    (10.0, Some(100.0), "total_bucket_10_100", "$10-$100"),
                    (100.0, Some(500.0), "total_bucket_100_500", "$100-$500"),
                    (500.0, Some(1000.0), "total_bucket_500_1000", "$500-$1K"),
                    (1000.0, Some(10000.0), "total_bucket_1000_10000", "$1K-$10K"),
                    (10000.0, None, "total_bucket_10000_plus", "$10K+"),
                ];

                let mut has_data = false;
                let pool_name = get_pool_name(&pool_address);

                // Process each bucket
                for (start, end, column_name, label) in bucket_configs {
                    let count = histogram::get_bucket_value(&batch, column_name)
                        .map_err(|e| anyhow::anyhow!("Failed to get {} value: {}", column_name, e))?;

                    if count > 0 {
                        has_data = true;
                        pool_addresses.push(pool_address.clone());
                        pool_names.push(pool_name.clone());
                        markout_times.push(markout_time.to_string());
                        bucket_starts.push(start);
                        bucket_ends.push(end);
                        counts.push(count);
                        labels.push(label.to_string());
                    }
                }

                if has_data {
                    debug!(
                        "Added histogram data for pool {} with markout time {}", 
                        pool_address, markout_time
                    );
                }
            }
        }

        // Create record batch
        let batch = RecordBatch::try_new(
            Arc::new(schema),
            vec![
                Arc::new(StringArray::from(pool_addresses)),
                Arc::new(StringArray::from(pool_names)),
                Arc::new(StringArray::from(markout_times)),
                Arc::new(Float64Array::from(bucket_starts)),
                Arc::new(Float64Array::from(
                    bucket_ends.iter().map(|opt| *opt).collect::<Vec<_>>()
                )),
                Arc::new(UInt64Array::from(counts)),
                Arc::new(StringArray::from(labels)),
            ],
        )?;

        // Write to output file
        let output_path = Path::from("precomputed/distributions/histograms.parquet");
        self.write_batch_to_store(output_path, batch).await?;

        info!("Successfully wrote precomputed histogram distributions");
        Ok(())
    }

    pub async fn write_percentile_bands(&self) -> Result<(), anyhow::Error> {
        info!("Starting precomputation of percentile band distributions");
    
        let schema = arrow::datatypes::Schema::new(vec![
            arrow::datatypes::Field::new("pool_address", arrow::datatypes::DataType::Utf8, false),
            arrow::datatypes::Field::new("pool_name", arrow::datatypes::DataType::Utf8, false),
            arrow::datatypes::Field::new("markout_time", arrow::datatypes::DataType::Utf8, false),
            arrow::datatypes::Field::new("start_block", arrow::datatypes::DataType::UInt64, false),
            arrow::datatypes::Field::new("end_block", arrow::datatypes::DataType::UInt64, false),
            arrow::datatypes::Field::new("total_lvr_dollars", arrow::datatypes::DataType::Float64, false),
            arrow::datatypes::Field::new("percentile_25_dollars", arrow::datatypes::DataType::Float64, false),
            arrow::datatypes::Field::new("median_dollars", arrow::datatypes::DataType::Float64, false),
            arrow::datatypes::Field::new("percentile_75_dollars", arrow::datatypes::DataType::Float64, false),
        ]);
    
        let mut pool_addresses = Vec::new();
        let mut pool_names = Vec::new();
        let mut markout_times = Vec::new();
        let mut start_blocks = Vec::new();
        let mut end_blocks = Vec::new();
        let mut total_lvr_values = Vec::new();
        let mut percentile_25_values = Vec::new();
        let mut median_values = Vec::new();
        let mut percentile_75_values = Vec::new();
    
        let valid_pools = get_valid_pools();
    
        // Process all interval files
        let intervals_path = object_store::path::Path::from("intervals");
        let mut interval_files = self.object_store.list(Some(&intervals_path));
    
        while let Some(meta_result) = interval_files.next().await {
            let meta = meta_result.context("Failed to get file metadata")?;
            let file_path = meta.location.to_string();
    
            // Extract block range from file name
            let (file_start, file_end) = if let Some(file_name) = file_path.split('/').last() {
                let parts: Vec<&str> = file_name.split('_').collect();
                if parts.len() == 2 {
                    let start = parts[0].parse::<u64>().context("Failed to parse start block")?;
                    let end = parts[1].trim_end_matches(".parquet").parse::<u64>()
                        .context("Failed to parse end block")?;
                    (start, end)
                } else {
                    continue;
                }
            } else {
                continue;
            };
    
            let bytes = self.object_store.get(&meta.location).await?.bytes().await?;
            let record_reader = ParquetRecordBatchReader::try_new(bytes, 1024)?;
    
            // Collect and group data for this interval file
            let mut interval_data: HashMap<(String, String), Vec<(u64, u64, u64)>> = HashMap::new();
    
            for batch_result in record_reader {
                let batch = batch_result?;
    
                let markout_times_col = get_string_column(&batch, "markout_time")
                .map_err(|e| anyhow::anyhow!("Failed to get markout_time column: {}", e))?;
                let pool_addresses_col = get_string_column(&batch, "pair_address")
                    .map_err(|e| anyhow::anyhow!("Failed to get pair_address column: {}", e))?;
                let total_lvr_cents = get_uint64_column(&batch, "total_lvr_cents")
                    .map_err(|e| anyhow::anyhow!("Failed to get total_lvr_cents column: {}", e))?;
                let non_zero_counts = get_uint64_column(&batch, "non_zero_count")
                    .map_err(|e| anyhow::anyhow!("Failed to get non_zero_count column: {}", e))?;
                let total_counts = get_uint64_column(&batch, "total_count")
                    .map_err(|e| anyhow::anyhow!("Failed to get total_count column: {}", e))?;
    
                for i in 0..batch.num_rows() {
                    let pool_address = pool_addresses_col.value(i).to_lowercase();
                    if !valid_pools.contains(&pool_address) {
                        continue;
                    }
    
                    let markout_time = markout_times_col.value(i).to_string();
                    let lvr_cents = total_lvr_cents.value(i);
                    let non_zero_count = non_zero_counts.value(i);
                    let total_count = total_counts.value(i);
    
                    if lvr_cents > 0 && total_count > 0 {
                        interval_data
                            .entry((pool_address.clone(), markout_time.clone()))
                            .or_default()
                            .push((lvr_cents, non_zero_count, total_count));
                    }
                }
            }
    
            // Process collected data for this interval
            for ((pool_address, markout_time), values) in interval_data {
                // Calculate weighted percentiles
                let weighted_percentile = |target: f64| -> f64 {
                    Self::calculate_weighted_percentile(
                        &values
                            .iter()
                            .map(|(lvr, non_zero, total)| (*lvr, *non_zero, *total))
                            .collect::<Vec<_>>(),
                        target,
                    ) as f64 / 100.0
                };
    
                let total_lvr = values.iter().map(|(lvr, _, _)| *lvr).sum::<u64>() as f64 / 100.0;
                let p25 = weighted_percentile(0.25);
                let p50 = weighted_percentile(0.50);
                let p75 = weighted_percentile(0.75);
    
                let pool_name = get_pool_name(&pool_address);
    
                pool_addresses.push(pool_address);
                pool_names.push(pool_name);
                markout_times.push(markout_time);
                start_blocks.push(file_start);
                end_blocks.push(file_end);
                total_lvr_values.push(total_lvr);
                percentile_25_values.push(p25);
                median_values.push(p50);
                percentile_75_values.push(p75);
            }
        }
    
        // Create record batch
        let batch = RecordBatch::try_new(
            Arc::new(schema),
            vec![
                Arc::new(StringArray::from(pool_addresses)),
                Arc::new(StringArray::from(pool_names)),
                Arc::new(StringArray::from(markout_times)),
                Arc::new(UInt64Array::from(start_blocks)),
                Arc::new(UInt64Array::from(end_blocks)),
                Arc::new(Float64Array::from(total_lvr_values)),
                Arc::new(Float64Array::from(percentile_25_values)),
                Arc::new(Float64Array::from(median_values)),
                Arc::new(Float64Array::from(percentile_75_values)),
            ],
        )?;
    
        // Write to output file
        let output_path = Path::from("precomputed/distributions/percentile_bands.parquet");
        self.write_batch_to_store(output_path, batch).await?;
    
        info!("Successfully wrote precomputed percentile band distributions");
        Ok(())
    }
    

    pub async fn write_quartile_plots(&self) -> Result<(), anyhow::Error> {
        info!("Starting precomputation of quartile plot distributions");
    
        let schema = arrow::datatypes::Schema::new(vec![
            arrow::datatypes::Field::new("pool_address", arrow::datatypes::DataType::Utf8, false),
            arrow::datatypes::Field::new("pool_name", arrow::datatypes::DataType::Utf8, false),
            arrow::datatypes::Field::new("markout_time", arrow::datatypes::DataType::Utf8, false),
            arrow::datatypes::Field::new("percentile_25_cents", arrow::datatypes::DataType::UInt64, false),
            arrow::datatypes::Field::new("median_cents", arrow::datatypes::DataType::UInt64, false),
            arrow::datatypes::Field::new("percentile_75_cents", arrow::datatypes::DataType::UInt64, false),
        ]);
    
        let mut pool_addresses = Vec::new();
        let mut pool_names = Vec::new();
        let mut markout_times = Vec::new();
        let mut percentile_25_values = Vec::new();
        let mut median_values = Vec::new();
        let mut percentile_75_values = Vec::new();
    
        // Map to collect segment-level percentiles and weights
        let mut distribution_data: HashMap<(String, String), Vec<(u64, u64, u64, u64, u64)>> = HashMap::new();
    
        let intervals_path = object_store::path::Path::from("intervals");
        let mut interval_files = self.object_store.list(Some(&intervals_path));
        let valid_pools = get_valid_pools();
    
        while let Some(meta_result) = interval_files.next().await {
            let meta = meta_result.context("Failed to get file metadata")?;
            let bytes = self.object_store.get(&meta.location).await?.bytes().await?;
            let record_reader = ParquetRecordBatchReader::try_new(bytes, 1024)?;
    
            for batch_result in record_reader {
                let batch = batch_result?; 
                let markout_times_col = get_string_column(&batch, "markout_time")
                .map_err(|e| anyhow::anyhow!("Failed to get markout_time column: {}", e))?;
                let pool_addresses_col = get_string_column(&batch, "pair_address")
                    .map_err(|e| anyhow::anyhow!("Failed to get pair_address column: {}", e))?;
                let percentile_25_cents = get_uint64_column(&batch, "percentile_25_cents")
                    .map_err(|e| anyhow::anyhow!("Failed to get percentile_25_cents column: {}", e))?;
                let median_cents = get_uint64_column(&batch, "median_lvr_cents")
                    .map_err(|e| anyhow::anyhow!("Failed to get median_cents column: {}", e))?;
                let percentile_75_cents = get_uint64_column(&batch, "percentile_75_cents")
                    .map_err(|e| anyhow::anyhow!("Failed to get percentile_75_cents column: {}", e))?;
                let non_zero_counts = get_uint64_column(&batch, "non_zero_count")
                    .map_err(|e| anyhow::anyhow!("Failed to get non_zero_count column: {}", e))?;
                let total_counts = get_uint64_column(&batch, "total_count")
                    .map_err(|e| anyhow::anyhow!("Failed to get total_count column: {}", e))?;

    
                for i in 0..batch.num_rows() {
                    let total_count = total_counts.value(i);
                    if total_count == 0 {
                        continue;
                    }
    
                    let pool_address = pool_addresses_col.value(i).to_lowercase();
                    if !valid_pools.contains(&pool_address) {
                        continue;
                    }
    
                    let markout_time = markout_times_col.value(i).to_string();
                    let segment_data = (
                        percentile_25_cents.value(i),
                        median_cents.value(i),
                        percentile_75_cents.value(i),
                        non_zero_counts.value(i),
                        total_count,
                    );
    
                    distribution_data
                        .entry((pool_address, markout_time))
                        .or_default()
                        .push(segment_data);
                }
            }
        }
    
        for ((pool_address, markout_time), data) in distribution_data {
            let pool_name = get_pool_name(&pool_address);
    
            let weighted_25 = Self::calculate_weighted_percentile(
                &data.iter()
                    .map(|(p25, _, _, nz_count, t_count)| (*p25, *nz_count, *t_count))
                    .collect::<Vec<_>>(),
                0.25,
            );
            let weighted_50 = Self::calculate_weighted_percentile(
                &data.iter()
                    .map(|(_, p50, _, nz_count, t_count)| (*p50, *nz_count, *t_count))
                    .collect::<Vec<_>>(),
                0.50,
            );
            let weighted_75 = Self::calculate_weighted_percentile(
                &data.iter()
                    .map(|(_, _, p75, nz_count, t_count)| (*p75, *nz_count, *t_count))
                    .collect::<Vec<_>>(),
                0.75,
            );
    
            pool_addresses.push(pool_address);
            pool_names.push(pool_name);
            markout_times.push(markout_time);
            percentile_25_values.push(weighted_25);
            median_values.push(weighted_50);
            percentile_75_values.push(weighted_75);
        }
    
        let batch = RecordBatch::try_new(
            Arc::new(schema),
            vec![
                Arc::new(StringArray::from(pool_addresses)),
                Arc::new(StringArray::from(pool_names)),
                Arc::new(StringArray::from(markout_times)),
                Arc::new(UInt64Array::from(percentile_25_values)),
                Arc::new(UInt64Array::from(median_values)),
                Arc::new(UInt64Array::from(percentile_75_values)),
            ],
        )?;
    
        let output_path = Path::from("precomputed/distributions/quartile_plots.parquet");
        self.write_batch_to_store(output_path, batch).await?;
    
        info!("Successfully wrote precomputed quartile plot distributions");
        Ok(())
    }
    
    pub fn calculate_weighted_percentile(percentiles: &[(u64, u64, u64)], target: f64) -> u64 {
        let mut expanded: Vec<u64> = Vec::new();
        for &(value, non_zero_count, total_count) in percentiles {
            let normalized_weight = (non_zero_count as f64 / total_count as f64) * 10000.0;
            expanded.extend(std::iter::repeat(value).take(normalized_weight as usize));
        }
        expanded.sort_unstable();
        calculate_percentile(&expanded, target)
    }

    pub async fn write_cluster_proportions(&self) -> Result<(), anyhow::Error> {
        info!("Starting precomputation of cluster proportions");
        
        let schema = arrow::datatypes::Schema::new(vec![
            arrow::datatypes::Field::new("cluster_name", arrow::datatypes::DataType::Utf8, false),
            arrow::datatypes::Field::new("markout_time", arrow::datatypes::DataType::Utf8, false),
            arrow::datatypes::Field::new("total_lvr_cents", arrow::datatypes::DataType::UInt64, false),
            arrow::datatypes::Field::new("proportion", arrow::datatypes::DataType::Float64, false),
        ]);

        let mut cluster_names = Vec::new();
        let mut markout_times = Vec::new();
        let mut total_lvr_values = Vec::new();
        let mut proportions = Vec::new();

        // Process checkpoint files to get proportions for each markout time
        let checkpoints_path = object_store::path::Path::from("checkpoints");
        let mut checkpoint_files = self.object_store.list(Some(&checkpoints_path));

        // Map to store results by markout time
        let mut markout_data: HashMap<String, HashMap<String, u64>> = HashMap::new();

        // Process all checkpoint files
        while let Some(meta_result) = checkpoint_files.next().await {
            let meta = meta_result.context("Failed to get file metadata")?;
            let file_path = meta.location.to_string();

            // Extract markout time from file path
            let markout_time = file_path
                .split('_')
                .last()
                .and_then(|s| s.strip_suffix(".parquet"))
                .context("Failed to extract markout time from file path")?;

            let bytes = self.object_store.get(&meta.location).await?.bytes().await?;
            let record_reader = ParquetRecordBatchReader::try_new(bytes, 1)?;

            for batch_result in record_reader {
                let batch = batch_result?;

                let pair_addresses = get_string_column(&batch, "pair_address")
                    .map_err(|e| anyhow::anyhow!("Failed to get pair_address column: {}", e))?;
                let running_totals = get_uint64_column(&batch, "running_total")
                    .map_err(|e| anyhow::anyhow!("Failed to get running_total column: {}", e))?;

                // Process each row
                for i in 0..batch.num_rows() {
                    let pool_address = pair_addresses.value(i);
                    let running_total = running_totals.value(i);

                    // Get the cluster name for this pool
                    if let Some(cluster_name) = clusters::get_cluster_name(pool_address) {
                        markout_data
                            .entry(markout_time.to_string())
                            .or_default()
                            .entry(cluster_name.to_string())
                            .and_modify(|total| *total = total.saturating_add(running_total))
                            .or_insert(running_total);
                    }
                }
            }
        }

        // Convert aggregated data into final format
        for (markout_time, cluster_totals) in markout_data {
            let total_lvr_cents: u64 = cluster_totals.values().sum();

            for (cluster_name, cluster_total) in cluster_totals {
                let proportion = if total_lvr_cents > 0 {
                    cluster_total as f64 / total_lvr_cents as f64
                } else {
                    0.0
                };

                cluster_names.push(cluster_name);
                markout_times.push(markout_time.clone());
                total_lvr_values.push(cluster_total);
                proportions.push(proportion);
            }
        }

        // Create record batch
        let batch = RecordBatch::try_new(
            Arc::new(schema),
            vec![
                Arc::new(StringArray::from(cluster_names)),
                Arc::new(StringArray::from(markout_times)),
                Arc::new(UInt64Array::from(total_lvr_values)),
                Arc::new(Float64Array::from(proportions)),
            ],
        )?;

        // Write to output file
        let output_path = Path::from("precomputed/clusters/proportions.parquet");
        self.write_batch_to_store(output_path, batch).await?;

        info!("Successfully wrote precomputed cluster proportions");
        Ok(())
    }

    pub async fn write_cluster_histograms(&self) -> Result<(), anyhow::Error> {
        info!("Starting precomputation of cluster histogram distributions");
        
        let schema = arrow::datatypes::Schema::new(vec![
            arrow::datatypes::Field::new("cluster_name", arrow::datatypes::DataType::Utf8, false),
            arrow::datatypes::Field::new("markout_time", arrow::datatypes::DataType::Utf8, false),
            arrow::datatypes::Field::new("bucket_range_start", arrow::datatypes::DataType::Float64, false),
            arrow::datatypes::Field::new("bucket_range_end", arrow::datatypes::DataType::Float64, true),
            arrow::datatypes::Field::new("count", arrow::datatypes::DataType::UInt64, false),
            arrow::datatypes::Field::new("label", arrow::datatypes::DataType::Utf8, false),
        ]);

        let mut cluster_names = Vec::new();
        let mut markout_times = Vec::new();
        let mut bucket_starts = Vec::new();
        let mut bucket_ends = Vec::new();
        let mut counts = Vec::new();
        let mut labels = Vec::new();

        // Process checkpoint files
        let checkpoints_path = object_store::path::Path::from("checkpoints");
        let mut checkpoint_files = self.object_store.list(Some(&checkpoints_path));

        // Map to store intermediate histogram data
        let mut cluster_data: HashMap<(String, String), Vec<u64>> = HashMap::new();

        while let Some(meta_result) = checkpoint_files.next().await {
            let meta = meta_result.context("Failed to get file metadata")?;
            let file_path = meta.location.to_string();

            let markout_time = file_path
                .split('_')
                .last()
                .and_then(|s| s.strip_suffix(".parquet"))
                .context("Failed to extract markout time")?;

            let bytes = self.object_store.get(&meta.location).await?.bytes().await?;
            let record_reader = ParquetRecordBatchReader::try_new(bytes, 1)?;

            for batch_result in record_reader {
                let batch = batch_result?;

                let pair_addresses = get_string_column(&batch, "pair_address")
                    .map_err(|e| anyhow::anyhow!("Failed to get pair_address column: {}", e))?;

                // Get all bucket columns
                let bucket_names = [
                    "total_bucket_0_10",
                    "total_bucket_10_100",
                    "total_bucket_100_500",
                    "total_bucket_500_1000",
                    "total_bucket_1000_10000",
                    "total_bucket_10000_plus",
                ];

                let mut bucket_columns = Vec::new();
                for name in &bucket_names {
                    let column = get_uint64_column(&batch, name)
                        .map_err(|e| anyhow::anyhow!("Failed to get {} column: {}", name, e))?;
                    bucket_columns.push(column);
                }

                // Process each row
                for row in 0..batch.num_rows() {
                    let pool_address = pair_addresses.value(row);
                    
                    // Get cluster name for this pool
                    if let Some(cluster_name) = get_cluster_name(&pool_address.to_lowercase()) {
                        let bucket_values: Vec<u64> = bucket_columns
                            .iter()
                            .map(|col| col.value(row))
                            .collect();

                        // Aggregate values by cluster and markout time
                        cluster_data
                            .entry((cluster_name.to_string(), markout_time.to_string()))
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

        // Define bucket configurations
        let bucket_configs = vec![
            (0.01, Some(10.0), "$0.01-$10"),
            (10.0, Some(100.0), "$10-$100"),
            (100.0, Some(500.0), "$100-$500"),
            (500.0, Some(3000.0), "$500-$3K"),
            (3000.0, Some(10000.0), "$3K-$10K"),
            (10000.0, Some(30000.0), "$10K-$30K"),
            (30000.0, None, "$30K+"),
        ];

        // Convert aggregated data into row format
        for ((cluster_name, markout_time), bucket_counts) in cluster_data {
            for ((start, end, label), count) in bucket_configs.iter().zip(bucket_counts.iter()) {
                if *count > 0 {
                    cluster_names.push(cluster_name.clone());
                    markout_times.push(markout_time.clone());
                    bucket_starts.push(*start);
                    bucket_ends.push(*end);
                    counts.push(*count);
                    labels.push(label.to_string());
                }
            }
        }

        // Create record batch
        let batch = RecordBatch::try_new(
            Arc::new(schema),
            vec![
                Arc::new(StringArray::from(cluster_names)),
                Arc::new(StringArray::from(markout_times)),
                Arc::new(Float64Array::from(bucket_starts)),
                Arc::new(Float64Array::from(
                    bucket_ends.into_iter().map(|opt| opt).collect::<Vec<Option<f64>>>()
                )),
                Arc::new(UInt64Array::from(counts)),
                Arc::new(StringArray::from(labels)),
            ],
        )?;

        // Write to output file
        let output_path = Path::from("precomputed/clusters/histograms.parquet");
        self.write_batch_to_store(output_path, batch).await?;

        info!("Successfully wrote precomputed cluster histogram distributions");
        Ok(())
    }

    pub async fn write_monthly_cluster_totals(&self) -> Result<(), anyhow::Error> {
        info!("Starting precomputation of monthly cluster totals");
        
        let schema = arrow::datatypes::Schema::new(vec![
            arrow::datatypes::Field::new("time_range", arrow::datatypes::DataType::Utf8, false),
            arrow::datatypes::Field::new("cluster_name", arrow::datatypes::DataType::Utf8, false),
            arrow::datatypes::Field::new("markout_time", arrow::datatypes::DataType::Utf8, false),
            arrow::datatypes::Field::new("total_lvr_cents", arrow::datatypes::DataType::UInt64, false),
        ]);

        let mut time_ranges = Vec::new();
        let mut cluster_names = Vec::new();
        let mut markout_times = Vec::new();
        let mut total_lvr_values = Vec::new();

        let intervals_path = object_store::path::Path::from("intervals");
        let mut interval_files = self.object_store.list(Some(&intervals_path));
        
        // Collect data by start block and cluster
        let mut monthly_data: HashMap<(u64, String, String), u64> = HashMap::new();
        let mut files_processed = 0;
        
        while let Some(meta_result) = interval_files.next().await {
            files_processed += 1;
            let meta = meta_result.context("Failed to get file metadata")?;
            let file_path = meta.location.to_string();
            
            // Extract start block from file path
            let start_block = file_path
                .split('/')
                .last()
                .and_then(|name| name.split('_').next())
                .and_then(|num| num.parse::<u64>().ok())
                .context("Failed to parse start block")?;

            // Skip if we don't have a time range for this start block
            if !INTERVAL_RANGES.contains_key(&start_block) {
                continue;
            }

            let bytes = self.object_store.get(&meta.location).await?.bytes().await?;
            let record_reader = ParquetRecordBatchReader::try_new(bytes, 1024)?;

            for batch_result in record_reader {
                let batch = batch_result?;

                let markout_times_col = get_string_column(&batch, "markout_time")
                    .map_err(|e| anyhow::anyhow!("Failed to get markout_time column: {}", e))?;
                let pair_addresses = get_string_column(&batch, "pair_address")
                    .map_err(|e| anyhow::anyhow!("Failed to get pair_address column: {}", e))?;
                let total_lvr_cents = get_uint64_column(&batch, "total_lvr_cents")
                    .map_err(|e| anyhow::anyhow!("Failed to get total_lvr_cents column: {}", e))?;
                let non_zero_counts = get_uint64_column(&batch, "non_zero_count")
                    .map_err(|e| anyhow::anyhow!("Failed to get non_zero_count column: {}", e))?;

                for i in 0..batch.num_rows() {
                    if non_zero_counts.value(i) == 0 {
                        continue;
                    }

                    let pool_address = pair_addresses.value(i).to_lowercase();
                    if let Some(cluster_name) = get_cluster_name(&pool_address) {
                        let markout_time = markout_times_col.value(i).to_string();
                        let lvr_cents = total_lvr_cents.value(i);

                        monthly_data
                            .entry((start_block, cluster_name.to_string(), markout_time))
                            .and_modify(|total| *total = total.saturating_add(lvr_cents))
                            .or_insert(lvr_cents);
                    }
                }
            }
        }

        // Convert collected data into row format
        for ((start_block, cluster_name, markout_time), total_cents) in monthly_data {
            if let Some(&time_range) = INTERVAL_RANGES.get(&start_block) {
                time_ranges.push(time_range.to_string());
                cluster_names.push(cluster_name);
                markout_times.push(markout_time);
                total_lvr_values.push(total_cents);
            }
        }

        // Create record batch
        let batch = RecordBatch::try_new(
            Arc::new(schema),
            vec![
                Arc::new(StringArray::from(time_ranges)),
                Arc::new(StringArray::from(cluster_names)),
                Arc::new(StringArray::from(markout_times)),
                Arc::new(UInt64Array::from(total_lvr_values)),
            ],
        )?;

        // Write to output file
        let output_path = Path::from("precomputed/clusters/monthly_totals.parquet");
        self.write_batch_to_store(output_path, batch).await?;

        info!(
            "Successfully wrote precomputed monthly cluster totals (processed {} files)", 
            files_processed
        );
        Ok(())
    }

    pub async fn write_cluster_non_zero(&self) -> Result<(), anyhow::Error> {
        info!("Starting precomputation of cluster non-zero proportions");
        
        let schema = arrow::datatypes::Schema::new(vec![
            arrow::datatypes::Field::new("cluster_name", arrow::datatypes::DataType::Utf8, false),
            arrow::datatypes::Field::new("markout_time", arrow::datatypes::DataType::Utf8, false),
            arrow::datatypes::Field::new("total_observations", arrow::datatypes::DataType::UInt64, false),
            arrow::datatypes::Field::new("non_zero_observations", arrow::datatypes::DataType::UInt64, false),
            arrow::datatypes::Field::new("non_zero_proportion", arrow::datatypes::DataType::Float64, false),
        ]);

        let mut cluster_names = Vec::new();
        let mut markout_times = Vec::new();
        let mut total_observations = Vec::new();
        let mut non_zero_observations = Vec::new();
        let mut non_zero_proportions = Vec::new();

        // Process checkpoint files
        let checkpoints_path = object_store::path::Path::from("checkpoints");
        let mut checkpoint_files = self.object_store.list(Some(&checkpoints_path));

        // Store cluster stats by markout time
        let mut cluster_stats: HashMap<(String, String), (u64, u64)> = HashMap::new();
        
        while let Some(meta_result) = checkpoint_files.next().await {
            let meta = meta_result.context("Failed to get file metadata")?;
            let file_path = meta.location.to_string();

            // Extract markout time from file path
            let markout_time = file_path
                .split('_')
                .last()
                .and_then(|s| s.strip_suffix(".parquet"))
                .context("Failed to extract markout time")?;

            let bytes = self.object_store.get(&meta.location).await?.bytes().await?;
            let record_reader = ParquetRecordBatchReader::try_new(bytes, 1)?;

            for batch_result in record_reader {
                let batch = batch_result?;

                let pool_addresses = get_string_column(&batch, "pair_address")
                    .map_err(|e| anyhow::anyhow!("Failed to get pair_address column: {}", e))?;
                let total_bucket_0 = get_uint64_column(&batch, "total_bucket_0")
                    .map_err(|e| anyhow::anyhow!("Failed to get total_bucket_0 column: {}", e))?;

                // Get all non-zero bucket columns
                let non_zero_buckets = [
                    "total_bucket_0_10",
                    "total_bucket_10_100",
                    "total_bucket_100_500",
                    "total_bucket_500_1000",
                    "total_bucket_1000_10000",
                    "total_bucket_10000_plus",
                ];

                for i in 0..batch.num_rows() {
                    let pool_address = pool_addresses.value(i).to_lowercase();
                    
                    if let Some(cluster_name) = get_cluster_name(&pool_address) {
                        let zero_count = total_bucket_0.value(i);
                        let mut non_zero_count = 0u64;

                        for bucket_name in &non_zero_buckets {
                            let bucket = get_uint64_column(&batch, bucket_name)
                                .map_err(|e| anyhow::anyhow!("Failed to get {} column: {}", bucket_name, e))?;
                            non_zero_count = non_zero_count.saturating_add(bucket.value(i));
                        }

                        cluster_stats
                            .entry((cluster_name.to_string(), markout_time.to_string()))
                            .and_modify(|(total, non_zero)| {
                                *total = total.saturating_add(zero_count + non_zero_count);
                                *non_zero = non_zero.saturating_add(non_zero_count);
                            })
                            .or_insert((zero_count + non_zero_count, non_zero_count));
                    }
                }
            }
        }

        // Convert aggregated data into row format
        for ((cluster_name, markout_time), (total, non_zero)) in cluster_stats {
            let proportion = if total > 0 {
                non_zero as f64 / total as f64
            } else {
                0.0
            };

            cluster_names.push(cluster_name);
            markout_times.push(markout_time);
            total_observations.push(total);
            non_zero_observations.push(non_zero);
            non_zero_proportions.push(proportion);
        }

        // Create record batch
        let batch = RecordBatch::try_new(
            Arc::new(schema),
            vec![
                Arc::new(StringArray::from(cluster_names)),
                Arc::new(StringArray::from(markout_times)),
                Arc::new(UInt64Array::from(total_observations)),
                Arc::new(UInt64Array::from(non_zero_observations)),
                Arc::new(Float64Array::from(non_zero_proportions)),
            ],
        )?;

        // Write to output file
        let output_path = Path::from("precomputed/clusters/non_zero.parquet");
        self.write_batch_to_store(output_path, batch).await?;

        info!("Successfully wrote precomputed cluster non-zero proportions");
        Ok(())
    }
}