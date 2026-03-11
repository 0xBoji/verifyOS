use clap::{Parser, ValueEnum};
use indicatif::{ProgressBar, ProgressStyle};
use miette::{IntoDiagnostic, Result};
use std::path::PathBuf;

use verifyos_cli::core::engine::Engine;
use verifyos_cli::report::{apply_baseline, build_report, render_json, render_sarif, render_table};
use verifyos_cli::rules::core::{RuleStatus, Severity};
use verifyos_cli::rules::entitlements::EntitlementsMismatchRule;
use verifyos_cli::rules::info_plist::InfoPlistRequiredKeysRule;
use verifyos_cli::rules::info_plist::UsageDescriptionsRule;
use verifyos_cli::rules::info_plist::UsageDescriptionsValueRule;
use verifyos_cli::rules::info_plist::InfoPlistCapabilitiesRule;
use verifyos_cli::rules::entitlements::EntitlementsProvisioningMismatchRule;
use verifyos_cli::rules::permissions::CameraUsageDescriptionRule;
use verifyos_cli::rules::privacy::MissingPrivacyManifestRule;

#[derive(Clone, Debug, ValueEnum)]
enum OutputFormat {
    Table,
    Json,
    Sarif,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the iOS App Bundle (.ipa or .app)
    #[arg(short, long)]
    app: PathBuf,

    /// Output format: table, json, sarif
    #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
    format: OutputFormat,

    /// Baseline JSON file to suppress existing findings
    #[arg(long)]
    baseline: Option<PathBuf>,
}

fn main() -> Result<()> {
    // 1. Parse CLI arguments
    let args = Args::parse();

    // 2. Initialize spinner
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.green} [{elapsed_precise}] {msg}")
            .into_diagnostic()?,
    );
    pb.set_message("Analyzing app bundle...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    // 3. Initialize Core Engine
    let mut engine = Engine::new();

    // Register the current rules
    engine.register_rule(Box::new(MissingPrivacyManifestRule));
    engine.register_rule(Box::new(CameraUsageDescriptionRule));
    engine.register_rule(Box::new(UsageDescriptionsRule));
    engine.register_rule(Box::new(UsageDescriptionsValueRule));
    engine.register_rule(Box::new(InfoPlistRequiredKeysRule));
    engine.register_rule(Box::new(InfoPlistCapabilitiesRule));
    engine.register_rule(Box::new(EntitlementsMismatchRule));
    engine.register_rule(Box::new(EntitlementsProvisioningMismatchRule));

    // 4. Run the Engine
    let results = engine
        .run(&args.app)
        .map_err(|e| miette::miette!("Engine orchestrator failed: {}", e))?;

    // 5. Stop the spinner
    pb.finish_with_message("Analysis complete!");

    // 6. Build report and apply baseline (if any)
    let mut report = build_report(results);
    if let Some(path) = args.baseline {
        let baseline_raw = std::fs::read_to_string(path).into_diagnostic()?;
        let baseline: verifyos_cli::report::ReportData =
            serde_json::from_str(&baseline_raw).into_diagnostic()?;
        let _ = apply_baseline(&mut report, &baseline);
    }

    // 7. Render output
    match args.format {
        OutputFormat::Table => println!("{}", render_table(&report)),
        OutputFormat::Json => println!("{}", render_json(&report).into_diagnostic()?),
        OutputFormat::Sarif => println!("{}", render_sarif(&report).into_diagnostic()?),
    }

    // 8. Exit with code 1 if any Error severity check failed
    let has_errors = report.results.iter().any(|r| {
        matches!(r.status, RuleStatus::Fail | RuleStatus::Error)
            && matches!(r.severity, Severity::Error)
    });

    if has_errors {
        std::process::exit(1);
    }

    Ok(())
}
