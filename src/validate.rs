use crate::parser::EnvFile;
use anyhow::Result;
use colored::*;

pub fn run_validate(path: &str) -> Result<()> {
    let file = EnvFile::load(path)?;
    let mut issues: Vec<(String, String)> = Vec::new();

    for key in file.keys() {
        if let Some(reason) = validate_key(key) {
            issues.push((key.clone(), reason));
        }
    }

    if issues.is_empty() {
        println!("{} All keys in {} are POSIX-compliant.", "✓".green().bold(), path);
        return Ok(());
    }

    println!("{} {} invalid key(s) in {}:\n", "✗".red().bold(), issues.len(), path);
    for (key, reason) in &issues {
        println!("  {} {}: {}", "✗".red(), key.red().bold(), reason);
    }
    println!();

    std::process::exit(1);
}

fn validate_key(key: &str) -> Option<String> {
    if key.is_empty() {
        return Some("empty key name".into());
    }
    if key.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
        return Some("starts with a digit — invalid in POSIX and most shells".into());
    }
    if key.contains('-') {
        return Some("contains a hyphen — invalid in POSIX, crashes systemd and Kubernetes".into());
    }
    if key.contains(' ') {
        return Some("contains a space — breaks most env parsers".into());
    }
    let bad: Vec<char> = key
        .chars()
        .filter(|c| !c.is_ascii_alphanumeric() && *c != '_')
        .collect();
    if !bad.is_empty() {
        return Some(format!(
            "contains invalid character(s): {}",
            bad.iter().map(|c| format!("'{c}'")).collect::<Vec<_>>().join(", ")
        ));
    }
    None
}
