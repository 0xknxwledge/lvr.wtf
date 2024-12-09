use anyhow::Result;
use clap::{Parser, Subcommand};
use object_store::local::LocalFileSystem;
use std::{path::PathBuf, sync::Arc};
use tokio;
use tracing::{info, error, warn};
use dotenv::dotenv;
use object_store::ObjectStore;
use backend::{
    processor::ParallelLVRProcessor, 
    init_logging, 
    Validator,
    serve};
use futures::future::BoxFuture;

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
        /// Starting block number
        #[arg(short, long)]
        start_block: Option<u64>,
        
        /// Ending block number
        #[arg(short, long)]
        end_block: Option<u64>,
    },
    /// Validate processed data
    Validate {
        /// Directory containing the data
        #[arg(short, long)]
        data_dir: Option<PathBuf>,
    },
    /// Start the API server
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value = "3000")]
        port: u16,
        
        /// Host address to bind to
        #[arg(short = 'b', long, default_value = "127.0.0.1")]
        host: String,
    },
}

fn ensure_directories() -> Result<PathBuf> {
    let data_dir = PathBuf::from("data");
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

async fn run_validation(store: &Arc<dyn ObjectStore>) -> Result<()> {
    info!("Running data validation");
    let validator = Validator::new(store.clone());
    
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
                Err(anyhow::anyhow!("Validation failed with significant discrepancies"))
            } else if has_minor_discrepancies {
                warn!("Validation completed with minor discrepancies");
                Ok(())
            } else {
                info!("Validation completed successfully with no discrepancies");
                Ok(())
            }
        }
        Err(e) => Err(anyhow::anyhow!("Validation failed: {}", e))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    crate::init_logging();
    
    // Parse command line arguments
    let cli = Cli::parse();

    // Load .env file
    match dotenv() {
        Ok(_) => info!("Loaded environment variables from .env file"),
        Err(e) => error!("Failed to load .env file: {}", e),
    }

    // Ensure data directories exist
    let data_dir = ensure_directories()?;

    // Initialize object store
    let store: Arc<dyn object_store::ObjectStore> = Arc::new(
        LocalFileSystem::new_with_prefix(&data_dir)?
    );

    match cli.command {
        Commands::Process { start_block, end_block } => {
            let start_block = start_block.unwrap_or(START_BLOCK);
            let end_block = end_block.unwrap_or(END_BLOCK);
            
            info!("Starting LVR data processing");
            
            let processor = ParallelLVRProcessor::new(
                start_block,
                end_block,
                store.clone()
            ).await?;

            // Create validation callback
            let validation_callback: Option<ValidationCallback> = Some(|store: &Arc<dyn ObjectStore>| {
                Box::pin(async move {
                    match run_validation(store).await {
                        Ok(_) => Ok(()),
                        Err(e) => {
                            error!("Validation error during processing: {}", e);
                            Err(e)
                        }
                    }
                })
            });

            // Process blocks with validation after each chunk
            match processor.process_blocks(validation_callback).await {
                Ok(_) => info!("Processing completed successfully"),
                Err(e) => {
                    error!("Processing failed: {}", e);
                    return Err(e);
                }
            }
        },
        Commands::Validate { data_dir } => {
            let data_dir = data_dir.unwrap_or_else(|| PathBuf::from("data"));
            info!("Starting validation of data in {:?}", data_dir);
            
            let store: Arc<dyn object_store::ObjectStore> = 
                Arc::new(LocalFileSystem::new_with_prefix(data_dir)?);
            
            run_validation(&store).await?;
        },
        Commands::Serve { host, port } => {
            info!("Starting API server");
            serve(host, port, store).await?;
        }
    }

    Ok(())
}