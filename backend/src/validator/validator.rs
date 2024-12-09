use anyhow::{Context, Result};
use object_store::ObjectStore;
use parquet::arrow::arrow_reader::ParquetRecordBatchReader;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn, error};
use futures::StreamExt;

const BATCH_SIZE: usize = 1024;

#[derive(Debug)]
pub struct ValidationStats {
    pub checkpoint_total: u64,
    pub intervals_total: u64,
    pub difference: u64,
    pub difference_percent: f64,
    pub checkpoint_zero_count: u64,
    pub interval_zero_count: u64,
    pub checkpoint_non_zero_ratio: f64,
    pub interval_non_zero_ratio: f64,
}

pub struct Validator {
    object_store: Arc<dyn ObjectStore>,
}

#[derive(Debug)]
struct CheckpointData {
    running_total: u64,
    zero_count: u64,
    total_count: u64,
}

#[derive(Debug, Default, Clone)]
struct IntervalValidationData {
    total_lvr: u64,
    zero_count: u64,
    total_count: u64,
}

impl Validator {
    pub fn new(object_store: Arc<dyn ObjectStore>) -> Self {
        Self { object_store }
    }

    pub async fn validate_all(&self) -> Result<HashMap<String, ValidationStats>> {
        // Get checkpoint and interval data
        let checkpoint_data = self.load_checkpoint_data().await?;
        let interval_data = self.load_interval_data().await?;
        
        // Compare and generate validation stats
        let mut results = HashMap::new();
        
        for (key, checkpoint) in checkpoint_data {
            let interval = interval_data.get(&key).cloned().unwrap_or_default();
            
            let checkpoint_non_zero_ratio = if checkpoint.total_count > 0 {
                (checkpoint.total_count - checkpoint.zero_count) as f64 / checkpoint.total_count as f64
            } else {
                0.0
            };

            let interval_non_zero_ratio = if interval.total_count > 0 {
                (interval.total_count - interval.zero_count) as f64 / interval.total_count as f64
            } else {
                0.0
            };

            let difference = checkpoint.running_total.saturating_sub(interval.total_lvr);
            let difference_percent = if checkpoint.running_total > 0 {
                (difference as f64 / checkpoint.running_total as f64) * 100.0
            } else {
                0.0
            };

            let stats = ValidationStats {
                checkpoint_total: checkpoint.running_total,
                intervals_total: interval.total_lvr,
                difference,
                difference_percent,
                checkpoint_zero_count: checkpoint.zero_count,
                interval_zero_count: interval.zero_count,
                checkpoint_non_zero_ratio,
                interval_non_zero_ratio,
            };

            self.log_validation_results(&key, &stats);
            results.insert(key, stats);
        }

        Ok(results)
    }

    async fn load_checkpoint_data(&self) -> Result<HashMap<String, CheckpointData>> {
        let mut checkpoint_data = HashMap::new();
        let checkpoint_prefix = object_store::path::Path::from("checkpoints");
        let mut checkpoint_files = self.object_store.list(Some(&checkpoint_prefix));

        while let Some(meta) = checkpoint_files.next().await {
            let meta = meta?;
            let bytes = self.object_store.get(&meta.location).await?.bytes().await?;
            let reader = ParquetRecordBatchReader::try_new(bytes, BATCH_SIZE)?;

            for batch in reader {
                let batch = batch?;
                let data = self.extract_checkpoint_batch_data(&batch)?;
                checkpoint_data.insert(data.0, data.1);
            }
        }

        Ok(checkpoint_data)
    }

    async fn load_interval_data(&self) -> Result<HashMap<String, IntervalValidationData>> {
        let mut interval_data = HashMap::new();
        let intervals_prefix = object_store::path::Path::from("intervals");
        let mut interval_files = self.object_store.list(Some(&intervals_prefix));

        while let Some(meta) = interval_files.next().await {
            let meta = meta?;
            let bytes = self.object_store.get(&meta.location).await?.bytes().await?;
            let reader = ParquetRecordBatchReader::try_new(bytes, BATCH_SIZE)?;

            for batch in reader {
                let batch = batch?;
                self.process_interval_batch(&batch, &mut interval_data)?;
            }
        }

        Ok(interval_data)
    }

    fn extract_checkpoint_batch_data(&self, batch: &arrow::record_batch::RecordBatch) 
        -> Result<(String, CheckpointData)> {
        let pair_address = batch
            .column(batch.schema().index_of("pair_address")?)
            .as_any()
            .downcast_ref::<arrow::array::StringArray>()
            .context("Failed to get pair_address column")?
            .value(0);

        let markout_time = batch
            .column(batch.schema().index_of("markout_time")?)
            .as_any()
            .downcast_ref::<arrow::array::StringArray>()
            .context("Failed to get markout_time column")?
            .value(0);

        let running_total = batch
            .column(batch.schema().index_of("running_total")?)
            .as_any()
            .downcast_ref::<arrow::array::UInt64Array>()
            .context("Failed to get running_total column")?
            .value(0);

        let zero_count = batch
            .column(batch.schema().index_of("total_bucket_0")?)
            .as_any()
            .downcast_ref::<arrow::array::UInt64Array>()
            .context("Failed to get total_bucket_0 column")?
            .value(0);

        let total_count = self.get_total_bucket_count(batch)?;

        Ok((
            format!("{}_{}", pair_address, markout_time),
            CheckpointData {
                running_total,
                zero_count,
                total_count,
            },
        ))
    }

    fn get_total_bucket_count(&self, batch: &arrow::record_batch::RecordBatch) -> Result<u64> {
        let mut total = 0u64;
        let bucket_names = [
            "total_bucket_0",
            "total_bucket_0_10",
            "total_bucket_10_100",
            "total_bucket_100_500",
            "total_bucket_1000_3000",
            "total_bucket_3000_10000",
            "total_bucket_10000_30000",
            "total_bucket_30000_plus",
        ];

        for name in bucket_names {
            let count = batch
                .column(batch.schema().index_of(name)?)
                .as_any()
                .downcast_ref::<arrow::array::UInt64Array>()
                .context(format!("Failed to get {} column", name))?
                .value(0);
            total += count;
        }

        Ok(total)
    }

    fn process_interval_batch(
        &self,
        batch: &arrow::record_batch::RecordBatch,
        interval_data: &mut HashMap<String, IntervalValidationData>,
    ) -> Result<()> {
        let pair_addresses = batch
            .column(batch.schema().index_of("pair_address")?)
            .as_any()
            .downcast_ref::<arrow::array::StringArray>()
            .context("Failed to get pair_address column")?;

        let markout_times = batch
            .column(batch.schema().index_of("markout_time")?)
            .as_any()
            .downcast_ref::<arrow::array::StringArray>()
            .context("Failed to get markout_time column")?;

        let total_lvr_cents = batch
            .column(batch.schema().index_of("total_lvr_cents")?)
            .as_any()
            .downcast_ref::<arrow::array::UInt64Array>()
            .context("Failed to get total_lvr_cents column")?;

        let total_counts = batch
            .column(batch.schema().index_of("total_count")?)
            .as_any()
            .downcast_ref::<arrow::array::UInt64Array>()
            .context("Failed to get total_count column")?;

        let non_zero_counts = batch
            .column(batch.schema().index_of("non_zero_count")?)
            .as_any()
            .downcast_ref::<arrow::array::UInt64Array>()
            .context("Failed to get non_zero_count column")?;

        for i in 0..batch.num_rows() {
            let key = format!("{}_{}", pair_addresses.value(i), markout_times.value(i));
            let data = interval_data.entry(key).or_default();
            
            data.total_lvr += total_lvr_cents.value(i);
            data.total_count += total_counts.value(i);
            let non_zero_count = non_zero_counts.value(i);
            data.zero_count += total_counts.value(i) - non_zero_count;
        }

        Ok(())
    }

    fn log_validation_results(&self, key: &str, stats: &ValidationStats) {
        let zero_count_difference = if stats.checkpoint_zero_count >= stats.interval_zero_count {
            stats.checkpoint_zero_count - stats.interval_zero_count
        } else {
            stats.interval_zero_count - stats.checkpoint_zero_count
        };

        if stats.difference != 0 || zero_count_difference != 0 {
            if stats.difference_percent.abs() > 1.0 {
                error!(
                    "Significant discrepancy for {}: Checkpoint total: {}, Intervals total: {}, \
                     Difference: {} ({:.2}%), Zero counts: {} vs {}, Non-zero ratio: {:.2}% vs {:.2}%",
                    key, stats.checkpoint_total, stats.intervals_total, stats.difference, 
                    stats.difference_percent, stats.checkpoint_zero_count, stats.interval_zero_count,
                    stats.checkpoint_non_zero_ratio * 100.0, stats.interval_non_zero_ratio * 100.0
                );
            } else {
                warn!(
                    "Minor discrepancy for {}: Checkpoint total: {}, Intervals total: {}, \
                     Difference: {} ({:.2}%), Zero counts: {} vs {}, Non-zero ratio: {:.2}% vs {:.2}%",
                    key, stats.checkpoint_total, stats.intervals_total, stats.difference,
                    stats.difference_percent, stats.checkpoint_zero_count, stats.interval_zero_count,
                    stats.checkpoint_non_zero_ratio * 100.0, stats.interval_non_zero_ratio * 100.0
                );
            }
        } else {
            info!(
                "Validation passed for {}: Total {}, Zero count: {}, Non-zero ratio: {:.2}% vs {:.2}%",
                key, stats.checkpoint_total, stats.checkpoint_zero_count,
                stats.checkpoint_non_zero_ratio * 100.0, stats.interval_non_zero_ratio * 100.0
            );
        }
    }
}