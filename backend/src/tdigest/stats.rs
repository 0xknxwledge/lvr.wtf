use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionMetrics {
    pub mean: f64,
    pub variance: f64,
    pub std_dev: f64,
    pub skewness: f64,
    pub kurtosis: f64,
    pub sample_count: u64,
}


impl Default for DistributionMetrics {
    fn default() -> Self {
        Self {
            mean: 0.0,
            variance: 0.0,
            std_dev: 0.0,
            skewness: 0.0,
            kurtosis: 0.0,
            sample_count: 0,
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnlineStats {
    n: u64,
    mean: f64,
    m2: f64,   // Second central moment
    m3: f64,   // Third central moment
    m4: f64,   // Fourth central moment
}
impl OnlineStats {
    pub fn new() -> Self {
        Self {
            n: 0,
            mean: 0.0,
            m2: 0.0,
            m3: 0.0,
            m4: 0.0,
        }
    }

    /// Compute exact moments on first merge
    pub fn create(values: &[f64]) -> Self {
        let mut stats = Self::new();
        let n = values.len() as u64;
        if n == 0 {
            return stats;
        }

        // Calculate mean first
        let mean: f64 = values.iter().sum::<f64>() / n as f64;
        
        // Calculate central moments
        let mut m2 = 0.0;
        let mut m3 = 0.0;
        let mut m4 = 0.0;
        
        for &x in values {
            let delta = x - mean;
            let delta2 = delta * delta;
            m2 += delta2;
            m3 += delta2 * delta;
            m4 += delta2 * delta2;
        }

        stats.n = n;
        stats.mean = mean;
        stats.m2 = m2;
        stats.m3 = m3;
        stats.m4 = m4;
        stats
    }

    /// Batch Implementation of Pebay&Terriberry's general algorithm
    pub fn combine(a: &Self, b: &Self) -> Self {
        
        let delta = b.mean - a .mean;
        let total = a.n as f64 + b.n as f64;
        
        let a_prop = a.n as f64 / total;
        let b_prop = -(b.n as f64) / total;

        let da = a_prop * delta;
        let db = b_prop * delta;

        let da_2 = da * da;
        let db_2 = db * db;

        let m2 = a.m2 + b.m2 +
        (a.n as f64 * db_2) + (b.n as f64 * da_2);


        let m3 = a.m3 + b.m3 + 
        (a.n as f64 * db_2 * db)  + (b.n as f64 * da_2 * da) +
        3.0 * delta * (a.m2 * b_prop + b.m2 * a_prop);

        let m4 = a.m4 + b.m4 +
        (a.n as f64 * db_2 * db_2) + (b.n as f64 * da_2 * da_2) + 
        4.0 * delta * (a.m3 * b_prop + b.m3 * a_prop) +
        6.0 * (delta * delta) * (a.m2 * b_prop * b_prop + b.m2 * a_prop * a_prop);

        Self {
            n: a.n + b.n,
            mean: a.mean - db,
            m2,
            m3,
            m4,
        }
    }

    pub fn to_metrics(&self) -> DistributionMetrics {
        if self.n < 2 {
            return DistributionMetrics::default();
        }

        let n = self.n as f64;
        
        // Calculate variance with Bessel's correction
        let variance = self.m2 / (n - 1.0);
        let std_dev = variance.sqrt();
        let n: f64 = self.n as f64;

        // Fisher-Pearson Coefficient of Skewness
        let skewness = if self.n < 3 {
            0.0
        } else {
            self.m3 / (n * variance * std_dev)
        };

        // MoM  estimator for excess kurtosis 
        let kurtosis = if self.n < 4 {
            0.0
        } else {
            // Calculate excess kurtosis directly
            let n = self.n as f64;
            let variance = self.m2 / n;
            let m4_normalized = self.m4 / n;
            (m4_normalized / (variance * variance)) - 3.0
        };

        DistributionMetrics {
            mean: self.mean,
            variance,
            std_dev,
            skewness,
            kurtosis,
            sample_count: self.n,
        }
    }
}
