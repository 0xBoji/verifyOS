use clap::{Parser, Subcommand, ValueEnum};
use comfy_table::Table;
use indicatif::{ProgressBar, ProgressStyle};
use miette::{IntoDiagnostic, Result};
use std::io::{stderr, IsTerminal};
use std::path::PathBuf;

mod commands;

use commands::analyze_size::{run as run_analyze_size_command, AnalyzeSizeArgs};
use commands::doctor::{run as run_doctor_command, DoctorArgs};
use commands::handoff::{run as run_handoff_command, HandoffArgs};
use commands::init::{run as run_init_command, InitArgs};
use commands::lsp::{run as run_lsp_command, LspArgs};
use commands::pr_comment::{run as run_pr_comment_command, PrCommentArgs};
use commands::scan::ScanArgs;
use commands::summary::{run as run_summary_command, SummaryArgs};
use commands::support::{
    agent_pack_format_key, build_rule_selection, fail_on_key, output_format_key,
    parse_agent_pack_format, parse_fail_on, parse_output_format, parse_profile, parse_timing_mode,
    profile_key, timing_key,
};

use verifyos_cli::config::{load_file_config, resolve_runtime_config, CliOverrides, FileConfig};
use verifyos_cli::core::engine::Engine;
use verifyos_cli::profiles::{
    register_rules, rule_detail, rule_inventory, RuleDetailItem, RuleInventoryItem, RuleSelection,
    ScanProfile,
};
use verifyos_cli::report::{
    apply_agent_pack_baseline, apply_baseline, build_agent_pack, build_report,
    render_agent_pack_markdown, render_json, render_markdown, render_sarif, render_table,
    should_exit_with_failure, AgentPackFormat,
};

const HELP_BANNER: &str = r#"
 _    ______  ______
| |  / / __ \/ ____/
| | / / / / / /     
| |/ / /_/ / /___   
|___/\____/\____/   

verify-OS
"#;

#[derive(Clone, Copy, Debug, ValueEnum)]
enum OutputFormat {
    Table,
    Json,
    Sarif,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum FailOnLevel {
    Off,
    Error,
    Warning,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum TimingLevel {
    Summary,
    Full,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum AgentPackOutput {
    Json,
    Markdown,
    Bundle,
}

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    long_about = None,
    before_help = HELP_BANNER,
    subcommand_negates_reqs = true
)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Optional config file path. If omitted, verifyos.toml is used when present
    #[arg(long)]
    config: Option<PathBuf>,

    #[command(flatten)]
    scan: ScanArgs,

    /// List all available rules and exit
    #[arg(long)]
    list_rules: bool,

    /// Show details for a single rule ID and exit
    #[arg(long)]
    show_rule: Option<String>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Scan an .ipa or .app bundle for App Store rejection risks
    Scan(ScanArgs),
    /// Create or update AGENTS.md with verifyOS-cli guidance
    Init(InitArgs),
    /// Inspect IPA/app bundle size hotspots and category breakdowns
    AnalyzeSize(AnalyzeSizeArgs),
    /// Verify verifyOS-cli config and generated agent assets
    Doctor(DoctorArgs),
    /// Refresh the full agent handoff bundle from a fresh scan
    Handoff(HandoffArgs),
    /// Render a sticky PR comment body from an output directory
    PrComment(PrCommentArgs),
    /// Start the verifyOS Language Server (LSP)
    Lsp(LspArgs),
    /// Summarize an existing verifyOS JSON report for quick triage
    Summary(SummaryArgs),
}

fn main() -> Result<()> {
    // 1. Parse CLI arguments
    let args = Args::parse();
    let file_config = load_file_config(args.config.as_deref())?;
    if let Some(Commands::Init(init)) = args.command {
        return run_init_command(init, &file_config);
    }
    if let Some(Commands::AnalyzeSize(analyze_size)) = args.command {
        return run_analyze_size_command(analyze_size);
    }
    if let Some(Commands::Doctor(doctor)) = args.command {
        return run_doctor_command(doctor, &file_config);
    }
    if let Some(Commands::Handoff(handoff)) = args.command {
        return run_handoff_command(handoff, &file_config);
    }
    if let Some(Commands::PrComment(pr_comment)) = args.command {
        return run_pr_comment_command(pr_comment);
    }
    if let Some(Commands::Lsp(lsp)) = args.command {
        return run_lsp_command(lsp);
    }
    if let Some(Commands::Summary(summary)) = args.command {
        return run_summary_command(summary);
    }
    if let Some(Commands::Scan(scan)) = args.command {
        return run_scan(scan, file_config);
    }

    let runtime = resolve_runtime_config(file_config, scan_overrides(&args.scan));
    let output_format = parse_output_format(&runtime.format)?;
    if args.list_rules {
        render_rule_inventory(output_format)?;
        return Ok(());
    }
    if let Some(rule_id) = args.show_rule.as_deref() {
        render_rule_detail(rule_id, output_format)?;
        return Ok(());
    }
    let profile = parse_profile(&runtime.profile)?;
    let fail_on = parse_fail_on(&runtime.fail_on)?;
    let timing_mode = parse_timing_mode(&runtime.timings)?;
    let agent_pack_format = parse_agent_pack_format(&runtime.agent_pack_format)?;
    run_scan_with_runtime(
        &args.scan,
        runtime,
        output_format,
        profile,
        fail_on,
        timing_mode,
        agent_pack_format,
    )
}

fn run_scan(scan: ScanArgs, file_config: FileConfig) -> Result<()> {
    let runtime = resolve_runtime_config(file_config, scan_overrides(&scan));
    let output_format = parse_output_format(&runtime.format)?;
    let profile = parse_profile(&runtime.profile)?;
    let fail_on = parse_fail_on(&runtime.fail_on)?;
    let timing_mode = parse_timing_mode(&runtime.timings)?;
    let agent_pack_format = parse_agent_pack_format(&runtime.agent_pack_format)?;
    run_scan_with_runtime(
        &scan,
        runtime,
        output_format,
        profile,
        fail_on,
        timing_mode,
        agent_pack_format,
    )
}

fn run_scan_with_runtime(
    scan: &ScanArgs,
    runtime: verifyos_cli::config::RuntimeConfig,
    output_format: OutputFormat,
    profile: ScanProfile,
    fail_on: verifyos_cli::report::FailOn,
    timing_mode: verifyos_cli::report::TimingMode,
    agent_pack_format: AgentPackFormat,
) -> Result<()> {
    let app_path = scan
        .app
        .as_ref()
        .ok_or_else(|| miette::miette!("`--app <path>` is required for scans"))?;
    let ui = ScanUi::new(scan.quiet, scan.no_progress)?;

    // 2. Initialize spinner
    ui.set_message("Analyzing app bundle...");

    // 3. Initialize Core Engine
    let mut engine = Engine::new();
    let selection = build_rule_selection(profile, &runtime.include, &runtime.exclude)?;
    register_rules(&mut engine, profile, &selection);

    // 4. Handle Xcode Project
    let project_path = scan
        .project
        .clone()
        .or_else(|| auto_detect_project_path(app_path));

    if let Some(path) = project_path {
        ui.set_message(format!("Loading Xcode project: {}...", path.display()));
        if let Some(project) = load_xcode_project(&path) {
            engine.xcode_project = Some(project);
        }
    }

    // 5. Run the Engine
    ui.set_message("Analyzing app bundle...");
    let run = engine
        .run(app_path)
        .map_err(|e| miette::miette!("Engine orchestrator failed: {}", e))?;

    // 5. Stop the spinner
    ui.finish("Analysis complete!");

    // 6. Build report and apply baseline (if any)
    let mut report = build_report(run.results, run.total_duration_ms, run.cache_stats);
    let mut suppressed = None;
    if let Some(path) = runtime.baseline {
        let baseline_raw = std::fs::read_to_string(path).into_diagnostic()?;
        let baseline: verifyos_cli::report::ReportData =
            serde_json::from_str(&baseline_raw).into_diagnostic()?;
        let summary = apply_baseline(&mut report, &baseline);
        suppressed = Some(summary.suppressed);
    }

    // 7. Render output
    if !scan.quiet {
        match output_format {
            OutputFormat::Table => println!("{}", render_table(&report, timing_mode)),
            OutputFormat::Json => println!("{}", render_json(&report).into_diagnostic()?),
            OutputFormat::Sarif => println!("{}", render_sarif(&report).into_diagnostic()?),
        }
    }

    if let Some(path) = runtime.md_out {
        let markdown = render_markdown(&report, suppressed, timing_mode);
        std::fs::write(path, markdown).into_diagnostic()?;
    }

    if let Some(path) = runtime.agent_pack {
        let agent_pack = build_agent_pack(&report);
        write_agent_pack(&path, &agent_pack, agent_pack_format)?;
    }

    // 8. Exit with code 1 if findings meet the configured threshold
    if should_exit_with_failure(&report, fail_on) {
        std::process::exit(1);
    }

    Ok(())
}

fn scan_overrides(scan: &ScanArgs) -> CliOverrides {
    CliOverrides {
        format: scan.format.map(output_format_key),
        baseline: scan.baseline.clone(),
        md_out: scan.md_out.clone(),
        agent_pack: scan.agent_pack.clone(),
        agent_pack_format: scan.agent_pack_format.map(agent_pack_format_key),
        profile: scan.profile.map(profile_key),
        fail_on: scan.fail_on.map(fail_on_key),
        timings: scan.timings.map(timing_key),
        include: scan.include.clone(),
        exclude: scan.exclude.clone(),
    }
}

fn auto_detect_project_path(app_path: &std::path::Path) -> Option<PathBuf> {
    let mut workspace = None;
    let mut project = None;
    if let Some(parent) = app_path.parent() {
        if let Ok(entries) = std::fs::read_dir(parent) {
            for entry in entries.flatten() {
                let path = entry.path();
                if workspace.is_none()
                    && path
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("xcworkspace"))
                {
                    workspace = Some(path.clone());
                }
                if project.is_none() && path.extension().is_some_and(|ext| ext == "xcodeproj") {
                    project = Some(path);
                }
            }
        }
    }
    workspace.or(project)
}

struct ScanUi {
    progress: Option<ProgressBar>,
}

impl ScanUi {
    fn new(quiet: bool, no_progress: bool) -> Result<Self> {
        let progress = if quiet || no_progress || !stderr().is_terminal() {
            None
        } else {
            let progress = ProgressBar::new_spinner();
            progress.set_style(
                ProgressStyle::default_spinner()
                    .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
                    .template("{spinner:.green} [{elapsed_precise}] {msg}")
                    .into_diagnostic()?,
            );
            progress.enable_steady_tick(std::time::Duration::from_millis(100));
            Some(progress)
        };
        Ok(Self { progress })
    }

    fn set_message(&self, message: impl Into<String>) {
        if let Some(progress) = &self.progress {
            progress.set_message(message.into());
        }
    }

    fn finish(&self, message: &str) {
        if let Some(progress) = &self.progress {
            progress.finish_with_message(message.to_string());
        }
    }
}

fn write_agent_pack(
    path: &std::path::Path,
    agent_pack: &verifyos_cli::report::AgentPack,
    format: AgentPackFormat,
) -> Result<()> {
    match format {
        AgentPackFormat::Json => {
            let json = serde_json::to_string_pretty(agent_pack).into_diagnostic()?;
            std::fs::write(path, json).into_diagnostic()?;
        }
        AgentPackFormat::Markdown => {
            let markdown = render_agent_pack_markdown(agent_pack);
            std::fs::write(path, markdown).into_diagnostic()?;
        }
        AgentPackFormat::Bundle => {
            std::fs::create_dir_all(path).into_diagnostic()?;
            let json_path = path.join("agent-pack.json");
            let markdown_path = path.join("agent-pack.md");
            let json = serde_json::to_string_pretty(agent_pack).into_diagnostic()?;
            let markdown = render_agent_pack_markdown(agent_pack);
            std::fs::write(json_path, json).into_diagnostic()?;
            std::fs::write(markdown_path, markdown).into_diagnostic()?;
        }
    }

    Ok(())
}

fn run_scan_for_agent_pack(
    app_path: &std::path::Path,
    profile: ScanProfile,
    baseline_path: Option<&std::path::Path>,
) -> Result<verifyos_cli::report::AgentPack> {
    let mut engine = Engine::new();
    let selection = RuleSelection::default();
    register_rules(&mut engine, profile, &selection);

    let run = engine
        .run(app_path)
        .map_err(|e| miette::miette!("Engine orchestrator failed: {}", e))?;
    let report = build_report(run.results, run.total_duration_ms, run.cache_stats);
    let mut agent_pack = build_agent_pack(&report);
    if let Some(path) = baseline_path {
        let baseline_raw = std::fs::read_to_string(path).into_diagnostic()?;
        let baseline: verifyos_cli::report::ReportData =
            serde_json::from_str(&baseline_raw).into_diagnostic()?;
        apply_agent_pack_baseline(&mut agent_pack, &baseline);
    }
    Ok(agent_pack)
}

fn load_xcode_project(
    path: &std::path::Path,
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

fn render_rule_inventory(output_format: OutputFormat) -> Result<()> {
    let inventory = rule_inventory();
    match output_format {
        OutputFormat::Table => {
            println!("{}", render_rule_inventory_table(&inventory));
            Ok(())
        }
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&inventory).into_diagnostic()?
            );
            Ok(())
        }
        OutputFormat::Sarif => Err(miette::miette!(
            "`--list-rules` supports only table or json output"
        )),
    }
}

fn render_rule_inventory_table(items: &[RuleInventoryItem]) -> String {
    let mut table = Table::new();
    table.set_header(vec![
        "Rule ID",
        "Name",
        "Category",
        "Severity",
        "Default ScanProfiles",
    ]);

    for item in items {
        table.add_row(vec![
            item.rule_id.clone(),
            item.name.clone(),
            format!("{:?}", item.category),
            format!("{:?}", item.severity),
            item.default_profiles.join(", "),
        ]);
    }

    table.to_string()
}

fn render_rule_detail(rule_id: &str, output_format: OutputFormat) -> Result<()> {
    let Some(detail) = rule_detail(rule_id) else {
        return Err(miette::miette!("Unknown rule ID `{}`", rule_id));
    };

    match output_format {
        OutputFormat::Table => {
            println!("{}", render_rule_detail_table(&detail));
            Ok(())
        }
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&detail).into_diagnostic()?
            );
            Ok(())
        }
        OutputFormat::Sarif => Err(miette::miette!(
            "`--show-rule` supports only table or json output"
        )),
    }
}

fn render_rule_detail_table(item: &RuleDetailItem) -> String {
    let mut table = Table::new();
    table.set_header(vec!["Field", "Value"]);
    table.add_row(vec!["Rule ID", item.rule_id.as_str()]);
    table.add_row(vec!["Name", item.name.as_str()]);
    table.add_row(vec!["Category", &format!("{:?}", item.category)]);
    table.add_row(vec!["Severity", &format!("{:?}", item.severity)]);
    table.add_row(vec![
        "Default ScanProfiles",
        &item.default_profiles.join(", "),
    ]);
    table.add_row(vec!["Recommendation", item.recommendation.as_str()]);
    table.to_string()
}
