use verifyos_backend::app::{AppState, RateLimiter, ScanService};
use verifyos_backend::infra::http::router::build_router;

#[tokio::main]
async fn main() {
    verifyos_backend::infra::telemetry::init_tracing();
    let service = ScanService::new();
    let rate_limit = RateLimiter::new();
    let app = build_router(AppState::new(service, rate_limit));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:7070")
        .await
        .expect("bind backend listener");
    axum::serve(listener, app).await.expect("serve backend");
}
