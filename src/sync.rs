use crate::parser::EnvFile;
use anyhow::Result;
use colored::*;
use std::collections::HashSet;

pub fn run_sync(env_path: &str, example_path: &str, dry_run: bool) -> Result<()> {
    let env = EnvFile::load(env_path)?;

    // Load or start fresh for .env.example
    let example = EnvFile::load(example_path).ok();
    let existing_keys: HashSet<String> = example
        .as_ref()
        .map(|e| e.entries.keys().cloned().collect())
        .unwrap_or_default();

    let mut added = Vec::new();
    let mut removed = Vec::new();

    // Find keys to add
    for key in env.keys() {
        if !existing_keys.contains(key) {
            added.push(key.clone());
        }
    }

    // Find keys to remove (in example but not in .env)
    let env_keys: HashSet<String> = env.entries.keys().cloned().collect();
    for key in &existing_keys {
        if !env_keys.contains(key) {
            removed.push(key.clone());
        }
    }

    if added.is_empty() && removed.is_empty() {
        println!("{} {} is already in sync.", "✓".green().bold(), example_path);
        return Ok(());
    }

    if !added.is_empty() {
        println!("{} Keys to add to {}:", "+".green().bold(), example_path);
        for key in &added {
            println!("  {} {}", "+".green(), key.green());
        }
    }
    if !removed.is_empty() {
        println!("{} Keys to remove from {}:", "−".red().bold(), example_path);
        for key in &removed {
            println!("  {} {}", "−".red(), key.red());
        }
    }

    if dry_run {
        println!("\n{}", "(dry run — no files changed)".dimmed());
        return Ok(());
    }

    // Rebuild .env.example: keep existing lines, append new keys with empty values
    let mut lines: Vec<String> = Vec::new();

    if let Some(ref ex) = example {
        for line in &ex.raw_lines {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                lines.push(line.clone());
                continue;
            }
            if let Some(eq_pos) = trimmed.find('=') {
                let key = trimmed[..eq_pos].trim();
                if !removed.contains(&key.to_string()) {
                    lines.push(format!("{}=", key));
                }
            }
        }
    }

    // Append new keys
    if !added.is_empty() {
        if !lines.is_empty() {
            lines.push(String::new()); // blank separator
        }
        lines.push("# Added by envdiff sync".into());
        for key in &added {
            lines.push(format!("{}=", key));
        }
    }

    std::fs::write(example_path, lines.join("\n") + "\n")?;
    println!("\n{} {} updated.", "✓".green().bold(), example_path);

    Ok(())
}