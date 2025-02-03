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
}

impl TDigest {
    pub fn new(delta_partial: u64, delta_final: u64, buffer_capacity: usize) -> Self {
        Self {
            centroids: Vec::new(),
            buffer: Vec::with_capacity(buffer_capacity),
            buffer_capacity,
            delta_partial,
            delta_final,
            total_weight: 0.0,
            exact_samples: 0,
        }
    }

    pub fn add(&mut self, x: f64) {
        self.buffer.push(x);
        self.exact_samples += 1;
        self.total_weight += 1.0;

        if self.buffer.len() >= self.buffer_capacity {
            self.buffer.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
            self.partial_merge();
        }
    }

    pub fn add_many(&mut self, mut values: Vec<f64>) {
        values.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        let total_new = values.len() as u64;
        let mut index = 0;

        while index < values.len() {
            let space_left = self.buffer_capacity - self.buffer.len();
            
            if space_left == 0 {
                self.partial_merge();
            } else {
                let end_index = (index + space_left).min(values.len());
                self.buffer.extend_from_slice(&values[index..end_index]);
                index = end_index;

                if self.buffer.len() >= self.buffer_capacity {
                    self.partial_merge();
                }
            }
        }

        self.exact_samples += total_new;
        self.total_weight += total_new as f64;
    }

    pub fn merge(&mut self, other: &TDigest) -> u64 {
        let (merged_centroids, total_weight) = Self::merge_sorted_centroids(
            &self.centroids,
            &other.centroids
        );
        self.centroids = self.stratified_merge(merged_centroids, self.delta_partial);
        
        // Update exact samples count
        self.exact_samples += other.exact_samples;
        self.total_weight = total_weight;

        self.exact_samples
    }

    pub fn merge_into_locked(&self, locked_digest: &mut TDigest) -> u64 {
        let (merged_centroids, total_weight) = Self::merge_sorted_centroids(
            &locked_digest.centroids,
            &self.centroids
        );
        locked_digest.centroids = locked_digest.stratified_merge(merged_centroids, locked_digest.delta_partial);
        
        // Update exact counts in locked digest
        locked_digest.exact_samples += self.exact_samples;
        locked_digest.total_weight = total_weight;
        
        locked_digest.exact_samples
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
            self.partial_merge();
        }
        self.stratified_merge_in_place(self.delta_final);
    }

    // Get the exact count of non-zero samples
    pub fn samples(&self) -> u64 {
        self.exact_samples
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
}