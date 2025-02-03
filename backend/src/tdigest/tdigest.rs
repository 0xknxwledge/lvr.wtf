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

/// A skeletal T-Digest structure that uses:
///  - Stratified merging (buffer + partial merges)
///  - Weight = 1 for each non-zero data point

#[derive(Debug, Serialize, Deserialize)]
pub struct TDigest {
    /// A sorted list of centroids (once merged). 
    pub centroids: Vec<Centroid>,

    /// Temporary buffer of raw data points (non-zero) that haven't been merged yet.
    /// Since each data point has weight = 1, we just store the value directly.
    pub buffer: Vec<f64>,

    /// When `buffer.len()` reaches `buffer_capacity`, we trigger a `partial_merge()`.
    pub buffer_capacity: usize,

    /// "Partial" compression parameter (used for intermediate merges).
    /// Might be, for example, 3× your final compression value.
    pub delta_partial: u64,

    /// The "real" compression parameter used for the final merge in `finalize()`.
    pub delta_final: u64,

    /// Number of blocks processed so far
    pub samples: u64
}

impl TDigest {
    /// Create a new (empty) T-Digest.
    ///
    /// # Arguments
    /// * `delta_partial` - compression for partial merges
    /// * `delta_final`   - final compression for the ultimate merge
    /// * `buffer_capacity` - how many un-merged points we allow before triggering partial merge
    pub fn new(delta_partial: u64, delta_final: u64, buffer_capacity: usize) -> Self {
        Self {
            centroids: Vec::new(),
            buffer: Vec::with_capacity(buffer_capacity),
            buffer_capacity,
            delta_partial,
            delta_final,
            samples: 0
        }
    }

    /// Add a single data point to the T-Digest.
    pub fn add(&mut self, x: f64) {
        self.buffer.push(x);
        self.samples += 1;

        // If we've exceeded the buffer capacity, trigger a partial merge.
        if self.buffer.len() >= self.buffer_capacity {
            // All data points in buffer are ensured to be != NaN, 
            // so should have total order
            self.buffer.sort_unstable_by(|a,b|
                a.partial_cmp(b).unwrap());

            self.partial_merge();
        }
    }

    /// Adds a vector of data points to the T-Digest
    pub fn add_many(&mut self, mut values: Vec<f64>) {
        // Ensure input values are sorted
        values.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        let total_samples = values.len();
        let mut index = 0;
        while index < total_samples{
            let space_left = self.buffer_capacity - self.buffer.len();

            if space_left == 0 {
                // If buffer is already full, trigger a partial merge
                self.partial_merge();
            } else {
                // Append as many values as possible to the buffer
                let end_index = (index + space_left).min(values.len());
                self.buffer.extend_from_slice(&values[index..end_index]);

                // Move index forward
                index = end_index;

                // If buffer is full after adding, trigger a partial merge
                if self.buffer.len() >= self.buffer_capacity {
                    self.partial_merge();
                }
            }
        }
        self.samples += total_samples as u64;
    }

    /// Merges two sorted slices of centroids into a single sorted Vec.
    pub fn merge_sorted_centroids(a: &[Centroid], b: &[Centroid]) -> Vec<Centroid> {
        let mut merged = Vec::with_capacity(a.len() + b.len());
        let mut i = 0;
        let mut j = 0;

        while i < a.len() && j < b.len() {
            if a[i].mean <= b[j].mean {
                merged.push(a[i].clone());
                i += 1;
            } else {
                merged.push(b[j].clone());
                j += 1;
            }
        }
        
        // At this point, either a or b have been exhausted, so safe to append the rest of the  other
        merged.extend_from_slice(&a[i..]);
        merged.extend_from_slice(&b[j..]);
        merged
    }


    /// Perform a *partial* merge of the buffer into the current `centroids`.
    pub fn partial_merge(&mut self) {
        if self.buffer.is_empty() {
            return;
        }

        // Ensure the buffer is sorted
        self.buffer.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());

        // Convert buffer to centroids with weight 1.0
        let buffer_centroids: Vec<Centroid> = self
            .buffer
            .iter()
            .map(|&x| Centroid::new(x, 1.0))
            .collect();

        // Merge existing centroids with buffer centroids (both are sorted)
        let merged = Self::merge_sorted_centroids(&self.centroids, &buffer_centroids);

        // Apply stratified merging with delta_partial
        self.centroids = self.stratified_merge(merged, self.delta_partial);

        // Clear the buffer
        self.buffer.clear();
    }

    /// Finalize the T-Digest by:
    ///  1. Doing a partial merge on leftover data in the buffer,
    ///  2. Re-merging everything using `delta_final`.
    pub fn finalize(&mut self) {
        // 1. Merge any leftover buffer with`delta_partial`
        if !self.buffer.is_empty() {
            // Perform a partial merge
            self.partial_merge();
        }

        // 2. Re-merge with `delta_final compression
        self.stratified_merge_in_place(self.delta_final);
    }

    /// Applies stratified merging to a sorted list of centroids using the given delta
    pub fn stratified_merge(&mut self, mut centroids: Vec<Centroid>, delta: u64) -> Vec<Centroid> {
        if centroids.is_empty() {
            return centroids;
        }
        let total_weight = self.samples as f64;

        let mut merged = Vec::with_capacity(centroids.len());

        // Start with the first centroid
        let mut current = centroids.remove(0);

        // q₀ = 0 at the start
        let mut q_0 = 0.0;
        let mut q_limit = self.weight_limit(q_0, delta);

        for centroid in centroids {
            let q = q_0 + (current.weight + centroid.weight) / total_weight;
            if q <= q_limit {
                // Merge
                let new_weight = current.weight + centroid.weight;
                let new_mean =
                    (current.mean * current.weight + centroid.mean * centroid.weight)
                     / new_weight;
                current = Centroid::new(new_mean, new_weight);
            } else {
                // Close out
                q_0 += current.weight / total_weight;
                q_limit = self.weight_limit(q_0, delta);

                merged.push(current);
                current = centroid;
            }
        }

        // Push last open centroid
        merged.push(current);
        merged
    }

    /// In-place "stratified merge" on `self.centroids`, using the given `delta`.
    /// Overwrites `self.centroids` in place, then truncates the extra tail.
    /// 
    /// Assumes:
    ///   - `self.centroids` is already sorted by mean.
    ///   - `self.samples` is up to date (so total_weight is correct).
    pub fn stratified_merge_in_place(&mut self, delta: u64) {
        // If nothing to merge, we're done.
        if self.centroids.is_empty() {
            return;
        }

        let total_weight = self.samples as f64;

        // We'll keep the "current centroid" in a local variable,
        // and maintain separate read and write indexes into `self.centroids`.
        let mut write_index = 0;
        let mut read_index = 0;

        // Start with the first centroid as `current`.
        let mut current = self.centroids[0].clone();
        read_index += 1;

        // q₀=0 at the start
        let mut q_0 = 0.0;
        let mut q_limit = self.weight_limit(q_0, delta);

        // Process the remaining centroids one by one
        while read_index < self.centroids.len() {
            // Clone out the next centroid so we don't keep a borrow
            let next = self.centroids[read_index].clone();

            // Tentative new q if we fold `next` into `current`
            let tentative_q = q_0 + (current.weight + next.weight) / total_weight;

            if tentative_q <= q_limit {
                // Merge `next` into `current`
                let new_weight = current.weight + next.weight;
                let new_mean =
                    (current.mean * current.weight + next.mean * next.weight)
                    / new_weight;
                current = Centroid::new(new_mean, new_weight);
            } else {
                // "Close out" the current centroid
                self.centroids[write_index] = current;
                write_index += 1;

                // Update q₀ and recalc the limit for the next group
                q_0 += current.weight / total_weight;
                q_limit = self.weight_limit(q_0, delta);

                // Start fresh with `next`
                current = next;
            }

            read_index += 1;
        }

        // Put the final "open" centroid into place
        self.centroids[write_index] = current;
        write_index += 1;

        // Truncate any leftover old centroids
        self.centroids.truncate(write_index);
    }

    /// Given a delta threshold and quantile, returns scale factor k
    pub fn k1(delta: u64, q: f64) -> f64 {
        let z: f64 = (2.0 * q) - 1.0;
        let b: f64 = (delta as f64)/TAU;
        b * z.asin()
    }   

    /// Given a scale factor k and delta threshold, returns quantile q
    pub fn inv_k1(k: f64, delta: u64) -> f64 {
        let x: f64 = (TAU * k) / (delta as f64);
        let x_sin = x.sin();
        (x_sin + 1.0) / 2.0
    }

    /// Computes the max weight for a given q_0 (i.e, left centroid weight?)
    pub fn weight_limit(&self, q_0: f64, delta: u64) -> f64 {
        return TDigest::inv_k1(TDigest::k1(delta, q_0) + 1.0, delta)
    }


    /// Computes the quantile for a given probability `q` (where 0.0 <= q <= 1.0).
    /// Uses linear interpolation between centroids to estimate the quantile.
    /// Returns `None` if `q` is out of bounds or if no data exists.
    pub fn quantile(&self, q: f64) -> Option<f64> {
        // Validate that q is in [0.0, 1.0].
        if q < 0.0 || q > 1.0 {
            return None;
        }

        // If there are no centroids (i.e., no data), return None.
        if self.centroids.is_empty() {
            return None;
        }

        // Ensure centroids are sorted by mean (they should already be sorted, but just in case).
        let mut sorted_centroids = self.centroids.clone();
        sorted_centroids.sort_by(|a, b| a.mean.partial_cmp(&b.mean).unwrap());

        // Total weight is equal to the number of samples.
        let total_weight: f64 = self.samples as f64;

        // Calculate the target cumulative weight.
        let target_weight = q * total_weight;

        let mut cumulative_weight = 0.0;
        for i in 0..sorted_centroids.len() {
            let centroid = &sorted_centroids[i];
            let next_cumulative_weight = cumulative_weight + centroid.weight;

            if next_cumulative_weight >= target_weight {
                if i == 0 {
                    // If it's the first centroid, we return its mean.
                    return Some(centroid.mean);
                }

                // Perform linear interpolation between the current and previous centroids.
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

        // If the target weight is beyond the last centroid, return the last centroid's mean.
        sorted_centroids.last().map(|c| c.mean)
    }


}