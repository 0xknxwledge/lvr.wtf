mod handlers;
mod types;
mod state;
pub use handlers::*;
pub use types::*;
pub use state::*;

use tokio::net::TcpListener;
use axum::{
    Router,
    routing::get,
};
use tower_http::cors::{CorsLayer, Any};
use std::sync::Arc;
use std::net::SocketAddr;
use object_store::ObjectStore;
use tracing::info;
use anyhow::Result;

pub async fn serve(host: String, port: u16, store: Arc<dyn ObjectStore>) -> Result<()> {
    // Create application state
    let state = Arc::new(AppState::new(store));

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([axum::http::Method::GET])
        .allow_headers([axum::http::header::CONTENT_TYPE]);

    // Build router with routes and middleware
    let app = Router::new()
        .route("/running_total", get(handlers::get_running_total))
        .route("/ratios", get(handlers::get_lvr_ratios))
        .route("/pool_totals", get(handlers::get_pool_totals))
        .route("/max_lvr", get(handlers::get_max_lvr))
        .route("/histogram", get(handlers::get_lvr_histogram))
        .route("/non_zero_proportion", get(handlers::get_non_zero_proportion))
        .route("/percentile_band", get(handlers::get_percentile_band))
        .route("/boxplot_lvr", get(handlers::get_boxplot_lvr))
        .route("/health", get(health_check))
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