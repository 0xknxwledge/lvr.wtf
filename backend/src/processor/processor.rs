use crate::{
    api::precompute::PrecomputedWriter, aurora::{AuroraConnection, LVRDetails}, brontes::{BrontesConnection, LVRAnalysis}, config::{AuroraConfig, BrontesConfig}, error::Error, models::{Checkpoint, CheckpointUpdate, DataSource, IntervalData, MarkoutTime, UnifiedLVRData},
     writer::ParallelParquetWriter, 
     USDeUSDT_DEPLOYMENT, 
     MARKOUT_TIMES, MARKOUT_TIME_MAPPING, 
     PEPE_DEPLOYMENT_V2, PEPE_DEPLOYMENT_V3,
      POOL_ADDRESSES, POOL_NAMES, BRONTES_ADDRESSES, WETH_USDT_100_DEPLOYMENT
};
use anyhow::Result;
use dashmap::DashMap;
use ordered_float::OrderedFloat;
use std::{collections::{HashSet,HashMap}, sync::Arc};
use tracing::{info, error, warn, debug};
use object_store::ObjectStore;
use std::sync::atomic::Ordering;
use futures::stream::{FuturesOrdered, StreamExt};
use futures::lock::Mutex;
use tokio::sync::Barrier;
use anyhow::Context;

const BLOCKS_PER_DAY: u64 = 7200;
const INTERVALS_PER_FILE: u64 = 30;
const BLOCKS_PER_CHUNK: u64 = BLOCKS_PER_DAY * INTERVALS_PER_FILE;

// Structure to hold processed data before committing
#[derive(Debug)]
struct ProcessedData {
    intervals: Vec<IntervalData>
}

pub struct ParallelLVRProcessor {
    start_block: u64,
    end_block: u64,
    checkpoints: Arc<DashMap<(String, MarkoutTime), Checkpoint>>,
    aurora_connection: Arc<AuroraConnection>,
    brontes_connection: Arc<BrontesConnection>,
    parquet_writer: Arc<Mutex<ParallelParquetWriter>>,
    update_barrier: Arc<Barrier>,
    object_store: Arc<dyn ObjectStore>
}

impl ParallelLVRProcessor {
    pub async fn new(
        start_block: u64,
        end_block: u64,
        object_store: Arc<dyn ObjectStore>
    ) -> Result<Self, Error> {
        let aurora_config = AuroraConfig::from_env()?;
        let brontes_config = BrontesConfig::from_env()?;
        
        let aurora_connection = Arc::new(AuroraConnection::new(aurora_config)?);
        let brontes_connection = Arc::new(BrontesConnection::new(brontes_config)?);
        let parquet_writer = Arc::new(Mutex::new(ParallelParquetWriter::new(object_store.clone())));

        Ok(Self {
            start_block,
            end_block,
            checkpoints: Arc::new(DashMap::new()),
            aurora_connection,
            brontes_connection,
            parquet_writer,
            update_barrier: Arc::new(Barrier::new(1)),
            object_store
        })
    }

    fn get_deployment_block(&self, pool_address: &str) -> u64 {
        match pool_address.to_lowercase().as_str() {
            "0x11950d141ecb863f01007add7d1a342041227b58" => *PEPE_DEPLOYMENT_V3,
            "0xa43fe16908251ee70ef74718545e4fe6c5ccec9f" => *PEPE_DEPLOYMENT_V2,
            "0x435664008f38b0650fbc1c9fc971d0a3bc2f1e47" => *USDeUSDT_DEPLOYMENT,
            "0xc7bbec68d12a0d1830360f8ec58fa599ba1b0e9b" => *WETH_USDT_100_DEPLOYMENT,
            _ => 0, // Pre-merge pools
        }
    }

    pub async fn process_blocks(
        &self,
        validation_callback: Option<fn(&Arc<dyn ObjectStore>) -> futures::future::BoxFuture<'_, Result<()>>>
    ) -> Result<()> {
        info!("Starting block processing from {} to {}", self.start_block, self.end_block);
        let total_blocks = self.end_block - self.start_block;
        let total_chunks = (total_blocks + BLOCKS_PER_CHUNK - 1) / BLOCKS_PER_CHUNK;
        let mut processed_blocks = 0;
        
        for chunk_idx in 0..total_chunks {
            let chunk_start = self.start_block + (chunk_idx * BLOCKS_PER_CHUNK);
            let chunk_end = std::cmp::min(chunk_start + BLOCKS_PER_CHUNK, self.end_block);
            
            match self.process_chunk_with_retries(chunk_idx, chunk_start, chunk_end, total_chunks).await {
                Ok(_) => {
                    processed_blocks += chunk_end - chunk_start;
                    info!(
                        "Successfully processed chunk {}/{}, progress: {:.2}% ({}/{} blocks)", 
                        chunk_idx + 1, total_chunks,
                        (processed_blocks as f64 / total_blocks as f64) * 100.0,
                        processed_blocks, total_blocks
                    );
    
                    // Run validation after each chunk if callback is provided
                    if let Some(validate) = validation_callback {
                        match validate(&self.object_store).await {
                            Ok(_) => info!("Validation passed for chunk {}/{}", chunk_idx + 1, total_chunks),
                            Err(e) => {
                                error!("Validation failed for chunk {}/{}: {}", chunk_idx + 1, total_chunks, e);
                                return Err(e);
                            }
                        }
                    }
                },
                Err(e) => return Err(e),
            }
        }

        // Finalize all checkpoints with delta_final
        info!("Finalizing checkpoints with delta_final parameter...");
        for checkpoint in self.checkpoints.iter_mut() {
            if let Err(e) = checkpoint.value().finalize() {
                error!("Failed to finalize checkpoint for {}-{}: {}", 
                    checkpoint.pair_address, checkpoint.markout_time, e);
            }
        }

        // Write the finalized checkpoints one last time
        self.write_checkpoints().await?;
        info!("Successfully finalized all checkpoints");
        
        info!(
            "Successfully completed processing all blocks from {} to {}", 
            self.start_block, self.end_block
        );

        // Run precomputation after successful processing
        info!("Starting precomputation phase...");
        match self.run_precomputation().await {
            Ok(_) => info!("Successfully completed precomputation phase"),
            Err(e) => {
                error!("Failed to run precomputation: {}", e);
                return Err(e);
            }
        }
        
        Ok(())
    }

    async fn process_chunk_with_retries(
        &self,
        chunk_idx: u64,
        chunk_start: u64,
        chunk_end: u64,
        total_chunks: u64,
    ) -> Result<()> {
        let max_retries = 20;
        let mut attempt = 0;

        loop {
            attempt += 1;
            info!(
                "Processing chunk {}/{} (blocks {} to {}), attempt {}/{}",
                chunk_idx + 1, total_chunks, chunk_start, chunk_end, attempt, max_retries
            );

            match self.process_chunk(chunk_start, chunk_end).await {
                Ok(_) => break Ok(()),
                Err(e) => {
                    if attempt >= max_retries {
                        error!(
                            "Chunk {}/{} failed after {} attempts: {}", 
                            chunk_idx + 1, total_chunks, max_retries, e
                        );
                        break Err(e);
                    }
                    
                    let delay = std::time::Duration::from_secs(5 * attempt as u64);
                    warn!(
                        "Chunk {}/{} failed (attempt {}/{}): {}. Retrying in {} seconds...",
                        chunk_idx + 1, total_chunks, attempt, max_retries, e, delay.as_secs()
                    );
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }

    async fn process_chunk(&self, chunk_start: u64, chunk_end: u64) -> Result<()> {
        // Fetch data from both sources concurrently
        let (aurora_results, brontes_results) = self.fetch_data(chunk_start, chunk_end).await?;
    
        // Process the results but don't update checkpoints yet
        let (processed_data, checkpoint_updates) = self
            .process_results(chunk_start, chunk_end, aurora_results, brontes_results)
            .await?;
    
        // Write interval data if needed
        if chunk_end - chunk_start >= BLOCKS_PER_CHUNK || chunk_end == self.end_block {
            if !processed_data.intervals.is_empty() {
                let mut writer = self.parquet_writer.lock().await;
                writer
                    .write_interval_data(processed_data.intervals, chunk_start, chunk_end)
                    .await?;
            }
        }
    
        // Atomically update and write checkpoints
        self.atomic_checkpoint_update(checkpoint_updates).await?;
    
        Ok(())
    }
    

    async fn fetch_data(
        &self,
        chunk_start: u64,
        chunk_end: u64,
    ) -> Result<(Vec<Vec<LVRDetails>>, Vec<LVRAnalysis>)> {
        // Create concurrent tasks for Aurora
        let mut aurora_tasks = FuturesOrdered::new();
        for &time in MARKOUT_TIMES.iter() {
            let index = *MARKOUT_TIME_MAPPING.get(&OrderedFloat(time))
                .context("Invalid markout time mapping")?;
            let task = self.aurora_connection.fetch_lvr_details(index, chunk_start, chunk_end);
            aurora_tasks.push_back(task);
        }

        // Fetch Brontes data concurrently
        let brontes_task = self.brontes_connection.fetch_lvr_analysis(chunk_start, chunk_end);

        // Wait for all Aurora results
        let mut aurora_results = Vec::new();
        while let Some(result) = aurora_tasks.next().await {
            aurora_results.push(result?);
        }

        // Wait for Brontes results
        let brontes_results = brontes_task.await?;

        Ok((aurora_results, brontes_results))
    }

    async fn process_results(
        &self,
        chunk_start: u64,
        chunk_end: u64,
        aurora_results: Vec<Vec<LVRDetails>>,
        brontes_results: Vec<LVRAnalysis>
    ) -> Result<(ProcessedData, Vec<CheckpointUpdate>)> {
        let unified_data = DashMap::new();
        let mut checkpoint_updates = Vec::new();
        let mut successful_intervals = Vec::new();
    
        // Process Aurora data
        for (markout_idx, aurora_markout_data) in aurora_results.into_iter().enumerate() {
            let markout_time = MarkoutTime::from_f64(MARKOUT_TIMES[markout_idx])
                .context("Invalid markout time")?;
    
            for pool_address in POOL_ADDRESSES.iter() {
                let pool_name = POOL_NAMES.get(*pool_address)
                    .context("Unknown pool address")?;
    
                let aurora_data: Vec<UnifiedLVRData> = aurora_markout_data.iter()
                    .filter_map(|detail| {
                        self.parse_lvr_details(&detail.details, pool_name)
                            .and_then(|lvr| {
                                self.to_cents(lvr).ok().map(|cents| UnifiedLVRData {
                                    block_number: detail.block_number,
                                    lvr_cents: cents,
                                    source: DataSource::Aurora,
                                })
                            })
                    })
                    .collect();
    
                if !aurora_data.is_empty() {
                    unified_data.insert((pool_address.to_string(), markout_time), aurora_data);
                }
            }
        }
    
        // Process Brontes data
        let mut brontes_data: HashMap<String, Vec<UnifiedLVRData>> = HashMap::new();
    
        // First, collect all actual Brontes events
        for result in brontes_results {
            if result.block_number >= chunk_start && result.block_number < chunk_end {
                if let Ok(cents) = self.to_cents(result.lvr) {
                    brontes_data
                        .entry(result.pool_address.to_lowercase())
                        .or_default()
                        .push(UnifiedLVRData {
                            block_number: result.block_number,
                            lvr_cents: cents,
                            source: DataSource::Brontes,
                        });
                }
            }
        }
    
        // Process each Brontes pool
        for pool_address in BRONTES_ADDRESSES.iter() {
            // Get or create vector for this pool
            let mut pool_data = brontes_data
                .remove(*pool_address)
                .unwrap_or_default();
            
            // Sort by block number for efficient lookup
            pool_data.sort_by_key(|data| data.block_number);
            let event_blocks: HashSet<u64> = pool_data
                .iter()
                .map(|data| data.block_number)
                .collect();
    
            // Add zeros for blocks without events
            let mut complete_data = Vec::with_capacity((chunk_end - chunk_start) as usize);
            
            // Add existing events and zeros in sorted order
            for block in chunk_start..chunk_end {
                if event_blocks.contains(&block) {
                    let event = pool_data
                        .iter()
                        .find(|data| data.block_number == block)
                        .unwrap()
                        .clone();
                    complete_data.push(event);
                } else {
                    complete_data.push(UnifiedLVRData {
                        block_number: block,
                        lvr_cents: 0,
                        source: DataSource::Brontes,
                    });
                }
            }
    
            // Insert into unified data (we'll always have at least zeros)
            unified_data.insert(
                (pool_address.to_string(), MarkoutTime::Brontes),
                complete_data
            );
        }
    
        // Process all data
        for entry in unified_data.iter() {
            let (key, data) = entry.pair();
            let (pool_address, markout_time) = key;
            
            // Add checkpoint update
            checkpoint_updates.push(CheckpointUpdate {
                pool_address: pool_address.clone(),
                markout_time: markout_time.clone(),
                data: data.clone(),
                chunk_start,
                chunk_end,
            });
    
            // Calculate intervals
            match self.calculate_interval_metrics(
                chunk_start,
                chunk_end,
                &pool_address,
                markout_time.clone(),
                &data,
            ) {
                Ok(intervals) => successful_intervals.extend(intervals),
                Err(e) => return Err(anyhow::anyhow!(
                    "Interval calculation failed for {}-{}: {}", 
                    pool_address, markout_time, e
                )),
            }
        }
    
        Ok((
            ProcessedData { intervals: successful_intervals },
            checkpoint_updates
        ))
    }

    async fn atomic_checkpoint_update(&self, updates: Vec<CheckpointUpdate>) -> Result<()> {
        // Apply all updates atomically
        for update in updates {
            self.update_checkpoint(
                &update.pool_address,
                update.markout_time,
                &update.data,
                update.chunk_start,
                update.chunk_end,
            ).await?;
        }
        
        // Write all updates at once
        self.write_checkpoints().await?;
        
        Ok(())
    }

    async fn write_checkpoints(&self) -> Result<()> {
        // Log the start of checkpoint writing
        info!("Starting to write checkpoints.");
    
        // Wait for any in-flight updates to complete
        let barrier = self.update_barrier.clone();
    
        // Spawn a task that waits for all updates
        let barrier_wait = tokio::spawn(async move {
            debug!("Waiting for the update barrier to synchronize.");
            barrier.wait().await;
        });
    
        // Wait for the barrier
        barrier_wait.await?;
    
        // Now safely collect and write checkpoints
        let checkpoints: Vec<_> = self
            .checkpoints
            .iter()
            .map(|entry| entry.value().to_snapshot())
            .collect();
    
        debug!(
            "Collected {} checkpoints to write.",
            checkpoints.len()
        );
    
        let mut writer = self.parquet_writer.lock().await;
        writer.write_checkpoints(checkpoints).await?;
    
        // Log the successful completion of checkpoint writing
        info!("Successfully wrote checkpoints.");
    
        Ok(())
    }
    


    fn to_cents(&self, value: f64) -> Result<u64> {
        let cents = (value * 100.0).round();
        
        if cents > u64::MAX as f64 || cents < u64::MIN as f64 {
            return Err(Error::Processing(
                format!("LVR value {} too large for u64 cents representation", value)
            ).into());
        }
        Ok(cents as u64)
    }

    async fn update_checkpoint(
        &self,
        pool_address: &str,
        markout_time: MarkoutTime,
        data: &[UnifiedLVRData],
        chunk_start: u64,
        chunk_end: u64,
    ) -> Result<()> {
        let deployment_block = self.get_deployment_block(pool_address);
        let effective_start = chunk_start.max(deployment_block);
    
        if effective_start >= chunk_end {
            return Ok(());
        }
    
        let checkpoint = self.checkpoints
            .entry((pool_address.to_string(), markout_time))
            .or_insert_with(|| Checkpoint::new(pool_address.to_string(), markout_time));
    
        // Create a map of block numbers to data points for efficient lookup
        let block_data: HashMap<u64, &UnifiedLVRData> = data.iter()
            .filter(|d| d.block_number >= effective_start && d.block_number < chunk_end)
            .map(|d| (d.block_number, d))
            .collect();
    
        let mut updates = 0;
        let mut max_lvr = 0u64;
        let mut max_lvr_block = 0u64;
        let mut running_total = 0i64;
        let mut bucket_counts = [0u64; 7];  // Array for all bucket counts
        let mut non_zero_values = Vec::new();
    
        // Process each block in the range
        for block_number in effective_start..chunk_end {
            updates += 1;
    
            if let Some(data_point) = block_data.get(&block_number) {
                let lvr_cents = data_point.lvr_cents;
                
                // Update running statistics
                running_total += lvr_cents as i64;
                
                // Update max LVR if needed
                if lvr_cents > max_lvr {
                    max_lvr = lvr_cents;
                    max_lvr_block = block_number;
                }
    
                // Collect non-zero values for TDigest
                if lvr_cents > 0 {
                    non_zero_values.push(lvr_cents as f64 / 100.0);  // Convert to dollars for TDigest
                }
    
                // Update bucket counts
                let dollars = lvr_cents as f64 / 100.0;
                let bucket_idx = match dollars {
                    x if x == 0.0 => 0,
                    x if x <= 10.0 => 1,
                    x if x <= 100.0 => 2,
                    x if x <= 500.0 => 3,
                    x if x <= 1000.0 => 4,
                    x if x <= 10000.0 => 5,
                    _ => 6,
                };
                bucket_counts[bucket_idx] += 1;
            } else {
                // Count zero values
                bucket_counts[0] += 1;
            }
        }
    
        if updates > 0 {
            // Update max LVR
            checkpoint.update_max_lvr(max_lvr_block, max_lvr);
            
            // Update running total
            checkpoint.running_total.fetch_add(running_total, Ordering::Release);
    
            // Update bucket counts atomically
            let bucket_refs = [
                &checkpoint.total_bucket_0,
                &checkpoint.total_bucket_0_10,
                &checkpoint.total_bucket_10_100,
                &checkpoint.total_bucket_100_500,
                &checkpoint.total_bucket_500_1000,
                &checkpoint.total_bucket_1000_10000,
                &checkpoint.total_bucket_10000_plus,
            ];
    
            for (count, bucket) in bucket_counts.iter().zip(bucket_refs.iter()) {
                bucket.fetch_add(*count, Ordering::Release);
            }
    
            // Update TDigest with non-zero values
            if let Ok(mut digest) = checkpoint.digest.lock() {
                for value in non_zero_values {
                    digest.add(value);
                }
            }
    
            // Update last processed block
            checkpoint.last_updated_block.fetch_max(chunk_end - 1, Ordering::Release);
        }
    
        Ok(())
    }

    fn calculate_interval_metrics(
        &self,
        chunk_start: u64,
        chunk_end: u64,
        pool_address: &str,
        markout_time: MarkoutTime,
        data: &[UnifiedLVRData],
    ) -> Result<Vec<IntervalData>> {
        let blocks_per_interval = BLOCKS_PER_DAY;
        let deployment_block = self.get_deployment_block(pool_address);
    
        // Adjust chunk boundaries based on deployment block
        let effective_chunk_start = chunk_start.max(deployment_block);
        
        // Early return if chunk is entirely before deployment or empty
        if effective_chunk_start >= chunk_end {
            return Ok(Vec::new());
        }
    
        // Create map to store data for each block
        let block_data: DashMap<u64, u64> = DashMap::new();
        
        // Map all available data points within effective range
        data.iter()
            .filter(|d| d.block_number >= effective_chunk_start && d.block_number < chunk_end)
            .for_each(|data_point| {
                block_data.insert(data_point.block_number, data_point.lvr_cents);
            });
    
        // Create interval groups with explicit zero handling
        let interval_groups: DashMap<u64, Vec<(u64, u64)>> = DashMap::new();
        
        // Process each block in range, mapping to intervals and tracking block numbers
        for block_number in effective_chunk_start..chunk_end {
            let interval_id = (block_number - chunk_start) / blocks_per_interval;
            let value = block_data.get(&block_number).map(|v| *v).unwrap_or(0);
            
            interval_groups
                .entry(interval_id)
                .and_modify(|v| v.push((block_number, value)))
                .or_insert_with(|| vec![(block_number, value)]);
        }
    
        // Calculate metrics for each interval
        let result: Vec<_> = interval_groups
            .into_iter()
            .map(|(interval_id, blocks)| {
                // Calculate interval boundaries
                let interval_start = chunk_start + (interval_id * blocks_per_interval);
                let interval_end = (interval_start + blocks_per_interval).min(chunk_end);
                
                // Calculate effective range for this interval
                let effective_interval_start = interval_start.max(deployment_block);
                
                // Count total blocks in effective range
                let total_count = if effective_interval_start >= interval_end {
                    0
                } else {
                    // Only count blocks after deployment
                    blocks.iter()
                        .filter(|(block_number, _)| *block_number >= effective_interval_start)
                        .count() as u64
                };
    
                // Count non-zero values in effective range
                let non_zero_values: Vec<_> = blocks.iter()
                    .filter(|(block_number, value)| {
                        *block_number >= effective_interval_start && *value > 0
                    })
                    .map(|(_, value)| *value)
                    .collect();
    
                IntervalData {
                    interval_id,
                    pair_address: pool_address.to_string(),
                    markout_time: markout_time.clone(),
                    total_lvr_cents: non_zero_values.iter().sum(),
                    max_lvr_cents: non_zero_values.iter().copied().max().unwrap_or(0),
                    non_zero_count: non_zero_values.len() as u64,
                    total_count,
                }
            })
            .collect();
    
        Ok(result)
    }

    pub async fn run_precomputation(&self) -> Result<()> {
        info!("Starting precomputation phase...");
        
        let precomputed_writer = PrecomputedWriter::new(self.object_store.clone());
        
        // Run all precomputation methods sequentially
        precomputed_writer.write_running_totals().await?;
        info!("Completed running totals precomputation");
        
        precomputed_writer.write_pool_totals().await?;
        info!("Completed pool totals precomputation");
        
        precomputed_writer.write_max_lvr().await?;
        info!("Completed max LVR precomputation");
        
        precomputed_writer.write_non_zero_proportions().await?;
        info!("Completed non-zero proportions precomputation");
        
        precomputed_writer.write_histograms().await?;
        info!("Completed histograms precomputation");
        
        precomputed_writer.write_percentile_bands().await?;
        info!("Completed percentile bands precomputation");
        
        precomputed_writer.write_quartile_plots().await?;
        info!("Completed quartile plots precomputation");
        
        precomputed_writer.write_daily_time_series().await?;
        info!("Completed daily time series precomputation");
        
        precomputed_writer.write_cluster_proportions().await?;
        info!("Completed cluster proportions precomputation");
        
        precomputed_writer.write_cluster_histograms().await?;
        info!("Completed cluster histograms precomputation");
        
        precomputed_writer.write_monthly_cluster_totals().await?;
        info!("Completed monthly cluster totals precomputation");
        
        precomputed_writer.write_cluster_non_zero().await?;
        info!("Completed cluster non-zero precomputation");
    
        precomputed_writer.write_distribution_metrics().await?;
        info!("Completed distribution metrics precomputation");
    
        info!("Successfully completed all metric precomputations");
        Ok(())
    }
    fn parse_lvr_details(&self, details_str: &str, target_pool_name: &str) -> Option<f64> {
        // Attempt to parse as a vector of vectors of strings
        if let Ok(details) = serde_json::from_str::<Vec<Vec<String>>>(details_str) {
            for entry in details {
                if entry.len() == 2 {
                    let pool_name = &entry[0];
                    let value_str = &entry[1];
    
                    if pool_name == target_pool_name {
                        // Parse value_str as JSON to extract 'dollarValue'
                        if let Ok(detail) = serde_json::from_str::<HashMap<String, serde_json::Value>>(value_str) {
                            if let Some(dollar_value) = detail.get("dollarValue") {
                                return dollar_value.as_f64();
                            }
                        }
                        // Fall back to parsing value_str as a float
                        if let Ok(value) = value_str.parse::<f64>() {
                            return Some(value);
                        }
                    }
                }
            }
        } else {
            // Log the parsing error for debugging
            error!("Failed to parse details_str as Vec<Vec<String>>");
        }
    
        None
    }
}