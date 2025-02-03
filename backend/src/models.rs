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

            digest: Arc::new(Mutex::new(TDigest::new(
                200,  // delta_partial
                1000,   // delta_final
                10000, // buffer_capacity
            )))
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

    pub fn merge_digest(&self, other_digest: &TDigest) -> Result<u64, String> {
        let mut digest = self.digest.lock()
            .map_err(|_| "Failed to acquire digest lock".to_string())?;
        
        digest.merge(other_digest);
        Ok(digest.samples())
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

#[derive(Debug)]
pub struct CheckpointStats {
    pub updates: u64,
    pub running_total: u64,
    pub max_lvr: u64,
    pub max_lvr_block: u64,
    pub buckets: [u64; 7],
    pub digest: TDigest
}

impl Default for CheckpointStats {
    fn default() -> Self {
        Self {
            updates: 0,
            running_total: 0,
            max_lvr: 0,
            max_lvr_block: 0,
            buckets: [0; 7],
            digest: TDigest::new(100, 50, 1000)
        }
    }
}

impl CheckpointStats {
    pub fn update(&mut self, data_point: &UnifiedLVRData) {
        self.updates += 1;

        if data_point.lvr_cents == 0 {
            self.buckets[0] += 1;  // Zero bucket
            return;
        }

        self.running_total += data_point.lvr_cents;

        if data_point.lvr_cents > self.max_lvr {
            self.max_lvr = data_point.lvr_cents;
            self.max_lvr_block = data_point.block_number;
        }

        // Convert cents to dollars for bucketing and TDigest
        let abs_dollars = (data_point.lvr_cents as f64 / 100.0).abs();

        if data_point.lvr_cents != 0 {
            // Update TDigest with the non-zero dollar value
            self.digest.add(abs_dollars);
        }
        // Update bucket counts
        let bucket_idx = match abs_dollars {
            x if x == 0.0 => 0,          // $0
            x if x <= 10.0 => 1,         // $0-10
            x if x <= 100.0 => 2,        // $10-100
            x if x <= 500.0 => 3,        // $100-500
            x if x <= 1000.0 => 4,       // $500-1000
            x if x <= 10000.0 => 5,      // $1000-10000
            _ => 6,                      // $10000+
        };
        self.buckets[bucket_idx] += 1;
    }

    pub fn merge(&mut self, other: CheckpointStats) {
        // Update basic counters
        self.updates += other.updates;
        self.running_total += other.running_total;
        
        // Update max LVR if the other stats has a higher value
        if other.max_lvr > self.max_lvr {
            self.max_lvr = other.max_lvr;
            self.max_lvr_block = other.max_lvr_block;
        }

        // Merge bucket counts
        for (self_bucket, other_bucket) in self.buckets.iter_mut().zip(other.buckets.iter()) {
            *self_bucket += other_bucket;
        }

        // Merge TDigests while maintaining accurate counts
        let (merged_centroids, total_weight) = TDigest::merge_sorted_centroids(
            &self.digest.centroids,
            &other.digest.centroids
        );

        // Perform stratified merge using the merged centroids
        self.digest.centroids = self.digest.stratified_merge(
            merged_centroids, 
            self.digest.delta_partial
        );

        // Update total weight tracking
        self.digest.total_weight = total_weight;
        
        // Update exact sample count by combining both digests
        self.digest.exact_samples += other.digest.exact_samples;
    }


    pub fn get_percentiles(&mut self) -> (u64, u64, u64) {
        // Finalize the digest before computing percentiles
        self.digest.finalize();
        
        // Convert the dollar values back to cents
        let p25 = (self.digest.quantile(0.25).unwrap() * 100.0).round() as u64;
        let p50 = (self.digest.quantile(0.50).unwrap() * 100.0).round() as u64;
        let p75 = (self.digest.quantile(0.75).unwrap() * 100.0).round() as u64;

        (p25, p50, p75)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markout_time_round_trip() {
        // Test all non-Brontes variants
        let markouts = [ 
            MarkoutTime::Negative2,
            MarkoutTime::Negative15,
            MarkoutTime::Negative1,
            MarkoutTime::Negative05,
            MarkoutTime::Zero,
            MarkoutTime::Positive05,
            MarkoutTime::Positive1,
            MarkoutTime::Positive15,
            MarkoutTime::Positive2,
        ];

        for &original in &markouts {
            // Convert to f64 and back
            let as_f64 = original.as_f64().unwrap();
            let roundtrip = MarkoutTime::from_f64(as_f64).unwrap();
            
            assert_eq!(original, roundtrip, 
                "Failed roundtrip for {:?}: f64({}) -> {:?}", 
                original, as_f64, roundtrip);
        }
    }

    #[test]
    fn test_brontes_conversion() {
        assert_eq!(MarkoutTime::Brontes.as_f64(), None);
        
        // Test some values that shouldn't convert to any variant
        let invalid_values = [
            -3.0, -2.1, -1.7, -1.2, -0.7, -0.2, 
            0.2, 0.7, 1.2, 1.7, 2.1, 3.0
        ];
        
        for &value in &invalid_values {
            assert_eq!(
                MarkoutTime::from_f64(value), 
                None, 
                "Expected None for {}", value
            );
        }
    }

    #[test]
    fn test_display_format() {
        assert_eq!(MarkoutTime::Negative2.to_string(), "-2.0");
        assert_eq!(MarkoutTime::Zero.to_string(), "0.0");
        assert_eq!(MarkoutTime::Positive2.to_string(), "2.0");
        assert_eq!(MarkoutTime::Brontes.to_string(), "brontes");
    }
}