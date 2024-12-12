use arrow::{
    array::{ArrayRef, StringArray, UInt64Array, Float64Array},
    record_batch::RecordBatch,
};
use object_store::{path::Path, ObjectStore};
use parquet::{
    arrow::ArrowWriter,
    basic::Compression,
    file::properties::WriterProperties,
};
use std::sync::Arc;
use tokio::sync::Semaphore;
use anyhow::{Result, Context};
use bytes::Bytes;
use futures::stream::{FuturesOrdered, StreamExt};
use crate::models::{IntervalData, CheckpointSnapshot};
use tracing::{warn, error};

const MAX_CONCURRENT_WRITES: usize = 8;

pub struct ParallelParquetWriter {
    write_semaphore: Arc<Semaphore>,
    object_store: Arc<dyn ObjectStore>,
    max_retries: u32,
}

impl ParallelParquetWriter {
    pub fn new(object_store: Arc<dyn ObjectStore>) -> Self {
        Self {
            write_semaphore: Arc::new(Semaphore::new(MAX_CONCURRENT_WRITES)),
            object_store,
            max_retries: 20,
        }
    }

    // Path construction helpers
    fn get_interval_path(&self, chunk_start: u64, chunk_end: u64) -> Path {
        Path::from(format!("intervals/{}_{}.parquet", chunk_start, chunk_end))
    }

    fn get_checkpoint_path(&self, pair_address: &str, markout_time: &str) -> Path {
        Path::from(format!(
            "checkpoints/{}_{}.parquet",
            pair_address,
            markout_time
        ))
    }

    pub async fn write_interval_data(
        &mut self,
        mut interval_data: Vec<IntervalData>,
        chunk_start: u64,
        chunk_end: u64,
    ) -> Result<()> {
        let _permit = self.write_semaphore.acquire().await?;
    
        if interval_data.is_empty() {
            warn!("No interval data to write for chunk {}-{}", chunk_start, chunk_end);
            return Ok(());
        }
    
        // Sort interval data by interval_id
        interval_data.sort_by_key(|data| data.interval_id);
    
        // Create a single batch for all data
        let batch = create_record_batch_from_interval_data(interval_data)?;
        let store = self.object_store.clone();
        let path = self.get_interval_path(chunk_start, chunk_end);
        
        // Single write operation
        write_batch_to_store(store, path, batch, self.max_retries).await?;
    
        Ok(())
    }

    pub async fn write_checkpoints(
        &mut self,
        checkpoints: Vec<CheckpointSnapshot>
    ) -> Result<()> {
        let _permit = self.write_semaphore.acquire().await?;
    
        // Create new FuturesOrdered for this batch of checkpoints
        let mut checkpoint_tasks = FuturesOrdered::new();
    
        // Process checkpoints in parallel
        for checkpoint in checkpoints {
            let store = self.object_store.clone();
            let path = self.get_checkpoint_path(&checkpoint.pair_address, &checkpoint.markout_time.to_string());
            
            let task = tokio::spawn(async move {
                let batch = create_record_batch_from_checkpoint(&checkpoint)?;
                write_batch_to_store(store, path, batch, 3).await
            });
    
            checkpoint_tasks.push_back(task);
        }
    
        // Wait for all checkpoint writes to complete
        while let Some(result) = checkpoint_tasks.next().await {
            match result {
                Ok(Ok(_)) => continue,
                Ok(Err(e)) => {
                    error!("Checkpoint write failed: {}", e);
                    return Err(e);
                }
                Err(e) => {
                    error!("Task join failed: {}", e);
                    return Err(anyhow::anyhow!("Task join failed: {}", e));
                }
            }
        }
    
        Ok(())
    }
}

// Helper functions
async fn write_batch_to_store(
    store: Arc<dyn ObjectStore>,
    path: Path,
    batch: RecordBatch,
    max_retries: u32,
) -> Result<()> {
    let props = WriterProperties::builder()
        .set_compression(Compression::SNAPPY)
        .set_write_batch_size(1024 * 1024)
        .set_data_page_size_limit(1024 * 1024)
        .build();

    let mut buffer = Vec::new();
    {
        let mut writer = ArrowWriter::try_new(&mut buffer, batch.schema(), Some(props))?;
        writer.write(&batch)?;
        writer.close()?;
    }

    let mut retries = 0;
    while retries < max_retries {
        match store.put(&path, Bytes::from(buffer.clone()).into()).await {
            Ok(_) => return Ok(()),
            Err(e) if retries < max_retries - 1 => {
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
    
    Err(anyhow::anyhow!("Failed to write after {} retries", max_retries))
}

fn create_record_batch_from_interval_data(data: Vec<IntervalData>) -> Result<RecordBatch> {
    RecordBatch::try_from_iter([
        ("interval_id", Arc::new(UInt64Array::from(data.iter().map(|d| d.interval_id).collect::<Vec<_>>())) as ArrayRef),
        ("pair_address", Arc::new(StringArray::from(data.iter().map(|d| d.pair_address.clone()).collect::<Vec<_>>())) as ArrayRef),
        ("markout_time", Arc::new(StringArray::from(data.iter().map(|d| d.markout_time.to_string()).collect::<Vec<_>>())) as ArrayRef),
        ("total_lvr_cents", Arc::new(UInt64Array::from(data.iter().map(|d| d.total_lvr_cents).collect::<Vec<_>>())) as ArrayRef),
        ("median_lvr_cents", Arc::new(UInt64Array::from(data.iter().map(|d| d.median_lvr_cents).collect::<Vec<_>>())) as ArrayRef),
        ("percentile_25_cents", Arc::new(UInt64Array::from(data.iter().map(|d| d.percentile_25_cents).collect::<Vec<_>>())) as ArrayRef),
        ("percentile_75_cents", Arc::new(UInt64Array::from(data.iter().map(|d| d.percentile_75_cents).collect::<Vec<_>>())) as ArrayRef),
        ("max_lvr_cents", Arc::new(UInt64Array::from(data.iter().map(|d| d.max_lvr_cents).collect::<Vec<_>>())) as ArrayRef),
        ("non_zero_count", Arc::new(UInt64Array::from(data.iter().map(|d| d.non_zero_count).collect::<Vec<_>>())) as ArrayRef),
        ("total_count", Arc::new(UInt64Array::from(data.iter().map(|d| d.total_count).collect::<Vec<_>>())) as ArrayRef),
    ]).context("Failed to create interval data record batch")
}

fn create_record_batch_from_checkpoint(checkpoint: &CheckpointSnapshot) -> Result<RecordBatch> {
    RecordBatch::try_from_iter([
        ("pair_address", Arc::new(StringArray::from(vec![checkpoint.pair_address.clone()])) as ArrayRef),
        ("markout_time", Arc::new(StringArray::from(vec![checkpoint.markout_time.to_string()])) as ArrayRef),
        ("max_lvr_block", Arc::new(UInt64Array::from(vec![checkpoint.max_lvr_block])) as ArrayRef),
        ("max_lvr_value", Arc::new(UInt64Array::from(vec![checkpoint.max_lvr_value])) as ArrayRef),
        ("running_total", Arc::new(UInt64Array::from(vec![checkpoint.running_total])) as ArrayRef),
        ("total_bucket_0", Arc::new(UInt64Array::from(vec![checkpoint.total_bucket_0])) as ArrayRef),
        ("total_bucket_0_10", Arc::new(UInt64Array::from(vec![checkpoint.total_bucket_0_10])) as ArrayRef),
        ("total_bucket_10_100", Arc::new(UInt64Array::from(vec![checkpoint.total_bucket_10_100])) as ArrayRef),
        ("total_bucket_100_500", Arc::new(UInt64Array::from(vec![checkpoint.total_bucket_100_500])) as ArrayRef),
        ("total_bucket_1000_3000", Arc::new(UInt64Array::from(vec![checkpoint.total_bucket_1000_3000])) as ArrayRef),
        ("total_bucket_3000_10000", Arc::new(UInt64Array::from(vec![checkpoint.total_bucket_3000_10000])) as ArrayRef),
        ("total_bucket_10000_30000", Arc::new(UInt64Array::from(vec![checkpoint.total_bucket_10000_30000])) as ArrayRef),
        ("total_bucket_30000_plus", Arc::new(UInt64Array::from(vec![checkpoint.total_bucket_30000_plus])) as ArrayRef),
        ("last_updated_block", Arc::new(UInt64Array::from(vec![checkpoint.last_updated_block])) as ArrayRef),
        ("non_zero_proportion", Arc::new(Float64Array::from(vec![checkpoint.non_zero_proportion])) as ArrayRef),
    ]).context("Failed to create checkpoint record batch")
}