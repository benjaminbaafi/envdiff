use crate::parser::EnvFile;
use anyhow::Result;
use colored::*;

pub fn run_missing(env_path: &str, example_path: &str) -> Result<()> {
    let env = EnvFile::load(env_path)?;
    let example = EnvFile::load(example_path)?;

    let mut missing = Vec::new();
    let mut present = Vec::new();

    for key in example.keys() {
        if env.entries.contains_key(key) {
            present.push(key.clone());
        } else {
            missing.push(key.clone());
        }
    }

    // Also report keys in .env that aren't in .env.example
    let mut undocumented = Vec::new();
    for key in env.keys() {
        if !example.entries.contains_key(key) {
            undocumented.push(key.clone());
        }
    }

    if missing.is_empty() && undocumented.is_empty() {
        println!("{} All keys accounted for.", "✓".green().bold());
        return Ok(());
    }

    if !missing.is_empty() {
        println!(
            "{} Keys required by {} but missing from {}:\n",
            "✗".red().bold(),
            example_path,
            env_path
        );
        for key in &missing {
            println!("  {} {}", "−".red(), key.red().bold());
        }
        println!();
    }

    if !undocumented.is_empty() {
        println!(
            "{} Keys in {} not documented in {}:\n",
            "!".yellow().bold(),
            env_path,
            example_path
        );
        for key in &undocumented {
            println!("  {} {}", "?".yellow(), key.yellow());
        }
        println!();
    }

    if !missing.is_empty() {
        eprintln!(
            "{}",
            format!(
                "{} missing key(s) — run `envdiff sync` to update .env.example",
                missing.len()
            )
            .red()
        );
        std::process::exit(1);
    }

    Ok(())
}
