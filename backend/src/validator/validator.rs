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
    pub tdigest_samples: u64,
    pub non_zero_samples: u64,
    pub bucket_sum_non_zero: u64,
    pub sample_count_match: bool,
    pub non_zero_counts_consistent: bool,
}

pub struct Validator {
    object_store: Arc<dyn ObjectStore>,
}

#[derive(Debug)]
struct CheckpointData {
    running_total: u64,
    zero_count: u64,
    total_count: u64,
    exact_samples: u64,
    non_zero_bucket_sum: u64,
}

#[derive(Debug, Default, Clone)]
struct IntervalValidationData {
    total_lvr: u64,
    non_zero_count: u64,
    total_count: u64,
}

impl Validator {
    pub fn new(object_store: Arc<dyn ObjectStore>) -> Self {
        Self { object_store }
    }

    pub async fn validate_all(&self) -> Result<HashMap<String, ValidationStats>> {
        let checkpoint_data = self.load_checkpoint_data().await?;
        let interval_data = self.load_interval_data().await?;
        
        let mut results = HashMap::new();
        
        for (key, checkpoint) in checkpoint_data {
            let interval = interval_data.get(&key).cloned().unwrap_or_default();
            
            let checkpoint_non_zero_ratio = if checkpoint.total_count > 0 {
                (checkpoint.total_count - checkpoint.zero_count) as f64 / checkpoint.total_count as f64
            } else {
                0.0
            };

            let interval_zero_count = interval.total_count.saturating_sub(interval.non_zero_count);
            let interval_non_zero_ratio = if interval.total_count > 0 {
                interval.non_zero_count as f64 / interval.total_count as f64
            } else {
                0.0
            };

            let difference = checkpoint.running_total.saturating_sub(interval.total_lvr);
            let difference_percent = if checkpoint.running_total > 0 {
                (difference as f64 / checkpoint.running_total as f64) * 100.0
            } else {
                0.0
            };

            // Check consistency between different non-zero count sources
            let sample_count_match = checkpoint.exact_samples == interval.non_zero_count;
            let non_zero_counts_consistent = checkpoint.exact_samples == checkpoint.non_zero_bucket_sum &&
                checkpoint.exact_samples == interval.non_zero_count;

            let stats = ValidationStats {
                checkpoint_total: checkpoint.running_total,
                intervals_total: interval.total_lvr,
                difference,
                difference_percent,
                checkpoint_zero_count: checkpoint.zero_count,
                interval_zero_count,
                checkpoint_non_zero_ratio,
                interval_non_zero_ratio,
                tdigest_samples: checkpoint.exact_samples,
                non_zero_samples: interval.non_zero_count,
                bucket_sum_non_zero: checkpoint.non_zero_bucket_sum,
                sample_count_match,
                non_zero_counts_consistent,
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

        let exact_samples = batch
            .column(batch.schema().index_of("non_zero_samples")?)
            .as_any()
            .downcast_ref::<arrow::array::UInt64Array>()
            .context("Failed to get non_zero_samples count")?
            .value(0);

        // Calculate total count and non-zero bucket sum
        let (total_count, non_zero_bucket_sum) = self.get_bucket_counts(batch)?;

        Ok((
            format!("{}_{}", pair_address, markout_time),
            CheckpointData {
                running_total,
                zero_count,
                total_count,
                exact_samples,
                non_zero_bucket_sum,
            },
        ))
    }

    fn get_bucket_counts(&self, batch: &arrow::record_batch::RecordBatch) -> Result<(u64, u64)> {
        let mut total_count = 0u64;
        let mut non_zero_sum = 0u64;
        
        let bucket_names = [
            "total_bucket_0",
            "total_bucket_0_10",
            "total_bucket_10_100",
            "total_bucket_100_500",
            "total_bucket_500_1000",
            "total_bucket_1000_10000",
            "total_bucket_10000_plus",
        ];

        for (idx, name) in bucket_names.iter().enumerate() {
            let count = batch
                .column(batch.schema().index_of(name)?)
                .as_any()
                .downcast_ref::<arrow::array::UInt64Array>()
                .context(format!("Failed to get {} column", name))?
                .value(0);
            
            total_count += count;
            if idx > 0 {  // Skip zero bucket when summing non-zero counts
                non_zero_sum += count;
            }
        }

        Ok((total_count, non_zero_sum))
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
            data.non_zero_count += non_zero_counts.value(i);
        }

        Ok(())
    }

    fn log_validation_results(&self, key: &str, stats: &ValidationStats) {
        let mut errors = Vec::new();
        
        // Check for non-zero count inconsistencies
        if !stats.non_zero_counts_consistent {
            errors.push(format!(
                "Non-zero count mismatch: TDigest={}, Intervals={}, Bucket sum={}", 
                stats.tdigest_samples, 
                stats.non_zero_samples, 
                stats.bucket_sum_non_zero
            ));
        }
    
        // Check for total value discrepancies
        if stats.difference != 0 {
            errors.push(format!(
                "Total mismatch: Checkpoint={}, Intervals={}, Difference={}({:.2}%)", 
                stats.checkpoint_total, 
                stats.intervals_total, 
                stats.difference, 
                stats.difference_percent
            ));
        }
    
        // Check for zero count discrepancies
        let zero_count_difference = (stats.checkpoint_zero_count as i64 - stats.interval_zero_count as i64).abs();
        if zero_count_difference != 0 {
            errors.push(format!(
                "Zero count mismatch: Checkpoint={}, Intervals={}, Difference={}", 
                stats.checkpoint_zero_count, 
                stats.interval_zero_count, 
                zero_count_difference
            ));
        }
    
        // Log appropriate message based on validation results
        if errors.is_empty() {
            info!(
                "Validation passed for {}: Total {}, Non-zero counts consistent ({} samples), Zero count: {}", 
                key, 
                stats.checkpoint_total, 
                stats.tdigest_samples, 
                stats.checkpoint_zero_count
            );
        } else {
            // Determine if discrepancies are significant
            let has_significant_errors = stats.difference_percent.abs() > 1.0 || !stats.non_zero_counts_consistent;
            
            if has_significant_errors {
                error!(
                    "Significant discrepancies for {}:\n{}", 
                    key,
                    errors.join("\n")
                );
            } else {
                warn!(
                    "Minor discrepancies for {}:\n{}", 
                    key,
                    errors.join("\n")
                );
            }
        }
    }
}