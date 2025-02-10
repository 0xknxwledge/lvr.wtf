use crate::stats::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AdaptiveParameters {
    // Current parameters
    pub delta_partial: u64,
    pub delta_final: u64,
    pub buffer_size: usize,
    
    // Base parameters for small sample sizes
    pub base_delta_partial: u64,    // 20
    pub base_delta_final: u64,      // 10
    pub base_buffer_size: usize,    // 200
    
    // Scaled parameters for large sample sizes
    pub scaled_delta_partial: u64,  // 1000
    pub scaled_delta_final: u64,    // 200
    pub scaled_buffer_size: usize,  // 2000
    
    // Thresholds for adaptation
    pub initial_scale_threshold: u64,  // 2000 samples
    pub adaptation_threshold: u64,     // 10000 samples
    pub samples_seen: u64,
    pub adapted: bool,
}

impl AdaptiveParameters {
    pub fn new() -> Self {
        Self {
            // Start with conservative base parameters
            delta_partial: 20,
            delta_final: 10,
            buffer_size: 200,
            
            // Base parameters (conservative)
            base_delta_partial: 20,
            base_delta_final: 10,
            base_buffer_size: 200,
            
            // Scaled parameters (for large sample sizes)
            scaled_delta_partial: 1000,
            scaled_delta_final: 200,
            scaled_buffer_size: 2000,
            
            // Thresholds
            initial_scale_threshold: 2000,
            adaptation_threshold: 10000,
            samples_seen: 0,
            adapted: false,
        }
    }

    pub fn fine_tune_parameters(&mut self, stats: &DistributionMetrics) {
        // Base scaling factor on sample size relative to our thresholds
        let size_factor: f64 = (self.samples_seen as f64 / self.adaptation_threshold as f64)
            .min(3.0);  // Cap at 3x
    
        // Start with neutral adjustment
        let mut adjustment: f64 = 1.0;
        
        // Adjust for skewness - more compression for highly skewed distributions
        let abs_skew: f64 = stats.skewness.abs();
        if abs_skew > 1.0 {
            adjustment *= 1.0 + (0.1 * (abs_skew - 1.0));  // Cap at 30% increase
            adjustment = adjustment.min(0.3);
        }
    
        // Adjust for kurtosis
        // For platykurtic (negative excess kurtosis), increase compression
        // For leptokurtic (positive excess kurtosis), decrease compression
        if stats.kurtosis < 0.0 {
            // More compression for platykurtic distributions
            // Maximum 20% increase for highly platykurtic cases
            adjustment *= 1.0 + (0.2 * (-stats.kurtosis / 2.0));
        } else {
            // Less compression for leptokurtic distributions
            // Maximum 20% decrease for highly leptokurtic cases
            adjustment *= 1.0 - (0.2 * (stats.kurtosis / 4.0));
        }

        adjustment = adjustment.min(0.2);
    
        // Conservative compression for small samples
        if self.samples_seen < 5000 {
            adjustment *= 0.8;
        }
    
        // Calculate new parameters with upper bound
        let new_delta_partial = ((self.base_delta_partial as f64 * size_factor * adjustment)
            .min(self.scaled_delta_partial as f64)) as u64;
            
        let new_delta_final = ((self.base_delta_final as f64 * size_factor * adjustment)
            .min(self.scaled_delta_final as f64)) as u64;
            
        let new_buffer_size = ((self.base_buffer_size as f64 * size_factor)
            .min(self.scaled_buffer_size as f64)) as usize;
    
        // Check for lower bound
        self.delta_partial = new_delta_partial.max(self.base_delta_partial);
        self.delta_final = new_delta_final.max(self.base_delta_final);
        self.buffer_size = new_buffer_size.max(self.base_buffer_size);
    }

    pub fn adapt(&mut self, stats: &DistributionMetrics) {
        self.samples_seen = stats.sample_count;
        
        if self.samples_seen < self.initial_scale_threshold {
            return;
        }
        
        if self.delta_partial == self.base_delta_partial {
            self.apply_initial_scaling();
            return;
        }
        
        if self.samples_seen >= self.adaptation_threshold {
            self.fine_tune_parameters(stats);
        }
    }

    fn apply_initial_scaling(&mut self) {
        // Scale up parameters, but with safety limits for smaller datasets
        let scale_factor = (self.samples_seen as f64 / self.initial_scale_threshold as f64)
            .min(2.0);  // Cap initial scaling at 2x

        self.delta_partial = ((self.base_delta_partial as f64 * scale_factor)
            .min(self.scaled_delta_partial as f64)) as u64;
        
        self.delta_final = ((self.base_delta_final as f64 * scale_factor)
            .min(self.scaled_delta_final as f64)) as u64;
        
        self.buffer_size = ((self.base_buffer_size as f64 * scale_factor)
            .min(self.scaled_buffer_size as f64)) as usize;
    }

    pub fn should_merge(&self, buffer_count: usize) -> bool {
        buffer_count >= self.buffer_size
    }

    pub fn reset(&mut self) {
        self.delta_partial = self.base_delta_partial;
        self.delta_final = self.base_delta_final;
        self.buffer_size = self.base_buffer_size;
        self.samples_seen = 0;
        self.adapted = false;
    }

    pub fn current_scale_factor(&self) -> f64 {
        self.delta_partial as f64 / self.base_delta_partial as f64
    }
}
