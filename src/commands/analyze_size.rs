use clap::{Parser, ValueEnum};
use comfy_table::Table;
use miette::Result;
use std::path::PathBuf;

use verifyos_cli::size_analysis::{analyze_app_size, SizeReport};

#[derive(Debug, Clone, ValueEnum)]
pub enum AnalyzeSizeFormat {
    Table,
    Json,
}

#[derive(Debug, Parser)]
pub struct AnalyzeSizeArgs {
    /// Path to the .ipa or .app bundle to inspect
    #[arg(long)]
    pub app: PathBuf,

    /// Output format
    #[arg(long, value_enum, default_value = "table")]
    pub format: AnalyzeSizeFormat,

    /// Number of top largest files to show
    #[arg(long, default_value_t = 10)]
    pub top: usize,
}

pub fn run(args: AnalyzeSizeArgs) -> Result<()> {
    let report = analyze_app_size(&args.app, args.top)?;
    match args.format {
        AnalyzeSizeFormat::Table => {
            println!("{}", render_table(&report));
        }
        AnalyzeSizeFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&report).map_err(|err| miette::miette!(err))?
            );
        }
    }
    Ok(())
}

fn render_table(report: &SizeReport) -> String {
    let mut out = String::new();
    out.push_str(&format!("App: {}\n", report.app_path));
    out.push_str(&format!("Total size: {} bytes\n\n", report.total_bytes));

    let mut categories = Table::new();
    categories.set_header(vec!["Category", "Bytes", "Files", "%"]);
    for item in &report.categories {
        categories.add_row(vec![
            item.category.clone(),
            item.bytes.to_string(),
            item.file_count.to_string(),
            format!("{:.1}", item.percent_of_total),
        ]);
    }
    out.push_str("Category breakdown\n");
    out.push_str(&categories.to_string());
    out.push_str("\n\n");

    let mut top_files = Table::new();
    top_files.set_header(vec!["Path", "Category", "Bytes"]);
    for item in &report.top_files {
        top_files.add_row(vec![
            item.path.clone(),
            item.category.clone(),
            item.bytes.to_string(),
        ]);
    }
    out.push_str("Top files\n");
    out.push_str(&top_files.to_string());
    out
}
