use clap::{Parser, Subcommand};
use colored::*;
use sqruff_core::config::Config;
use sqruff_core::rules::Severity;
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(
    name = "sqruff",
    about = "An extremely fast SQL linter and formatter, written in Rust.",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// SQL dialect (generic, mysql, postgresql, tidb, sqlite, duckdb)
    #[arg(long, global = true)]
    dialect: Option<String>,

    /// Config file path
    #[arg(long, global = true)]
    config: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Lint SQL files for issues
    Check {
        /// SQL files to check
        files: Vec<PathBuf>,

        /// Automatically fix violations where possible
        #[arg(long)]
        fix: bool,
    },
    /// Format SQL files
    Format {
        /// SQL files to format
        files: Vec<PathBuf>,

        /// Check formatting without writing (exit 1 if changes needed)
        #[arg(long)]
        check: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    let mut config = Config::load(cli.config.as_deref());
    if let Some(dialect) = &cli.dialect {
        config.dialect = dialect.clone();
    }

    let exit_code = match cli.command {
        Commands::Check { files, fix: _ } => run_check(&files, &config),
        Commands::Format { files, check } => run_format(&files, &config, check),
    };

    process::exit(exit_code);
}

fn run_check(files: &[PathBuf], config: &Config) -> i32 {
    if files.is_empty() {
        eprintln!("{}: no files specified", "error".red().bold());
        return 2;
    }

    let mut total_violations = 0;
    let mut file_count = 0;

    for path in files {
        let source = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("{}: {}: {}", "error".red().bold(), path.display(), e);
                continue;
            }
        };

        file_count += 1;
        let mut violations = match sqruff_core::lint(&source, config) {
            Ok(v) => v,
            Err(e) => {
                eprintln!(
                    "{}: {}: parse error: {}",
                    "error".red().bold(),
                    path.display(),
                    e
                );
                continue;
            }
        };

        // Also run source-level trailing comma check
        let trailing = sqruff_core::rules::trailing_comma::check_trailing_comma_in_source(&source);
        violations.extend(trailing);
        violations.sort_by(|a, b| a.line.cmp(&b.line).then(a.col.cmp(&b.col)));

        for v in &violations {
            let severity_str = match v.severity {
                Severity::Error => format!("{}", "error".red().bold()),
                Severity::Warning => format!("{}", "warning".yellow().bold()),
            };
            println!(
                "{}:{}:{}: {} {} {}",
                path.display().to_string().cyan(),
                v.line.to_string().white(),
                v.col.to_string().white(),
                severity_str,
                v.rule_code.dimmed(),
                v.message,
            );
        }

        total_violations += violations.len();
    }

    if total_violations > 0 {
        eprintln!(
            "\n{} {} in {} {}",
            "Found".bold(),
            format!(
                "{} {}",
                total_violations,
                if total_violations == 1 {
                    "violation"
                } else {
                    "violations"
                }
            )
            .red()
            .bold(),
            file_count,
            if file_count == 1 { "file" } else { "files" }
        );
        1
    } else {
        eprintln!(
            "{} {} {} checked",
            "All good!".green().bold(),
            file_count,
            if file_count == 1 { "file" } else { "files" }
        );
        0
    }
}

fn run_format(files: &[PathBuf], config: &Config, check_only: bool) -> i32 {
    if files.is_empty() {
        eprintln!("{}: no files specified", "error".red().bold());
        return 2;
    }

    let mut needs_formatting = 0;

    for path in files {
        let source = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("{}: {}: {}", "error".red().bold(), path.display(), e);
                continue;
            }
        };

        let formatted = match sqruff_core::format(&source, config) {
            Ok(f) => f,
            Err(e) => {
                eprintln!(
                    "{}: {}: parse error: {}",
                    "error".red().bold(),
                    path.display(),
                    e
                );
                continue;
            }
        };

        if source != formatted {
            if check_only {
                println!(
                    "{} {}",
                    "Would reformat:".yellow().bold(),
                    path.display()
                );
                needs_formatting += 1;
            } else {
                std::fs::write(path, &formatted).unwrap_or_else(|e| {
                    eprintln!("{}: {}: {}", "error".red().bold(), path.display(), e);
                });
                println!("{} {}", "Formatted:".green().bold(), path.display());
            }
        }
    }

    if check_only && needs_formatting > 0 {
        eprintln!(
            "\n{} {} {} need formatting",
            "Error:".red().bold(),
            needs_formatting,
            if needs_formatting == 1 { "file" } else { "files" }
        );
        1
    } else {
        0
    }
}
