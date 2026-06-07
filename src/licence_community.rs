use anyhow::Result;
use colored::*;

/// Community build — pro commands require the official binary from https://envdiff.dev
/// The real licence.rs is not open sourced.

pub fn require_pro(command: &str) {
    if !is_pro() {
        eprintln!(
            "{} {} requires envdiff Pro.",
            "✗".red().bold(),
            command.yellow().bold()
        );
        eprintln!(
            "  Get your licence at {} or run {} to activate.",
            "https://envdiff.dev".cyan().underline(),
            "envdiff activate <key>".dimmed()
        );
        std::process::exit(2);
    }
}

pub fn is_pro() -> bool {
    // ENVDIFF_PRO_KEY bypasses the gate for CI and integration tests.
    std::env::var("ENVDIFF_PRO_KEY").is_ok()
}

pub fn run_activate(_key: &str) -> Result<()> {
    anyhow::bail!(
        "Key activation requires the official Pro binary.\nDownload it at https://envdiff.dev"
    )
}

pub fn validate_key(_key: &str) -> bool {
    false
}
