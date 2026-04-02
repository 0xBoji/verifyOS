use clap::Args;
use std::path::PathBuf;
use verifyos_cli::profiles::ScanProfile;

use crate::{AgentPackOutput, FailOnLevel, OutputFormat, TimingLevel};

#[derive(Debug, Clone, Args)]
pub struct ScanArgs {
    /// Path to the iOS App Bundle (.ipa or .app)
    #[arg(short, long)]
    pub app: Option<PathBuf>,

    /// Path to a sibling .xcodeproj for deeper analysis (auto-detected if sibling to app)
    #[arg(long)]
    pub project: Option<PathBuf>,

    /// Output format: table, json, sarif
    #[arg(long, value_enum)]
    pub format: Option<OutputFormat>,

    /// Baseline JSON file to suppress existing findings
    #[arg(long)]
    pub baseline: Option<PathBuf>,

    /// Write a clean Markdown report to a file (agent-friendly)
    #[arg(long)]
    pub md_out: Option<PathBuf>,

    /// Write a machine-readable fix pack for AI agents
    #[arg(long)]
    pub agent_pack: Option<PathBuf>,

    /// Agent pack output format: json, markdown, bundle
    #[arg(long, value_enum)]
    pub agent_pack_format: Option<AgentPackOutput>,

    /// Scan profile: basic or full
    #[arg(long, value_enum)]
    pub profile: Option<ScanProfile>,

    /// Exit with code 1 when findings reach this severity threshold
    #[arg(long, value_enum)]
    pub fail_on: Option<FailOnLevel>,

    /// Show timing telemetry: summary or full (defaults to summary when flag is present)
    #[arg(long, value_enum, num_args = 0..=1, default_missing_value = "summary")]
    pub timings: Option<TimingLevel>,

    /// Only run the listed rule IDs (repeat or comma-separate)
    #[arg(long, value_delimiter = ',', num_args = 1..)]
    pub include: Vec<String>,

    /// Skip the listed rule IDs (repeat or comma-separate)
    #[arg(long, value_delimiter = ',', num_args = 1..)]
    pub exclude: Vec<String>,

    /// Suppress the primary stdout report while keeping exit codes and explicit file outputs
    #[arg(long)]
    pub quiet: bool,

    /// Disable spinner/progress output for automation and non-interactive shells
    #[arg(long)]
    pub no_progress: bool,
}
