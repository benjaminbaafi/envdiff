mod audit;
mod check;
mod diff;
mod guard;
mod licence;
mod lint;
mod mask;
mod missing;
mod parser;
mod sync;
mod validate;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;

#[derive(Parser)]
#[command(
    name = "envdiff",
    about = "A fast, local .env auditor — diff, validate, and guard your env files",
    version = "0.1.0",
    after_help = "Tip: run `envdiff check` first to get an overview of your project."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan all .env files in the project and report key mismatches
    Check {
        #[arg(default_value = ".")]
        dir: String,
    },
    /// Show a colored diff between two .env files
    Diff { file_a: String, file_b: String },
    /// Lint a .env file for formatting issues and bad key names
    Lint {
        #[arg(default_value = ".env")]
        path: String,
    },
    /// Show keys in .env.example missing from .env
    Missing {
        #[arg(default_value = ".env")]
        env: String,
        #[arg(default_value = ".env.example")]
        example: String,
    },
    /// Sync keys from .env into .env.example (never values)
    Sync {
        #[arg(default_value = ".env")]
        env: String,
        #[arg(default_value = ".env.example")]
        example: String,
        #[arg(long)]
        dry_run: bool,
    },
    /// Scan a .env file for exposed secrets (API keys, tokens, credentials)
    Audit {
        #[arg(default_value = ".env")]
        path: String,
    },
    /// Install a git pre-commit hook that blocks commits containing secrets
    Guard {
        #[arg(default_value = ".")]
        dir: String,
    },
    /// Redact all values in a .env file for safe sharing
    Mask {
        #[arg(default_value = ".env")]
        path: String,
        /// Write masked output back to the file instead of stdout
        #[arg(long)]
        in_place: bool,
    },
    /// Validate key names for POSIX/systemd/Kubernetes compliance
    Validate {
        #[arg(default_value = ".env")]
        path: String,
    },
    /// Activate envdiff Pro with a licence key
    Activate {
        key: String,
    },
}

fn main() -> Result<()> {
    println!("{} {}", "envdiff".cyan().bold(), "v0.1.0".dimmed());
    println!("{}", "─".repeat(40).dimmed());

    let cli = Cli::parse();

    match cli.command {
        Commands::Check { dir } => check::run_check(&dir)?,
        Commands::Diff { file_a, file_b } => {
            let a = parser::EnvFile::load(&file_a)?;
            let b = parser::EnvFile::load(&file_b)?;
            let result = diff::diff(&a, &b);
            diff::print_diff(&result, &file_a, &file_b);
        }
        Commands::Lint { path } => lint::run_lint(&path)?,
        Commands::Missing { env, example } => missing::run_missing(&env, &example)?,
        Commands::Sync {
            env,
            example,
            dry_run,
        } => sync::run_sync(&env, &example, dry_run)?,
        Commands::Audit { path } => {
            licence::require_pro("audit");
            audit::run_audit(&path)?;
        }
        Commands::Guard { dir } => {
            licence::require_pro("guard");
            guard::run_guard(&dir)?;
        }
        Commands::Mask { path, in_place } => mask::run_mask(&path, in_place)?,
        Commands::Validate { path } => validate::run_validate(&path)?,
        Commands::Activate { key } => licence::run_activate(&key)?,
    }

    Ok(())
}
