use serde::{Serialize, Deserialize};
use std::sync::atomic::{AtomicU64, AtomicI64};
use std::fmt;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use num_traits::cast::ToPrimitive;
use crate::tdigest::*;

#[derive(Debug, Clone)]
pub struct UnifiedLVRData {
    pub block_number: u64,
    pub lvr_cents: u64,
    pub source: DataSource,
}

#[derive(Debug, Clone)]
pub struct ClusterBlockActivity {
    pub cluster_name: String,
    pub markout_time: MarkoutTime,
    pub total_blocks: u64,
    pub non_zero_blocks: u64,
}

impl ClusterBlockActivity {
    pub fn new(cluster_name: String, markout_time: MarkoutTime) -> Self {
        Self {
            cluster_name,
            markout_time,
            total_blocks: 0,
            non_zero_blocks: 0,
        }
    }
    
    pub fn increment_total(&mut self) {
        self.total_blocks += 1;
    }
    
    pub fn increment_non_zero(&mut self) {
        self.non_zero_blocks += 1;
    }
    
    pub fn get_proportion(&self) -> f64 {
        if self.total_blocks > 0 {
            self.non_zero_blocks as f64 / self.total_blocks as f64
        } else {
            0.0
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DataSource {
    Aurora,
    Brontes,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum MarkoutTime {
    Negative2,
    Negative15,
    Negative1,
    Negative05,
    Zero,
    Positive05,
    Positive1,
    Positive15,
    Positive2,
    Brontes,
}

impl fmt::Display for MarkoutTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MarkoutTime::Negative2 => write!(f, "-2.0"),
            MarkoutTime::Negative15 => write!(f, "-1.5"), 
            MarkoutTime::Negative1 => write!(f, "-1.0"),
            MarkoutTime::Negative05 => write!(f, "-0.5"),
            MarkoutTime::Zero => write!(f, "0.0"),
            MarkoutTime::Positive05 => write!(f, "0.5"),
            MarkoutTime::Positive1 => write!(f, "1.0"),
            MarkoutTime::Positive15 => write!(f, "1.5"),
            MarkoutTime::Positive2 => write!(f, "2.0"),
            MarkoutTime::Brontes => write!(f, "brontes"),
        }
    }
}

impl MarkoutTime {
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            MarkoutTime::Negative2 => Some(-2.0),
            MarkoutTime::Negative15 => Some(-1.5),
            MarkoutTime::Negative1 => Some(-1.0),
            MarkoutTime::Negative05 => Some(-0.5),
            MarkoutTime::Zero => Some(0.0),
            MarkoutTime::Positive05 => Some(0.5),
            MarkoutTime::Positive1 => Some(1.0),
            MarkoutTime::Positive15 => Some(1.5),
            MarkoutTime::Positive2 => Some(2.0),
            MarkoutTime::Brontes => None,
        }
    }

    pub fn from_f64(value: f64) -> Option<Self> {
        const EPSILON: f64 = 1e-10;
        
        if (value + 2.0).abs() < EPSILON {
            Some(Self::Negative2)
        } else if (value + 1.5).abs() < EPSILON {
            Some(Self::Negative15)
        } else if (value + 1.0).abs() < EPSILON {
            Some(Self::Negative1)
        } else if (value + 0.5).abs() < EPSILON {
            Some(Self::Negative05)
        } else if value.abs() < EPSILON {
            Some(Self::Zero)
        } else if (value - 0.5).abs() < EPSILON {
            Some(Self::Positive05)
        } else if (value - 1.0).abs() < EPSILON {
            Some(Self::Positive1)
        } else if (value - 1.5).abs() < EPSILON {
            Some(Self::Positive15)
        } else if (value - 2.0).abs() < EPSILON {
            Some(Self::Positive2)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct MaxLVRData {
    pub value: u64,
    pub block: u64,
}

#[derive(Debug)]
pub struct Checkpoint {
    pub pair_address: String,
    pub markout_time: MarkoutTime,
    pub max_lvr: Arc<Mutex<MaxLVRData>>,
    pub running_total: AtomicI64,
    pub total_bucket_0: AtomicU64,        
    pub total_bucket_0_10: AtomicU64,     
    pub total_bucket_10_100: AtomicU64,   
    pub total_bucket_100_500: AtomicU64,  
    pub total_bucket_500_1000: AtomicU64, 
    pub total_bucket_1000_10000: AtomicU64, 
    pub total_bucket_10000_plus: AtomicU64, 
    pub last_updated_block: AtomicU64,
    pub digest: Arc<Mutex<TDigest>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointSnapshot {
    pub pair_address: String,
    pub markout_time: MarkoutTime,
    pub max_lvr_value: u64,
    pub max_lvr_block: u64,
    pub running_total: u64,
    pub total_bucket_0: u64,           
    pub total_bucket_0_10: u64,       
    pub total_bucket_10_100: u64,      
    pub total_bucket_100_500: u64,     
    pub total_bucket_500_1000: u64,   
    pub total_bucket_1000_10000: u64,  
    pub total_bucket_10000_plus: u64,  
    pub last_updated_block: u64,
    pub non_zero_proportion: f64,
    pub percentile_25_cents: u64,
    pub median_cents: u64,
    pub percentile_75_cents: u64,
    pub non_zero_samples: u64,
    pub mean: f64,
    pub std_dev: f64,
    pub skewness: f64,
    pub kurtosis: f64,
}

#[derive(Debug)]
pub struct CheckpointUpdate {
    pub pool_address: String,
    pub markout_time: MarkoutTime,
    pub data: Vec<UnifiedLVRData>,
    pub chunk_start: u64,
    pub chunk_end: u64,
}


impl Checkpoint {
    pub fn new(pair_address: String, markout_time: MarkoutTime) -> Self {
        Self {
            pair_address,
            markout_time,
            max_lvr: Arc::new(Mutex::new(MaxLVRData {
                value: 0,
                block: 0,
            })),
            running_total: AtomicI64::new(0),
            total_bucket_0: AtomicU64::new(0),
            total_bucket_0_10: AtomicU64::new(0),
            total_bucket_10_100: AtomicU64::new(0),
            total_bucket_100_500: AtomicU64::new(0),
            total_bucket_500_1000: AtomicU64::new(0),
            total_bucket_1000_10000: AtomicU64::new(0),
            total_bucket_10000_plus: AtomicU64::new(0),
            last_updated_block: AtomicU64::new(0),

            digest: Arc::new(Mutex::new(TDigest::new()))
        }
    }

    pub fn to_snapshot(&self) -> CheckpointSnapshot {
        let max_lvr_data = self.max_lvr.lock().unwrap();
        let digest = self.digest.lock().unwrap();
        
        let total_observations = self.total_bucket_0.load(Ordering::Acquire) +
            self.total_bucket_0_10.load(Ordering::Acquire) +
            self.total_bucket_10_100.load(Ordering::Acquire) +
            self.total_bucket_100_500.load(Ordering::Acquire) +
            self.total_bucket_500_1000.load(Ordering::Acquire) +
            self.total_bucket_1000_10000.load(Ordering::Acquire) +
            self.total_bucket_10000_plus.load(Ordering::Acquire);

        let non_zero_observations = total_observations - self.total_bucket_0.load(Ordering::Acquire);
        
        let non_zero_proportion = if total_observations > 0 {
            non_zero_observations as f64 / total_observations as f64
        } else {
            0.0
        };

        // Calculate percentiles using TDigest
        let p25 = digest.quantile(0.25).map(|x| (x * 100.0).round() as u64).unwrap_or(0);
        let p50 = digest.quantile(0.50).map(|x| (x * 100.0).round() as u64).unwrap_or(0);
        let p75 = digest.quantile(0.75).map(|x| (x * 100.0).round() as u64).unwrap_or(0);

        // Get distribution metrics from TDigest
        let distribution_metrics = digest.online_stats.to_metrics();

        CheckpointSnapshot {
            pair_address: self.pair_address.clone(),
            markout_time: self.markout_time,
            max_lvr_value: max_lvr_data.value,
            max_lvr_block: max_lvr_data.block,
            running_total: self.running_total.load(Ordering::Acquire).to_u64().unwrap(),
            total_bucket_0: self.total_bucket_0.load(Ordering::Acquire),
            total_bucket_0_10: self.total_bucket_0_10.load(Ordering::Acquire),
            total_bucket_10_100: self.total_bucket_10_100.load(Ordering::Acquire),
            total_bucket_100_500: self.total_bucket_100_500.load(Ordering::Acquire),
            total_bucket_500_1000: self.total_bucket_500_1000.load(Ordering::Acquire),
            total_bucket_1000_10000: self.total_bucket_1000_10000.load(Ordering::Acquire),
            total_bucket_10000_plus: self.total_bucket_10000_plus.load(Ordering::Acquire),
            last_updated_block: self.last_updated_block.load(Ordering::Acquire),
            non_zero_proportion,
            percentile_25_cents: p25,
            median_cents: p50,
            percentile_75_cents: p75,
            non_zero_samples: digest.samples(),
            mean: distribution_metrics.mean,
            std_dev: distribution_metrics.std_dev,
            skewness: distribution_metrics.skewness,
            kurtosis: distribution_metrics.kurtosis,
        }
    }
    pub fn update_digest(&self, value: f64) -> Result<(), String> {
        if let Ok(mut digest) = self.digest.lock() {
            digest.add(value);
            Ok(())
        } else {
            Err("Failed to acquire digest lock".to_string())
        }
    }
    
    pub fn update_max_lvr(&self, block_number: u64, lvr_cents: u64) {
        let mut max_lvr = self.max_lvr.lock().unwrap();
        if lvr_cents > max_lvr.value {
            max_lvr.value = lvr_cents;
            max_lvr.block = block_number;
        }
    }

    pub fn finalize(&self) -> Result<(), String> {
        if let Ok(mut digest) = self.digest.lock() {
            digest.finalize();
            Ok(())
        } else {
            Err("Failed to acquire digest lock for finalization".to_string())
        }
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IntervalData {
    pub interval_id: u64,
    pub pair_address: String,
    pub markout_time: MarkoutTime,
    pub total_lvr_cents: u64,     
    pub max_lvr_cents: u64,       
    pub non_zero_count: u64,        
    pub total_count: u64,            
}

impl IntervalData {
    pub fn total_lvr_dollars(&self) -> f64 {
        self.total_lvr_cents as f64 / 100.0
    }

    pub fn non_zero_proportion(&self) -> f64 {
        if self.total_count == 0 {
            0.0
        } else {
            self.non_zero_count as f64 / self.total_count as f64
        }
    }
}