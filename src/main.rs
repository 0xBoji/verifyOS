use clap::{Parser, ValueEnum};
use indicatif::{ProgressBar, ProgressStyle};
use miette::{IntoDiagnostic, Result};
use std::collections::HashSet;
use std::path::PathBuf;

use verifyos_cli::core::engine::Engine;
use verifyos_cli::profiles::{
    available_rule_ids, normalize_rule_id, register_rules, RuleSelection, ScanProfile,
};
use verifyos_cli::report::{
    apply_baseline, build_report, render_json, render_markdown, render_sarif, render_table,
    should_exit_with_failure, FailOn,
};

#[derive(Clone, Debug, ValueEnum)]
enum OutputFormat {
    Table,
    Json,
    Sarif,
}

#[derive(Clone, Debug, ValueEnum)]
enum Profile {
    Basic,
    Full,
}

#[derive(Clone, Debug, ValueEnum)]
enum FailOnLevel {
    Off,
    Error,
    Warning,
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

    /// Write a clean Markdown report to a file (agent-friendly)
    #[arg(long)]
    md_out: Option<PathBuf>,

    /// Scan profile: basic or full
    #[arg(long, value_enum, default_value_t = Profile::Full)]
    profile: Profile,

    /// Exit with code 1 when findings reach this severity threshold
    #[arg(long, value_enum, default_value_t = FailOnLevel::Error)]
    fail_on: FailOnLevel,

    /// Only run the listed rule IDs (repeat or comma-separate)
    #[arg(long, value_delimiter = ',', num_args = 1..)]
    include: Vec<String>,

    /// Skip the listed rule IDs (repeat or comma-separate)
    #[arg(long, value_delimiter = ',', num_args = 1..)]
    exclude: Vec<String>,
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
    let profile = match args.profile {
        Profile::Basic => ScanProfile::Basic,
        Profile::Full => ScanProfile::Full,
    };
    let fail_on = match args.fail_on {
        FailOnLevel::Off => FailOn::Off,
        FailOnLevel::Error => FailOn::Error,
        FailOnLevel::Warning => FailOn::Warning,
    };
    let selection = build_rule_selection(profile, &args.include, &args.exclude)?;
    register_rules(&mut engine, profile, &selection);

    // 4. Run the Engine
    let results = engine
        .run(&args.app)
        .map_err(|e| miette::miette!("Engine orchestrator failed: {}", e))?;

    // 5. Stop the spinner
    pb.finish_with_message("Analysis complete!");

    // 6. Build report and apply baseline (if any)
    let mut report = build_report(results);
    let mut suppressed = None;
    if let Some(path) = args.baseline {
        let baseline_raw = std::fs::read_to_string(path).into_diagnostic()?;
        let baseline: verifyos_cli::report::ReportData =
            serde_json::from_str(&baseline_raw).into_diagnostic()?;
        let summary = apply_baseline(&mut report, &baseline);
        suppressed = Some(summary.suppressed);
    }

    // 7. Render output
    match args.format {
        OutputFormat::Table => println!("{}", render_table(&report)),
        OutputFormat::Json => println!("{}", render_json(&report).into_diagnostic()?),
        OutputFormat::Sarif => println!("{}", render_sarif(&report).into_diagnostic()?),
    }

    if let Some(path) = args.md_out {
        let markdown = render_markdown(&report, suppressed);
        std::fs::write(path, markdown).into_diagnostic()?;
    }

    // 8. Exit with code 1 if findings meet the configured threshold
    if should_exit_with_failure(&report, fail_on) {
        std::process::exit(1);
    }

    Ok(())
}

fn build_rule_selection(
    profile: ScanProfile,
    include: &[String],
    exclude: &[String],
) -> Result<RuleSelection> {
    let available: HashSet<String> = available_rule_ids(profile).into_iter().collect();
    let include = normalize_requested_rules(include, &available, "--include")?;
    let exclude = normalize_requested_rules(exclude, &available, "--exclude")?;

    Ok(RuleSelection { include, exclude })
}

fn normalize_requested_rules(
    values: &[String],
    available: &HashSet<String>,
    flag_name: &str,
) -> Result<HashSet<String>> {
    let mut normalized = HashSet::new();

    for value in values {
        let rule_id = normalize_rule_id(value);
        if !available.contains(&rule_id) {
            let mut available_ids: Vec<&str> = available.iter().map(String::as_str).collect();
            available_ids.sort_unstable();
            return Err(miette::miette!(
                "{flag_name} contains unknown rule ID `{}`. Available rule IDs for this profile: {}",
                value,
                available_ids.join(", ")
            ));
        }
        normalized.insert(rule_id);
    }

    Ok(normalized)
}
