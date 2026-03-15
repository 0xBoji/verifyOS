use crate::app::ScanService;
use crate::infra::http::handlers::{health, scan_bundle};
use axum::routing::{get, post};
use axum::Router;
use tower_http::trace::TraceLayer;

pub fn build_router(scan_service: ScanService) -> Router {
    Router::new()
        .route("/healthz", get(health))
        .route("/api/v1/scan", post(scan_bundle))
        .with_state(scan_service)
        .layer(TraceLayer::new_for_http())
}
