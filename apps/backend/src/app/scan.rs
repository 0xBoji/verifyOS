use crate::domain::{ScanProfileInput, ScanRequest, ScanResponse};
use std::path::Path;
use std::time::Instant;
use thiserror::Error;
use verifyos_cli::core::engine::Engine;
use verifyos_cli::profiles::{register_rules, RuleSelection, ScanProfile};
use verifyos_cli::report::build_report;

#[derive(Debug, Error)]
pub enum ScanError {
    #[error("scan failed: {0}")]
    ScanFailed(String),
}

#[derive(Clone, Copy)]
pub struct ScanService;

impl ScanService {
    pub fn new() -> Self {
        Self
    }

    pub fn run_scan<P: AsRef<Path>>(
        &self,
        request: ScanRequest,
        bundle_path: P,
    ) -> Result<ScanResponse, ScanError> {
        let started = Instant::now();
        let profile = match request.profile {
            Some(ScanProfileInput::Basic) => ScanProfile::Basic,
            Some(ScanProfileInput::Full) | None => ScanProfile::Full,
        };

        let mut engine = Engine::new();
        let selection = RuleSelection::default();
        register_rules(&mut engine, profile, &selection);

        let run = engine
            .run(bundle_path)
            .map_err(|err| ScanError::ScanFailed(err.to_string()))?;

        let report = build_report(run.results, run.total_duration_ms, run.cache_stats);
        Ok(ScanResponse {
            report,
            warnings: vec![format!(
                "scan completed in {duration}ms",
                duration = started.elapsed().as_millis()
            )],
        })
    }
}
