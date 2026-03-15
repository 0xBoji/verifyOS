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
        project_path: Option<&Path>,
    ) -> Result<ScanResponse, ScanError> {
        let started = Instant::now();
        let profile = match request.profile {
            Some(ScanProfileInput::Basic) => ScanProfile::Basic,
            Some(ScanProfileInput::Full) | None => ScanProfile::Full,
        };

        let mut engine = Engine::new();
        let selection = RuleSelection::default();
        register_rules(&mut engine, profile, &selection);

        if let Some(project_path) = project_path {
            if let Some(project) = load_xcode_project(project_path) {
                engine.xcode_project = Some(project);
            }
        }

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

fn load_xcode_project(
    path: &Path,
) -> Option<verifyos_cli::parsers::xcode_parser::XcodeProject> {
    let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
    if extension.eq_ignore_ascii_case("xcworkspace") {
        match verifyos_cli::parsers::xcworkspace_parser::Xcworkspace::from_path(path) {
            Ok(workspace) => {
                for project_path in workspace.project_paths {
                    match verifyos_cli::parsers::xcode_parser::XcodeProject::from_path(
                        &project_path,
                    ) {
                        Ok(project) => return Some(project),
                        Err(err) => {
                            eprintln!(
                                "Warning: Failed to load Xcode project at {}: {}",
                                project_path.display(),
                                err
                            );
                        }
                    }
                }
                eprintln!(
                    "Warning: No usable .xcodeproj found in workspace {}",
                    path.display()
                );
                None
            }
            Err(err) => {
                eprintln!(
                    "Warning: Failed to read Xcode workspace at {}: {}",
                    path.display(),
                    err
                );
                None
            }
        }
    } else if extension.eq_ignore_ascii_case("xcodeproj") {
        match verifyos_cli::parsers::xcode_parser::XcodeProject::from_path(path) {
            Ok(project) => Some(project),
            Err(err) => {
                eprintln!(
                    "Warning: Failed to load Xcode project at {}: {}",
                    path.display(),
                    err
                );
                None
            }
        }
    } else {
        eprintln!(
            "Warning: Unsupported project type at {} (expected .xcodeproj or .xcworkspace)",
            path.display()
        );
        None
    }
}
