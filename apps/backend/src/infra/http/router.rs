use crate::app::ScanService;
use crate::infra::http::handlers::{health, scan_bundle};
use axum::routing::{get, post};
use axum::Router;
use axum::http::{HeaderValue, Method};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

pub fn build_router(scan_service: ScanService) -> Router {
    let cors = CorsLayer::new()
        .allow_origin([
            HeaderValue::from_static("http://localhost:3000"),
            HeaderValue::from_static("http://127.0.0.1:3000"),
        ])
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any);

    Router::new()
        .route("/healthz", get(health))
        .route("/api/v1/scan", post(scan_bundle))
        .with_state(scan_service)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}
