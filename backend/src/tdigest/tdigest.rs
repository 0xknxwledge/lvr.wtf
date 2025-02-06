use std::f64::consts::TAU;
use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Centroid {
    pub mean: f64,
    pub weight: f64,
}

impl Centroid {
    pub fn new(mean: f64, weight: f64) -> Self {
        Self { mean, weight }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionMetrics {
    pub mean: f64,
    pub variance: f64,
    pub std_dev: f64,
    pub skewness: f64,
    pub concentration_ratio: f64,
    pub sample_count: u64,
}
impl DistributionMetrics {
    pub fn calculate(
        value_ranges: &[(f64, f64, u64); 6],
        running_total_cents: u64,
    ) -> Self {
        let total_samples: u64 = value_ranges.iter()
            .map(|(_, _, count)| *count)
            .sum();

        if total_samples == 0 {
            return Self::default_metrics();
        }

        // Calculate true mean from running total (converting cents to dollars)
        let true_mean = (running_total_cents as f64) / (100.0 * total_samples as f64);
        
        // Calculate sum of squares and third moment
        let (sum_squares, third_moment) = Self::calculate_moments(
            value_ranges,
            true_mean,
            total_samples
        );
        
        // Calculate variance using the sum of squares method
        let variance = if total_samples > 1 {
            // Apply Bessel's correction for sample variance
            sum_squares / ((total_samples - 1) as f64)
        } else {
            sum_squares / (total_samples as f64)
        };

        let std_dev = variance.sqrt();
        
        // Calculate skewness using the method of moments estimator
        let skewness = if std_dev > 0.0 {
            let n = total_samples as f64;
            // Apply sample size correction for skewness
            let adjustment = (n * (n - 1.0)).sqrt() / (n - 2.0);
            (third_moment / (total_samples as f64)) / std_dev.powi(3) * adjustment
        } else {
            0.0
        };

        // Calculate concentration ratio
        let max_count = value_ranges.iter()
            .map(|(_, _, count)| *count)
            .max()
            .unwrap_or(0);
        let concentration_ratio = max_count as f64 / total_samples as f64;

        Self {
            mean: true_mean,
            variance,
            std_dev,
            skewness,
            concentration_ratio,
            sample_count: total_samples,
        }
    }

    fn calculate_moments(
        value_ranges: &[(f64, f64, u64); 6],
        true_mean: f64,
        total_samples: u64
    ) -> (f64, f64) {  // Returns (sum_squares, third_moment)
        let mut sum_squares = 0.0;
        let mut sum_cubes = 0.0;

        for (i, (start, end, count)) in value_ranges.iter().enumerate() {
            if *count == 0 {
                continue;
            }

            // For each bucket, estimate the sum of squares and cubes using value frequencies
            let frequencies = Self::estimate_value_frequencies(*start, *end, *count, i == 5);
            
            for (value, frequency) in frequencies {
                let diff = value - true_mean;
                sum_squares += diff * diff * frequency;
                sum_cubes += diff * diff * diff * frequency;
            }
        }

        (sum_squares, sum_cubes)
    }

    fn estimate_value_frequencies(
        start: f64,
        end: f64,
        count: u64,
        is_highest_bucket: bool
    ) -> Vec<(f64, f64)> {
        if count == 0 {
            return vec![];
        }

        if is_highest_bucket {
            // For the highest bucket (>$10,000), use a log-normal-like distribution
            let points = [
                (start * 1.0, 0.4),
                (start * 1.5, 0.3),
                (start * 2.0, 0.2),
                (start * 3.0, 0.1)
            ];
            points.iter()
                .map(|(value, weight)| (*value, *weight * count as f64))
                .collect()
        } else {
            // For finite buckets, use a more granular approach
            let num_points = 5;
            let step = (end - start) / (num_points as f64 - 1.0);
            let weight = count as f64 / num_points as f64;
            
            (0..num_points)
                .map(|i| {
                    let value = start + step * i as f64;
                    (value, weight)
                })
                .collect()
        }
    }

    fn default_metrics() -> Self {
        Self {
            mean: 0.0,
            variance: 0.0,
            std_dev: 0.0,
            skewness: 0.0,
            concentration_ratio: 0.0,
            sample_count: 0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TDigest {
    /// A sorted list of centroids (once merged)
    pub centroids: Vec<Centroid>,

    /// Temporary buffer of raw data points (non-zero) that haven't been merged yet
    pub buffer: Vec<f64>,

    /// When `buffer.len()` reaches `buffer_capacity`, we trigger a `partial_merge()`
    pub buffer_capacity: usize,

    /// "Partial" compression parameter (used for intermediate merges)
    pub delta_partial: u64,

    /// The "real" compression parameter used for the final merge in `finalize()`
    pub delta_final: u64,

    /// Total weight across all centroids (used for quantile calculations)
    pub total_weight: f64,

    /// Exact count of non-zero samples processed
    pub exact_samples: u64,

    /// Running total (in dollars) used for calculating distribution metrics
    pub running_total: f64,

    /// Histogram buckets for distribution metrics
    pub value_ranges: [(f64, f64, u64); 6], // (start, end, count) for non-zero ranges
}

impl TDigest {
    pub fn new() -> Self {
        Self {
            centroids: Vec::new(),
            buffer: Vec::with_capacity(500),
            buffer_capacity: 500,
            delta_partial: 100,
            delta_final: 50,
            total_weight: 0.0,
            exact_samples: 0,
            running_total: 0.0,
            // Initialize value ranges to match Checkpoint buckets
            value_ranges: [
                (0.01, 10.0, 0),   // $0.01-$10
                (10.0, 100.0, 0),  // $10-$100
                (100.0, 500.0, 0), // $100-$500
                (500.0, 1000.0, 0),// $500-$1000
                (1000.0, 10000.0, 0), // $1000-$10000
                (10000.0, f64::INFINITY, 0), // $10000+
            ],
        }
    }


    pub fn samples(&self) -> u64 {
        self.exact_samples
    }

    pub fn get_range_counts(&self) -> [(f64, f64, u64); 6] {
        self.value_ranges
    }

    pub fn add(&mut self, x: f64, range_count_idx: usize) {
        // Update the count for the pre-calculated range
        self.value_ranges[range_count_idx].2 += 1;
        
        // Add to buffer and update tracking metrics
        self.buffer.push(x);
        self.exact_samples += 1;
        self.total_weight += 1.0;
        self.running_total += x;

        // Adapt compression parameters based on distribution
        self.adapt_parameters();

        // Merge buffer if it reaches capacity
        if self.buffer.len() >= self.buffer_capacity {
            self.buffer.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
            self.partial_merge();
        }
    }

    pub fn adapt_parameters(&mut self) {
        // Calculate distribution metrics using the current value ranges and running total
        let metrics = DistributionMetrics::calculate(
            &self.value_ranges,
            (self.running_total * 100.0) as u64 // Convert running total to cents
        );

        // Adjust buffer capacity based on sample count and distribution characteristics
        self.buffer_capacity = self.calculate_buffer_capacity(&metrics);

        // Adjust compression parameters based on distribution characteristics
        self.adjust_compression_parameters(&metrics);
    }

    fn calculate_buffer_capacity(&self, metrics: &DistributionMetrics) -> usize {
        // Start with base capacity
        let mut capacity = 500;

        // Increase capacity for highly skewed distributions
        if metrics.skewness.abs() > 2.0 {
            capacity = (capacity as f64 * 1.5) as usize;
        }

        // Adjust based on concentration ratio
        if metrics.concentration_ratio > 0.3 {
            // Reduce capacity for highly concentrated distributions
            capacity = (capacity as f64 * 0.8) as usize;
        }

        // Scale with sample count, but cap at reasonable limits
        let sample_factor = (metrics.sample_count as f64 / 1000.0).sqrt();
        capacity = (capacity as f64 * sample_factor.min(2.0)) as usize;

        // Ensure capacity stays within reasonable bounds
        capacity.clamp(100, 2000)
    }

    fn adjust_compression_parameters(&mut self, metrics: &DistributionMetrics) {
        // Base delta values
        let base_partial = 100;
        let base_final = 50;

        // Calculate adjustment factors
        let variance_factor = if metrics.std_dev > 0.0 {
            // Normalize variance impact
            (1.0 + (metrics.variance / (metrics.mean * metrics.mean)).sqrt()).min(2.0)
        } else {
            1.0
        };

        let skewness_factor = if metrics.skewness.abs() > 1.0 {
            // More compression for highly skewed distributions
            0.8
        } else {
            1.0
        };

        let concentration_factor = if metrics.concentration_ratio > 0.5 {
            // Less compression for highly concentrated distributions
            1.2
        } else {
            1.0
        };

        // Combine adjustment factors
        let adjustment = variance_factor * skewness_factor * concentration_factor;

        // Apply adjustments to delta parameters
        self.delta_partial = (base_partial as f64 * adjustment) as u64;
        self.delta_final = (base_final as f64 * adjustment) as u64;

        // Ensure parameters stay within reasonable bounds
        self.delta_partial = self.delta_partial.clamp(50, 200);
        self.delta_final = self.delta_final.clamp(25, 100);
    }


    pub fn merge_value_ranges(&mut self, other: &[(f64, f64, u64); 6]) {
        for i in 0..6 {
            self.value_ranges[i].2 += other[i].2;
        }
    }

    pub fn merge(&mut self, other: &TDigest) -> u64 {
        let (merged_centroids, total_weight) = Self::merge_sorted_centroids(
            &self.centroids,
            &other.centroids
        );
        
        // Update centroids using stratified merge
        self.centroids = self.stratified_merge(merged_centroids, self.delta_partial);
        
        // Update total weight and sample tracking
        self.total_weight = total_weight;
        self.exact_samples += other.exact_samples;

        // Merge value range counts
        self.merge_value_ranges(&other.value_ranges);

        self.exact_samples
    }

    pub fn merge_into_locked(&self, target: &mut TDigest) -> u64 {
        // Merge centroids
        let (merged_centroids, total_weight) = Self::merge_sorted_centroids(
            &target.centroids,
            &self.centroids
        );
        target.centroids = target.stratified_merge(merged_centroids, target.delta_partial);
        
        // Update counts and weights
        target.total_weight = total_weight;
        target.exact_samples += self.exact_samples;

        // Merge value range counts
        target.merge_value_ranges(&self.value_ranges);

        target.exact_samples
    }



    pub fn merge_sorted_centroids(a: &[Centroid], b: &[Centroid]) -> (Vec<Centroid>, f64) {
        let mut merged = Vec::with_capacity(a.len() + b.len());
        let mut total_weight = 0.0;

        let mut i = 0;
        let mut j = 0;
        
        while i < a.len() && j < b.len() {
            if a[i].mean <= b[j].mean {
                total_weight += a[i].weight;
                merged.push(a[i].clone());
                i += 1;
            } else {
                total_weight += b[j].weight;
                merged.push(b[j].clone());
                j += 1;
            }
        }

        for centroid in &a[i..] {
            total_weight += centroid.weight;
            merged.push(centroid.clone());
        }
        for centroid in &b[j..] {
            total_weight += centroid.weight;
            merged.push(centroid.clone());
        }

        (merged, total_weight)
    }

    pub fn partial_merge(&mut self) {
        if self.buffer.is_empty() {
            return;
        }

        let buffer_centroids: Vec<Centroid> = self.buffer
            .iter()
            .map(|&x| Centroid::new(x, 1.0))
            .collect();

        let (merged, total_weight) = Self::merge_sorted_centroids(
            &self.centroids,
            &buffer_centroids
        );
        
        self.centroids = self.stratified_merge(merged, self.delta_partial);
        self.total_weight = total_weight;
        self.buffer.clear();
    }

    pub fn finalize(&mut self) {
        if !self.buffer.is_empty() {
            // Process any remaining buffered values
            let buffered_digest = {
                let mut temp_digest = TDigest::new();
                for &value in &self.buffer {
                    // Update value ranges for remaining buffered values
                    for (start, end, count) in temp_digest.value_ranges.iter_mut() {
                        if value >= *start && (value < *end || (*end == f64::INFINITY && value >= *start)) {
                            *count += 1;
                            break;
                        }
                    }
                }
                temp_digest.centroids = self.buffer.iter()
                    .map(|&x| Centroid::new(x, 1.0))
                    .collect();
                temp_digest.total_weight = self.buffer.len() as f64;
                temp_digest
            };

            // Merge the buffered values
            let (merged_centroids, _) = Self::merge_sorted_centroids(
                &self.centroids,
                &buffered_digest.centroids
            );
            
            // Update centroids with merged result
            self.centroids = merged_centroids;
            
            // Merge value range counts from buffered data
            self.merge_value_ranges(&buffered_digest.value_ranges);
            
            // Clear the buffer
            self.buffer.clear();
        }

        // Perform final in-place compression
        self.stratified_merge_in_place(self.delta_final);
    }

    pub fn stratified_merge(&mut self, mut centroids: Vec<Centroid>, delta: u64) -> Vec<Centroid> {
        if centroids.is_empty() {
            return centroids;
        }

        let mut merged = Vec::with_capacity(centroids.len());
        let mut current = centroids.remove(0);
        let mut q_0 = 0.0;
        let mut q_limit = self.weight_limit(q_0, delta);

        for centroid in centroids {
            let q = q_0 + (current.weight + centroid.weight) / self.total_weight;
            if q <= q_limit {
                let new_weight = current.weight + centroid.weight;
                let new_mean = (current.mean * current.weight + 
                              centroid.mean * centroid.weight) / new_weight;
                current = Centroid::new(new_mean, new_weight);
            } else {
                q_0 += current.weight / self.total_weight;
                q_limit = self.weight_limit(q_0, delta);
                merged.push(current);
                current = centroid;
            }
        }

        merged.push(current);
        merged
    }

    pub fn stratified_merge_in_place(&mut self, delta: u64) {
        if self.centroids.is_empty() {
            return;
        }

        let mut write_index = 0;
        let mut read_index = 0;
        let mut current = self.centroids[0].clone();
        read_index += 1;

        let mut q_0 = 0.0;
        let mut q_limit = self.weight_limit(q_0, delta);

        while read_index < self.centroids.len() {
            let next = self.centroids[read_index].clone();
            let tentative_q = q_0 + (current.weight + next.weight) / self.total_weight;

            if tentative_q <= q_limit {
                let new_weight = current.weight + next.weight;
                let new_mean = (current.mean * current.weight + next.mean * next.weight) / new_weight;
                current = Centroid::new(new_mean, new_weight);
            } else {
                self.centroids[write_index] = current;
                write_index += 1;
                q_0 += current.weight / self.total_weight;
                q_limit = self.weight_limit(q_0, delta);
                current = next;
            }
            read_index += 1;
        }

        self.centroids[write_index] = current;
        write_index += 1;
        self.centroids.truncate(write_index);
    }

    pub fn k1(delta: u64, q: f64) -> f64 {
        let z: f64 = (2.0 * q) - 1.0;
        let b: f64 = (delta as f64)/TAU;
        b * z.asin()
    }   

    pub fn inv_k1(k: f64, delta: u64) -> f64 {
        let x: f64 = (TAU * k) / (delta as f64);
        let x_sin = x.sin();
        (x_sin + 1.0) / 2.0
    }

    pub fn weight_limit(&self, q_0: f64, delta: u64) -> f64 {
        TDigest::inv_k1(TDigest::k1(delta, q_0) + 1.0, delta)
    }

    pub fn quantile(&self, q: f64) -> Option<f64> {
        if q < 0.0 || q > 1.0 || self.centroids.is_empty() {
            return None;
        }

        let mut sorted_centroids = self.centroids.clone();
        sorted_centroids.sort_by(|a, b| a.mean.partial_cmp(&b.mean).unwrap());

        let target_weight = q * self.total_weight;
        let mut cumulative_weight = 0.0;

        for i in 0..sorted_centroids.len() {
            let centroid = &sorted_centroids[i];
            let next_cumulative_weight = cumulative_weight + centroid.weight;

            if next_cumulative_weight >= target_weight {
                if i == 0 {
                    return Some(centroid.mean);
                }

                let prev_centroid = &sorted_centroids[i - 1];
                let prev_cumulative_weight = cumulative_weight;
                let interpolated = prev_centroid.mean
                    + (target_weight - prev_cumulative_weight)
                        * (centroid.mean - prev_centroid.mean)
                        / centroid.weight;
                return Some(interpolated);
            }

            cumulative_weight = next_cumulative_weight;
        }

        sorted_centroids.last().map(|c| c.mean)
    }

    fn calculate_range_index(value: f64) -> usize {
        match value {
            x if x <= 10.0 => 0,    // $0.01-$10
            x if x <= 100.0 => 1,   // $10-$100
            x if x <= 500.0 => 2,   // $100-$500
            x if x <= 1000.0 => 3,  // $500-$1000
            x if x <= 10000.0 => 4, // $1000-$10000
            _ => 5                  // $10000+
        }
    }

}