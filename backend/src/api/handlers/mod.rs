// Submodules
pub mod common;  // Common utilities used by other modules
pub mod health;  // Health check endpoint
pub mod clusters;  // Cluster analysis endpoints

// Data analysis endpoints
pub mod running_total;
pub mod ratios;
pub mod pool_totals;
pub mod max;
pub mod histogram;
pub mod nonzero;
pub mod percentile;
pub mod quartile;
pub mod moment; 

// Re-exports
pub use health::health_check;

// Data analysis endpoints
pub use running_total::get_running_total;
pub use ratios::get_lvr_ratios;
pub use pool_totals::get_pool_totals;
pub use max::get_max_lvr;
pub use histogram::get_lvr_histogram;
pub use nonzero::get_non_zero_proportion;
pub use percentile::get_percentile_band;
pub use quartile::get_quartile_plot;
pub use moment::get_distribution_metrics;

// Cluster analysis endpoints
pub use clusters::*;