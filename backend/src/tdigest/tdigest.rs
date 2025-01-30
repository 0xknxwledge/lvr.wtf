/// TODO: 
/// 1. Implement Ord for Centroid
/// 2. Implement k1 scaling function and its inverse
/// 3. Implement merge_centroids
///     i. First, convert buffer into Vec<Centroids>, and sort with previous vec of Centroids
///     2. Then, pass through with the inputted delta parameter
/// 4, Implement quantile function w/ linear interpolation

#[derive(Clone, Debug)]
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
///
/// We are *not* yet implementing real merging logic; this is purely
/// for demonstrating structure and flow.
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
    pub delta_partial: f64,

    /// The "real" compression parameter used for the final merge in `finalize()`.
    pub delta_final: f64,
}

impl TDigest {
    /// Create a new (empty) T-Digest.
    ///
    /// # Arguments
    /// * `delta_partial` - compression for partial merges
    /// * `delta_final`   - final compression for the ultimate merge
    /// * `buffer_capacity` - how many un-merged points we allow before triggering partial merge
    pub fn new(delta_partial: f64, delta_final: f64, buffer_capacity: usize) -> Self {
        Self {
            centroids: Vec::new(),
            buffer: Vec::with_capacity(buffer_capacity),
            buffer_capacity,
            delta_partial,
            delta_final,
        }
    }

    /// Add a single data point to the T-Digest.
    /// We skip zero values outside of this structure (the caller tracks them).
    /// Weight is always 1 for each non-zero value.
    pub fn add(&mut self, x: f64) {
        // For now, we assume the caller doesn't call `add()` with zero,
        // or that they've already filtered zeros out.
        // If needed, you could explicitly skip if x == 0.0, etc.

        // Push the new value into the buffer
        self.buffer.push(x);

        // If we've exceeded the buffer capacity, trigger a partial merge.
        if self.buffer.len() >= self.buffer_capacity {
            self.partial_merge();
        }
    }

    /// Perform a *partial* merge of the buffer into the current `centroids`.
    ///
    /// This is a placeholder; in a real implementation, you'd:
    ///  1. Sort the buffer
    ///  2. Merge it with `centroids` using an approximate size constraint based on `delta_partial`
    ///  3. Produce a new (reduced) set of centroids
    ///  4. Clear the buffer
    pub fn partial_merge(&mut self) {
        if self.buffer.is_empty() {
            return;
        }

        // 1. Sort the buffer
        self.buffer.sort_by(|a, b| a.partial_cmp(b).unwrap());

        // 2. TODO: Implement actual merging logic
        //    For now, let's pretend we just convert each buffer item
        //    into an individual centroid (very naive).
        //    Real logic would combine points into fewer centroids.
        for &value in &self.buffer {
            let c = Centroid::new(value, 1.0);
            self.centroids.push(c);
        }

        // 3. Sort centroids by mean (normally you'd do a more sophisticated merge)
        self.centroids.sort_by(|a, b| a.mean.partial_cmp(&b.mean).unwrap());

        // 4. Clear the buffer
        self.buffer.clear();
    }

    /// Finalize the T-Digest by:
    ///  1. Doing a partial merge on leftover data in the buffer,
    ///  2. Re-merging everything using `delta_final`.
    pub fn finalize(&mut self) {
        // 1. Merge any leftover buffer with `delta_partial`
        if !self.buffer.is_empty() {
            self.partial_merge();
        }

        // 2. Re-merge everything with `delta_final`.
        //    A real approach would build a new set of centroids from self.centroids,
        //    applying the correct t-digest constraints with `delta_final`.
        //    For demonstration, we’ll just sort them (again).
        self.centroids.sort_by(|a, b| a.mean.partial_cmp(&b.mean).unwrap());
    }

}