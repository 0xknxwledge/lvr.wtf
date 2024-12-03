use anyhow::{Context, Result};
use object_store::ObjectStore;
use parquet::arrow::arrow_reader::ParquetRecordBatchReader;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn, error};
use futures::StreamExt;
use bytes::Bytes;

const BATCH_SIZE: usize = 1024;

pub struct ValidationStats {
    pub checkpoint_total: u64,
    pub intervals_total: u64,
    pub difference: u64,
    pub difference_percent: f64,
}

pub struct Validator {
    object_store: Arc<dyn ObjectStore>,
}

impl Validator {
    pub fn new(object_store: Arc<dyn ObjectStore>) -> Self {
        Self { object_store }
    }

    pub async fn validate_all(&self) -> Result<HashMap<String, ValidationStats>> {
        let mut results = HashMap::new();
        
        // List all checkpoint files
    let checkpoint_prefix = object_store::path::Path::from("checkpoints");
    let mut checkpoint_files = self.object_store.list(Some(&checkpoint_prefix));
    
    // List all interval files
    let intervals_prefix = object_store::path::Path::from("intervals");
    let mut interval_files = self.object_store.list(Some(&intervals_prefix));
        
        // Load and process all checkpoint files
        let mut checkpoint_totals = HashMap::new();
        while let Some(checkpoint_meta) = checkpoint_files.next().await {
            let checkpoint_meta = checkpoint_meta?;
            let path = checkpoint_meta.location;
            
            // Read the checkpoint file
            let bytes: Bytes = self.object_store.get(&path).await?.bytes().await?;
            let record_reader = ParquetRecordBatchReader::try_new(bytes, BATCH_SIZE)?;
            
            for batch in record_reader {
                let batch = batch?;
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
                
                let key = format!("{}_{}", pair_address, markout_time);
                checkpoint_totals.insert(key, running_total);
            }
        }
        
        // Process interval files and aggregate totals
        let mut interval_totals = HashMap::new();
        while let Some(interval_meta) = interval_files.next().await {
            let interval_meta = interval_meta?;
            let path = interval_meta.location;
            
            // Read the interval file
            let bytes: Bytes = self.object_store.get(&path).await?.bytes().await?;
            let record_reader = ParquetRecordBatchReader::try_new(bytes, BATCH_SIZE)?;
            
            for batch in record_reader {
                let batch = batch?;
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
                
                for i in 0..batch.num_rows() {
                    let key = format!("{}_{}", pair_addresses.value(i), markout_times.value(i));
                    *interval_totals.entry(key).or_insert(0) += total_lvr_cents.value(i);
                }
            }
        }
        
        // Compare totals and generate validation stats
        for (key, checkpoint_total) in checkpoint_totals {
            let intervals_total = *interval_totals.get(&key).unwrap_or(&0);
            let difference = checkpoint_total - intervals_total;
            let difference_percent = if checkpoint_total != 0 {
                (difference as f64 / checkpoint_total as f64) * 100.0
            } else {
                0.0
            };
            
            let stats = ValidationStats {
                checkpoint_total,
                intervals_total,
                difference,
                difference_percent,
            };
            
            // Log validation results
            if difference != 0 {
                if difference_percent.abs() > 1.0 {
                    error!(
                        "Significant discrepancy for {}: Checkpoint total: {}, Intervals total: {}, Difference: {} ({:.2}%)",
                        key, checkpoint_total, intervals_total, difference, difference_percent
                    );
                } else {
                    warn!(
                        "Minor discrepancy for {}: Checkpoint total: {}, Intervals total: {}, Difference: {} ({:.2}%)",
                        key, checkpoint_total, intervals_total, difference, difference_percent
                    );
                }
            } else {
                info!("Validation passed for {}: Total {}", key, checkpoint_total);
            }
            
            results.insert(key, stats);
        }
        
        Ok(results)
    }
}