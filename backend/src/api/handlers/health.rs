use axum::response::{Json, IntoResponse};
use axum::http::StatusCode;
use time::OffsetDateTime;
use crate::HealthResponse;

pub async fn health_check() -> impl IntoResponse {
    let response = HealthResponse {
        status: "OK",
        version: env!("CARGO_PKG_VERSION"),
        timestamp: OffsetDateTime::now_utc().to_string(),
    };

    (StatusCode::OK, Json(response))
}