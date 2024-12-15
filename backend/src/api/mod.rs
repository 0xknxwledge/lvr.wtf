mod handlers;
mod types;
mod state;
pub use handlers::*;
pub use types::*;
pub use state::*;

use tokio::net::TcpListener;
use axum::{
    Router,
    routing::get
};
use tower_http::cors::{Any, CorsLayer};
use std::sync::Arc;
use std::net::SocketAddr;
use object_store::ObjectStore;
use tracing::info;
use anyhow::Result;
use std::time::Duration;

pub async fn serve(host: String, port: u16, store: Arc<dyn ObjectStore>) -> Result<()> {
    // Create application state
    let state = Arc::new(AppState::new(store));

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers(Any)
        .max_age(Duration::from_secs(3600));

    // Build router with routes and middleware
    let app = Router::new()
        // Core endpoints
        .route("/health", get(health_check))
        
        // Data analysis endpoints
        .route("/running_total", get(get_running_total))
        .route("/ratios", get(get_lvr_ratios))
        .route("/pool_totals", get(get_pool_totals))
        .route("/max_lvr", get(get_max_lvr))
        .route("/histogram", get(get_lvr_histogram))
        .route("/non_zero_proportion", get(get_non_zero_proportion))
        .route("/percentile_band", get(get_percentile_band))
        
        // Cluster analysis endpoints
        .route("/clusters/pie", get(get_cluster_proportion))
        .route("/clusters/histogram", get(get_cluster_histogram))
        .route("/clusters/monthly", get(get_monthly_cluster_totals))
        .layer(cors)
        .with_state(state);

    // Create socket address
    let addr = format!("{}:{}", host, port)
        .parse::<SocketAddr>()?;

    // Create TCP listener
    let listener = TcpListener::bind(&addr).await?;

    info!("API server listening on {}", addr);

    // Start server
    axum::serve(listener, app)
        .await
        .map_err(|e| anyhow::anyhow!("Server error: {}", e))?;

    Ok(())
}