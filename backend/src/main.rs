use anyhow::Result;
use backend::{init_logging, processor::ParallelLVRProcessor, serve, Validator, PrecomputedWriter};
use clap::{Parser, Subcommand};
use futures::future::BoxFuture;
use object_store::local::LocalFileSystem;
use object_store::ObjectStore;
use std::{path::PathBuf, sync::Arc};
use tracing::{error, info, warn};

// Block boundaries for processing
const START_BLOCK: u64 = 15537392;
const END_BLOCK: u64 = 20000000;

type ValidationCallback = for<'a> fn(&'a Arc<dyn ObjectStore>) -> BoxFuture<'a, Result<()>>;

#[derive(Debug, Parser)]
#[command(name = "lvr")]
#[command(about = "LVR data processor and API server")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Process LVR data
    Process {
        #[arg(short, long)]
        start_block: Option<u64>,

        #[arg(short, long)]
        end_block: Option<u64>,
    },
    /// Validate processed data
    Validate {
        #[arg(short, long)]
        data_dir: Option<PathBuf>,
    },
    /// Start the API server
    Serve {
        #[arg(short, long, default_value = "50001")]
        port: u16,

        #[arg(short = 'b', long, default_value = "127.0.0.1")]
        host: String,
    },
    /// Precompute analytical data
    Precompute,
}

fn ensure_directories() -> Result<PathBuf> {
    let data_dir = PathBuf::from("smeed");
    let output_dir = data_dir.join("intervals");
    let checkpoints_dir = data_dir.join("checkpoints");

    for dir in [&output_dir, &checkpoints_dir] {
        if !dir.exists() {
            std::fs::create_dir_all(dir)?;
            info!("Created directory: {:?}", dir);
        }
    }

    Ok(data_dir)
}

async fn run_validation(store: Arc<dyn ObjectStore>) -> Result<()> {
    info!("Running data validation");
    let validator = Validator::new(Arc::clone(&store));

    match validator.validate_all().await {
        Ok(results) => {
            let mut has_significant_errors = false;
            let mut has_minor_discrepancies = false;

            for (key, stats) in results {
                if stats.difference != 0 {
                    if stats.difference_percent.abs() > 1.0 {
                        has_significant_errors = true;
                        error!(
                            "Significant discrepancy for {}: Difference of {} ({:.2}%)",
                            key, stats.difference, stats.difference_percent
                        );
                    } else {
                        has_minor_discrepancies = true;
                        warn!(
                            "Minor discrepancy for {}: Difference of {} ({:.2}%)",
                            key, stats.difference, stats.difference_percent
                        );
                    }
                }
            }

            if has_significant_errors {
                return Err(anyhow::anyhow!(
                    "Validation failed with significant discrepancies"
                ));
            }

            if has_minor_discrepancies {
                warn!("Validation completed with minor discrepancies");
            } else {
                info!("Validation completed successfully with no discrepancies");
            }

            Ok(())
        }
        Err(e) => Err(anyhow::anyhow!("Validation failed: {}", e)),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    init_logging();

    // Parse command line arguments
    let cli = Cli::parse();

    // Load environment variables
    dotenv::dotenv().ok();

    // Ensure data directories exist
    let data_dir = ensure_directories()?;

    // Initialize object store
    let store: Arc<dyn ObjectStore> =
        Arc::new(LocalFileSystem::new_with_prefix(&data_dir)?);

    match cli.command {
        Commands::Process {
            start_block,
            end_block,
        } => {
            let start_block = start_block.unwrap_or(START_BLOCK);
            let end_block = end_block.unwrap_or(END_BLOCK);

            info!("Starting LVR data processing");

            let processor = Arc::new(
                ParallelLVRProcessor::new(start_block, end_block, Arc::clone(&store)).await?
            );

            // Define validation callback
            let validation_callback: Option<ValidationCallback> =
                Some(|store: &Arc<dyn ObjectStore>| {
                    Box::pin(async move { run_validation(Arc::clone(store)).await })
                });

            // Process blocks with validation after each chunk
            let processor_clone = Arc::clone(&processor);
            match processor_clone.process_blocks(validation_callback).await {
                Ok(_) => info!("Processing completed successfully"),
                Err(e) => {
                    error!("Processing failed: {}", e);
                    return Err(e);
                }
            }
        }
        Commands::Validate { data_dir } => {
            let data_dir = data_dir.unwrap_or_else(|| PathBuf::from("smeed"));
            info!("Starting validation of data in {:?}", data_dir);

            let store: Arc<dyn ObjectStore> =
                Arc::new(LocalFileSystem::new_with_prefix(data_dir)?);

            run_validation(Arc::clone(&store)).await?;
        }
        Commands::Serve { host, port } => {
            let store: Arc<dyn ObjectStore> = Arc::new(LocalFileSystem::new_with_prefix("smeed")?);

            info!("Starting API server using data from smeed/");
            serve(host, port, store).await?;
        }
        Commands::Precompute => {
            info!("Starting precomputation of analytical data");
            
            let writer = Arc::new(PrecomputedWriter::new(Arc::clone(&store)));
            
            info!("Computing running totals...");
            writer.write_running_totals().await?;
            
            info!("Computing LVR ratios...");
            writer.write_lvr_ratios().await?;
            
            info!("Computing pool totals...");
            writer.write_pool_totals().await?;
            
            info!("Computing max LVR values...");
            writer.write_max_lvr().await?;
            
            info!("Computing non-zero proportions...");
            writer.write_non_zero_proportions().await?;
            
            info!("Computing histograms...");
            writer.write_histograms().await?;
            
            info!("Computing percentile bands...");
            writer.write_percentile_bands().await?;
            
            info!("Computing quartile plots...");
            writer.write_quartile_plots().await?;
            
            info!("Computing cluster proportions...");
            writer.write_cluster_proportions().await?;
            
            info!("Computing cluster histograms...");
            writer.write_cluster_histograms().await?;
            
            info!("Computing monthly cluster totals...");
            writer.write_monthly_cluster_totals().await?;
            
            info!("Computing cluster non-zero metrics...");
            writer.write_cluster_non_zero().await?;

            info!("Computing distribution metrics...");
            writer.write_distribution_metrics().await?;
    
            info!("Successfully completed all precomputation tasks");
        }
    }

    Ok(())
}
