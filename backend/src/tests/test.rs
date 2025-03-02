pub use crate::*;

#[cfg(test)]
pub mod tests {
    use super::*;
    use rand::prelude::*;
    use rand_distr::{Distribution, Normal, LogNormal, Uniform};
    use std::f64::consts::E;

    #[derive(Debug, Clone, Copy)]
    enum DataDistribution {
        Normal { mean: f64, std_dev: f64 },
        LogNormal { location: f64, scale: f64 },
        Uniform { lower: f64, upper: f64 },
    }
        // Helper function for relative error calculation
    fn relative_error(computed: f64, expected: f64) -> f64 {
        if expected == 0.0 {
            computed.abs()
        } else {
            ((computed - expected) / expected).abs()
        }
    }


    // Helper functions for statistical calculations
    fn mean(distribution: DataDistribution) -> f64 {
        match distribution {
            DataDistribution::Normal { mean, .. } => mean,
            DataDistribution::LogNormal { location, scale } => {
                let mu = location;
                let sigma = scale;
                E.powf(mu + (sigma * sigma / 2.0))
            },
            DataDistribution::Uniform { lower, upper } => {
                (lower + upper) / 2.0
            }
        }
    }

    fn variance(distribution: DataDistribution) -> f64 {
        match distribution {
            DataDistribution::Normal { std_dev, .. } => std_dev * std_dev,
            DataDistribution::LogNormal { location, scale } => {
                let mu = location;
                let sigma = scale;
                (E.powf(sigma * sigma) - 1.0) * E.powf(2.0 * mu + sigma * sigma)
            },
            DataDistribution::Uniform { lower, upper } => {
                (upper - lower).powi(2) / 12.0
            }
        }
    }

    fn skewness(distribution: DataDistribution) -> f64 {
        match distribution {
            DataDistribution::Normal { .. } | 
            DataDistribution::Uniform { .. } => {
                0.0
            },
            DataDistribution::LogNormal { scale, .. } => {
                let sigma_sq = scale * scale;
                let term1 = E.powf(sigma_sq) + 2.0;
                let term2 = (E.powf(sigma_sq) - 1.0).sqrt();
                term1 * term2
            }
        }
    }
    fn kurtosis(distribution: DataDistribution) -> f64 {
        match distribution {
            DataDistribution::Normal { .. } => {
                0.0  // Normal distribution has excess kurtosis of 0
            },
            DataDistribution::Uniform { .. } => {
                -1.2  // Uniform distribution has excess kurtosis of -1.2 (raw kurtosis - 3)
            },
            DataDistribution::LogNormal { scale, .. } => {
                let var = scale * scale;
                let term_1 = 4.0*var;
                let term_2 = 3.0*var;
                let term_3 = 2.0*var;
                term_1.exp() + (2.0 * term_2.exp()) + (3.0 * term_3.exp()) - 6.0

            }
        }
    }

    // Helper functions to generate datasets
    fn generate_normal_data(mean: f64, std_dev: f64, size: usize) -> (Vec<f64>, DataDistribution) {
        let normal = Normal::new(mean, std_dev).unwrap();
        let mut rng = thread_rng();
        (
            normal.sample_iter(&mut rng).take(size).collect(),
            DataDistribution::Normal { mean, std_dev }
        )
    }

    fn generate_lognormal_data(location: f64, scale: f64, size: usize) -> (Vec<f64>, DataDistribution) {
        let lognormal = LogNormal::new(location, scale).unwrap();
        let mut rng = thread_rng();
        (
            lognormal.sample_iter(&mut rng).take(size).collect(),
            DataDistribution::LogNormal { location, scale }
        )
    }

    fn generate_uniform_data(lower: f64, upper: f64, size: usize) -> (Vec<f64>, DataDistribution) {
        let uniform = Uniform::new(lower, upper);
        let mut rng = thread_rng();
        (
            uniform.sample_iter(&mut rng).take(size).collect(),
            DataDistribution::Uniform { lower, upper }
        )
    }

    // --- Stats.rs Tests ---
    #[test]
    fn test_distribution_metrics_normal() {
        let (data, dist) = generate_normal_data(10.0, 5.0, 10000); // Increased sample size
        let online_stats = OnlineStats::create(&data);
        let computed_metrics = online_stats.to_metrics();

        let expected_mean = mean(dist);
        let expected_variance = variance(dist);
        let expected_skewness = skewness(dist);
        let expected_kurtosis = kurtosis(dist);

        // Relative error tolerances
        let mean_tol = 0.05;  // 5% tolerance
        let var_tol = 0.10;   // 10% tolerance
        let skew_tol = 0.15;  // 15% tolerance
        let kurt_tol = 0.20;  // 20% tolerance

        assert!(relative_error(computed_metrics.mean, expected_mean) < mean_tol,
            "Mean mismatch: computed={}, expected={}, relative error={:.2}%", 
            computed_metrics.mean, expected_mean, 
            relative_error(computed_metrics.mean, expected_mean) * 100.0);

        assert!(relative_error(computed_metrics.variance, expected_variance) < var_tol,
            "Variance mismatch: computed={}, expected={}, relative error={:.2}%", 
            computed_metrics.variance, expected_variance,
            relative_error(computed_metrics.variance, expected_variance) * 100.0);

        // Only test skewness and kurtosis for non-zero expected values
        if expected_skewness != 0.0 {
            assert!(relative_error(computed_metrics.skewness, expected_skewness) < skew_tol,
                "Skewness mismatch: computed={}, expected={}, relative error={:.2}%", 
                computed_metrics.skewness, expected_skewness,
                relative_error(computed_metrics.skewness, expected_skewness) * 100.0);
        } else {
            assert!(computed_metrics.skewness.abs() < 0.1, 
                "Skewness should be close to 0, got {}", computed_metrics.skewness);
        }

        if expected_kurtosis != 0.0 {
            assert!(relative_error(computed_metrics.kurtosis, expected_kurtosis) < kurt_tol,
                "Kurtosis mismatch: computed={}, expected={}, relative error={:.2}%", 
                computed_metrics.kurtosis, expected_kurtosis,
                relative_error(computed_metrics.kurtosis, expected_kurtosis) * 100.0);
        } else {
            assert!(computed_metrics.kurtosis.abs() < 0.3,
                "Kurtosis should be close to 0, got {}", computed_metrics.kurtosis);
        }
    }

    #[test]
    fn test_distribution_metrics_lognormal() {
        let location = 0.5;
        let scale = 0.75;
        let (data, dist) = generate_lognormal_data(location, scale, 10000);
        let online_stats = OnlineStats::create(&data);
        let computed_metrics = online_stats.to_metrics();

        let expected_mean = mean(dist);
        let expected_variance = variance( dist);
        let expected_skewness = skewness( dist);
        let expected_kurtosis = kurtosis( dist);

        // Relative error tolerances
        let mean_tol = 0.05;  // 5% tolerance
        let var_tol = 0.30;   // 10% tolerance
        let skew_tol = 0.3;  // 15% tolerance
        let kurt_tol = 0.3;  // 70% tolerance

        assert!(relative_error(computed_metrics.mean, expected_mean) < mean_tol,
            "Mean mismatch: computed={}, expected={}, relative error={:.2}%", 
            computed_metrics.mean, expected_mean, 
            relative_error(computed_metrics.mean, expected_mean) * 100.0);

        assert!(relative_error(computed_metrics.variance, expected_variance) < var_tol,
            "Variance mismatch: computed={}, expected={}, relative error={:.2}%", 
            computed_metrics.variance, expected_variance,
            relative_error(computed_metrics.variance, expected_variance) * 100.0);

        // Only test skewness and kurtosis for non-zero expected values
        if expected_skewness != 0.0 {
            assert!(relative_error(computed_metrics.skewness, expected_skewness) < skew_tol,
                "Skewness mismatch: computed={}, expected={}, relative error={:.2}%", 
                computed_metrics.skewness, expected_skewness,
                relative_error(computed_metrics.skewness, expected_skewness) * 100.0);
        } else {
            assert!(computed_metrics.skewness.abs() < 0.1, 
                "Skewness should be close to 0, got {}", computed_metrics.skewness);
        }

        if expected_kurtosis != 0.0 {
            assert!(relative_error(computed_metrics.kurtosis, expected_kurtosis) < kurt_tol,
                "Kurtosis mismatch: computed={}, expected={}, relative error={:.2}%", 
                computed_metrics.kurtosis, expected_kurtosis,
                relative_error(computed_metrics.kurtosis, expected_kurtosis) * 100.0);
        } else {
            assert!(computed_metrics.kurtosis.abs() < 0.1,
                "Kurtosis should be close to 0, got {}", computed_metrics.kurtosis);
        }
    }

    #[test]
    fn test_distribution_metrics_uniform() {
        let lower = -1.0;
        let upper = 1.0;
        let (data, dist) = generate_uniform_data(lower, upper, 10000);
        let online_stats = OnlineStats::create(&data);
        let computed_metrics = online_stats.to_metrics();

        let expected_mean = mean(dist);
        let expected_variance = variance(dist);
        let expected_skewness = skewness(dist);
        let expected_kurtosis = kurtosis(dist);

        // Relative error tolerances
        let mean_tol = 0.05;  // 5% tolerance
        let var_tol = 0.10;   // 10% tolerance
        let skew_tol = 0.15;  // 15% tolerance
        let kurt_tol = 0.20;  // 20% tolerance

        assert!(relative_error(computed_metrics.mean, expected_mean) < mean_tol,
            "Mean mismatch: computed={}, expected={}, relative error={:.2}%", 
            computed_metrics.mean, expected_mean, 
            relative_error(computed_metrics.mean, expected_mean) * 100.0);

        assert!(relative_error(computed_metrics.variance, expected_variance) < var_tol,
            "Variance mismatch: computed={}, expected={}, relative error={:.2}%", 
            computed_metrics.variance, expected_variance,
            relative_error(computed_metrics.variance, expected_variance) * 100.0);

        // Only test skewness and kurtosis for non-zero expected values
        if expected_skewness != 0.0 {
            assert!(relative_error(computed_metrics.skewness, expected_skewness) < skew_tol,
                "Skewness mismatch: computed={}, expected={}, relative error={:.2}%", 
                computed_metrics.skewness, expected_skewness,
                relative_error(computed_metrics.skewness, expected_skewness) * 100.0);
        } else {
            assert!(computed_metrics.skewness.abs() < 0.1, 
                "Skewness should be close to 0, got {}", computed_metrics.skewness);
        }

        if expected_kurtosis != 0.0 {
            assert!(relative_error(computed_metrics.kurtosis, expected_kurtosis) < kurt_tol,
                "Kurtosis mismatch: computed={}, expected={}, relative error={:.2}%", 
                computed_metrics.kurtosis, expected_kurtosis,
                relative_error(computed_metrics.kurtosis, expected_kurtosis) * 100.0);
        } else {
            assert!(computed_metrics.kurtosis.abs() < 0.1,
                "Kurtosis should be close to 0, got {}", computed_metrics.kurtosis);
        }
    }

    fn percentile(data: &[f64], q: f64) -> f64 {
        let mut sorted = data.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let n = sorted.len();
        if n == 0 {
            return 0.0;
        }
        
        if n == 1 {
            return sorted[0];
        }
        
        let position = q * (n - 1) as f64;
        let lower_index = position.floor() as usize;
        let upper_index = position.ceil() as usize;
        
        if lower_index == upper_index {
            return sorted[lower_index];
        }
        
        let weight = position - lower_index as f64;
        sorted[lower_index] * (1.0 - weight) + sorted[upper_index] * weight
    }

    #[test]
    fn test_tdigest_quantiles_lognormal() {
        // Using parameters that produce a right-skewed distribution
        // similar to typical LVR patterns
        let location = 1.0;  // μ parameter
        let scale = 1.5;    // σ parameter, larger value increases skewness
        let sample_size = 50000;
        
        let (data, _) = generate_lognormal_data(location, scale, sample_size);
        let mut tdigest = TDigest::new();
        
        // Create sorted copy for exact percentile calculation
        let mut sorted_data = data.clone();
        sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        // Add data to TDigest
        for &x in &data {
            tdigest.add(x);
        }
        tdigest.finalize();
    
        // Test key percentiles that are important for LVR analysis
        let test_quantiles = [0.25, 0.5, 0.75];
        
        // Increase tolerance for higher quantiles since lognormal 
        // distributions have more variance in the upper tail
        let quantile_tol = 0.35;  
    
        for &q in &test_quantiles {
            let expected = percentile(&sorted_data, q);
            let computed = tdigest.quantile(q).unwrap();
            
            assert!(relative_error(computed, expected) < quantile_tol,
                "Quantile {} mismatch for lognormal distribution:\n\
                 Expected: {:.4}\n\
                 Computed: {:.4}\n\
                 Relative error: {:.2}%\n\
                 Parameters: μ={}, σ={}", 
                q, expected, computed, 
                relative_error(computed, expected) * 100.0,
                location, scale);
        }
    }



    #[test]
    fn test_tdigest_merge() {
        let (data1, _) = generate_normal_data(10.0, 5.0, 500);
        let (data2, _) = generate_normal_data(20.0, 5.0, 500);

        let mut td1 = TDigest::new();
        let mut td2 = TDigest::new();
        for &x in &data1 {
            td1.add(x);
        }
        for &x in &data2 {
            td2.add(x);
        }

        td1.finalize();
        td2.finalize();

        let (merged_centroids, _) = TDigest::merge_sorted_centroids(&td1.centroids, &td2.centroids);
        assert!(!merged_centroids.is_empty(), "Merged centroids should not be empty");
    }

    #[test]
    fn test_tdigest_edge_cases() {
        let mut tdigest = TDigest::new();
        assert_eq!(tdigest.quantile(0.5), None, "Quantile on empty TDigest should be None");

        tdigest.add(42.0);
        tdigest.finalize();
        assert_eq!(tdigest.quantile(0.5), Some(42.0), "Single-value TDigest should return that value");
    }

    // --- AdaptiveParameters Tests ---
    #[test]
    fn test_adaptive_parameters_initial() {
        let params = AdaptiveParameters::new();
        assert_eq!(params.delta_partial, 20);
        assert_eq!(params.delta_final, 10);
        assert_eq!(params.buffer_size, 200);
    }

    #[test]
    fn test_adaptive_parameters_scaling() {
        let mut params = AdaptiveParameters::new();
        
        // Create metrics indicating high skewness and kurtosis
        let stats = DistributionMetrics {
            mean: 0.0,
            variance: 100.0,  // Large variance
            std_dev: 10.0,
            skewness: 2.0,    // High skewness
            kurtosis: 6.0,    // High kurtosis
            sample_count: 10000, // Large sample size
        };
        
        // Store initial values
        let initial_delta_partial = params.delta_partial;
        let initial_delta_final = params.delta_final;
        let initial_buffer_size = params.buffer_size;
        
        params.adapt(&stats);
        
        // Check that parameters have changed
        assert!(params.delta_partial != initial_delta_partial || 
               params.delta_final != initial_delta_final || 
               params.buffer_size != initial_buffer_size,
               "Parameters should change after adaptation");
        
        // Check that at least one parameter has increased
        assert!(params.delta_partial > initial_delta_partial || 
               params.delta_final > initial_delta_final || 
               params.buffer_size > initial_buffer_size,
               "At least one parameter should increase after adaptation");
    }

    #[test]
    fn test_adaptive_parameters_reset() {
        let mut params = AdaptiveParameters::new();
        let stats = DistributionMetrics {
            mean: 0.0,
            variance: 1.0,
            std_dev: 1.0,
            skewness: 0.0,
            kurtosis: 0.0,
            sample_count: 10000,
        };

        params.adapt(&stats);
        params.reset();

        assert_eq!(params.delta_partial, params.base_delta_partial);
        assert_eq!(params.delta_final, params.base_delta_final);
        assert_eq!(params.buffer_size, params.base_buffer_size);
        assert_eq!(params.samples_seen, 0);
        assert_eq!(params.adapted, false);
    }

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

    #[test]
    fn test_process_block_basic() {
        let mut activity = ClusterBlockActivity::new(
            "Test Cluster".to_string(),
            MarkoutTime::Zero,
            1000,
            100
        );
        
        // Process a few blocks
        activity.process_block(1000, false);
        activity.process_block(1001, true);
        activity.process_block(1002, false);
        
        assert_eq!(activity.total_blocks(), 3, "Should count 3 total blocks");
        assert_eq!(activity.non_zero_blocks(), 1, "Should count 1 non-zero block");
        assert_eq!(activity.get_proportion(), 1.0/3.0, "Proportion should be 1/3");
    }
    
    #[test]
    fn test_process_block_duplicate() {
        let mut activity = ClusterBlockActivity::new(
            "Test Cluster".to_string(),
            MarkoutTime::Zero,
            1000,
            100
        );
        
        // Process the same block multiple times
        activity.process_block(1001, false);
        activity.process_block(1001, true);  // Same block, now with activity
        
        assert_eq!(activity.total_blocks(), 1, "Should count each block only once");
        assert_eq!(activity.non_zero_blocks(), 1, "Non-zero status should be updated");
    }
    
    #[test]
    fn test_process_block_out_of_range() {
        let mut activity = ClusterBlockActivity::new(
            "Test Cluster".to_string(),
            MarkoutTime::Zero,
            1000,
            10  // Small size to force reset
        );
        
        // Fill up to capacity
        for i in 0..10 {
            activity.process_block(1000 + i, i % 2 == 0);
        }
        
        // This should trigger flush_and_reset
        activity.process_block(1020, true);
        
        // Verify counts are accumulated correctly
        assert_eq!(activity.total_blocks(), 11, "Should have 10 from first chunk + 1 from new chunk");
        assert_eq!(activity.non_zero_blocks(), 6, "Should have 5 from first chunk + 1 from new chunk");
        
        // Process another block in the new range
        activity.process_block(1021, false);
        
        assert_eq!(activity.total_blocks(), 12, "Should now have 12 total blocks");
        assert_eq!(activity.non_zero_blocks(), 6, "Non-zero count should still be 6");
    }
    
    #[test]
    fn test_finalize_chunk() {
        let mut activity = ClusterBlockActivity::new(
            "Test Cluster".to_string(),
            MarkoutTime::Zero,
            1000,
            100
        );
        
        // Process some blocks
        for i in 0..5 {
            activity.process_block(1000 + i, i > 2);
        }
        
        // Finalize the chunk
        activity.finalize_chunk();
        
        // Verify the counts are moved to accumulated totals
        assert_eq!(activity.total_blocks(), 5, "Total should be preserved after finalization");
        assert_eq!(activity.non_zero_blocks(), 2, "Non-zero count should be preserved");
        
        // Process more blocks after finalization
        for i in 0..3 {
            activity.process_block(2000 + i, i == 1);
        }
        
        assert_eq!(activity.total_blocks(), 8, "Should now have 5 + 3 blocks");
        assert_eq!(activity.non_zero_blocks(), 3, "Should now have 2 + 1 non-zero blocks");
    }
    
    #[test]
    fn test_complex_sequence() {
        let mut activity = ClusterBlockActivity::new(
            "Test Cluster".to_string(), 
            MarkoutTime::Zero,
            1000,
            20
        );
        
        // First chunk - normal blocks
        for i in 0..10 {
            activity.process_block(1000 + i, i % 3 == 0);
        }
        
        // Process same blocks again (simulating multiple pools in cluster)
        for i in 0..10 {
            activity.process_block(1000 + i, i % 2 == 0);
        }
        
        assert_eq!(activity.total_blocks(), 10, "Should count each block only once");
        assert_eq!(activity.non_zero_blocks(), 7, "Should have blocks that are non-zero from either pass");
        
        // Process blocks in strictly increasing order
        activity.process_block(1020, false);
        activity.process_block(1025, true);
        
        // Trigger a reset by jumping ahead
        activity.process_block(1030, true);
        
        // Process blocks after the new base block (not before!)
        activity.process_block(1031, true);
        activity.process_block(1032, false);
        
        // Finalize everything
        activity.finalize_chunk();
        
        // Final verification
        assert_eq!(activity.total_blocks(), 15, "Should have correct total after complex sequence");
        assert_eq!(activity.non_zero_blocks(), 10, "Should have correct non-zero count");
        assert_eq!(activity.get_proportion(), 10.0/15.0, "Should calculate correct proportion");
    }

    #[test]
    fn test_out_of_order_blocks() {
        let mut activity = ClusterBlockActivity::new(
            "Test Cluster".to_string(),
            MarkoutTime::Zero,
            1000,
            50
        );
        
        // Process a sequence that will reset the base_block
        activity.process_block(1000, false);
        activity.process_block(1060, true);  // This causes a reset to base_block = 1060
        
        // Try to process a block before the new base_block
        activity.process_block(1050, true);  // This should be silently ignored
        
        // Process more blocks after the reset
        activity.process_block(1070, true);
        
        assert_eq!(activity.total_blocks(), 3, "Should count 1 from first chunk + 2 from second chunk");
        assert_eq!(activity.non_zero_blocks(), 2, "Should count 0 from first chunk + 2 from second chunk");
    }
}

