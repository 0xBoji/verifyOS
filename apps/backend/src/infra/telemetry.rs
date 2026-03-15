use tracing_subscriber::EnvFilter;

pub fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("verifyos_backend=info,tower_http=info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}
