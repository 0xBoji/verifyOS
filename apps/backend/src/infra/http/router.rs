use crate::app::ScanService;
use crate::infra::http::handlers::{health, handoff_bundle, scan_bundle};
use axum::routing::{get, post};
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

pub fn build_router(scan_service: ScanService) -> Router {
    let cors = CorsLayer::permissive();

    Router::new()
        .route("/healthz", get(health))
        .route("/api/v1/scan", post(scan_bundle))
        .route("/api/v1/handoff", post(handoff_bundle))
        .with_state(scan_service)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}
