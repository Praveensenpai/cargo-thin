use clap::Parser;
use std::io;
use std::path::{Path, PathBuf};

use crate::domain::cleaner::clean_project;
use crate::domain::project::{format_bytes, ProjectInfo};
use crate::domain::scanner::scan_projects;

#[derive(Parser, Debug)]
#[command(name = "cargo-thin")]
#[command(bin_name = "cargo thin")]
#[command(author = "Praveen Senpai <pvnt20@gmail.com>")]
#[command(version = "0.1.0")]
#[command(about = "Intelligent Rust target purger that preserves compiled binaries")]
pub struct CliArgs {
    /// Root directory to scan for Rust projects (defaults to current directory)
    #[arg(short, long, default_value = ".")]
    pub path: PathBuf,

    /// Perform a dry-run audit without deleting files
    #[arg(long)]
    pub dry_run: bool,

    /// Output audit or purge results in JSON format
    #[arg(long)]
    pub json: bool,

    /// Output plain text table instead of launching interactive TUI
    #[arg(long)]
    pub plain: bool,

    /// Select all found projects for cleaning
    #[arg(short, long)]
    pub all: bool,

    /// Skip confirmation prompts in headless mode
    #[arg(short, long)]
    pub yes: bool,

    /// Preserve both debug and release binaries (default prefers release)
    #[arg(long)]
    pub keep_both: bool,
}

pub fn run_headless(args: &CliArgs) -> io::Result<()> {
    let canonical = args
        .path
        .canonicalize()
        .unwrap_or_else(|_| args.path.clone());
    let projects = scan_projects(&canonical);

    if args.json {
        let json_str = serde_json::to_string_pretty(&projects).unwrap_or_default();
        println!("{json_str}");
        return Ok(());
    }

    print_audit_table(&canonical, &projects, args.keep_both);

    if args.dry_run || !args.yes {
        if !args.dry_run {
            println!("\nTip: Pass --yes to clean all selected projects non-interactively.");
        }
        return Ok(());
    }

    execute_headless_clean(&projects, args.keep_both);
    Ok(())
}

fn print_audit_table(canonical: &Path, projects: &[ProjectInfo], keep_both: bool) {
    println!("\n=== Cargo Thin (カーゴ・シン) Audit ===");
    println!("Scanning directory: {}\n", canonical.display());

    let total_target: u64 = projects.iter().map(|p| p.total_target_bytes).sum();
    let total_reclaim: u64 = projects
        .iter()
        .map(|p| p.reclaimable_bytes(keep_both))
        .sum();

    println!(
        "{:<24} | {:<12} | {:<12} | {:<6} | {:<30}",
        "PROJECT", "TARGET SIZE", "RECLAIMABLE", "SAVINGS", "PRESERVED BINARIES"
    );
    println!("{}", "-".repeat(95));

    for p in projects {
        let reclaim = format_bytes(p.reclaimable_bytes(keep_both));
        let pct = format!("{:.0}%", p.reclaimable_percent(keep_both));
        let bin_desc = if p.binaries.is_empty() {
            "None".to_string()
        } else {
            p.binaries
                .iter()
                .map(|b| format!("{}/{}", b.profile, b.filename))
                .collect::<Vec<_>>()
                .join(", ")
        };

        println!(
            "{:<24} | {:<12} | {:<12} | {:<6} | {:<30}",
            p.package_name,
            format_bytes(p.total_target_bytes),
            reclaim,
            pct,
            bin_desc
        );
    }
    println!("{}", "-".repeat(95));
    println!("Total Target Bloat:   {}", format_bytes(total_target));
    println!("Total Reclaimable:    {}", format_bytes(total_reclaim));
}

fn execute_headless_clean(projects: &[ProjectInfo], keep_both: bool) {
    println!("\nPurging intermediate bloat and preserving binaries...");
    let mut total_freed = 0u64;
    for p in projects {
        match clean_project(p, keep_both) {
            Ok(report) => {
                total_freed += report.bytes_freed;
                println!(
                    "  ✔ {} freed {} (kept {} binaries)",
                    report.package_name,
                    format_bytes(report.bytes_freed),
                    report.binaries_preserved
                );
            }
            Err(e) => {
                eprintln!("  ✖ Failed cleaning {}: {e}", p.package_name);
            }
        }
    }
    println!(
        "\nDone! Total disk space reclaimed: {}\n",
        format_bytes(total_freed)
    );
}
