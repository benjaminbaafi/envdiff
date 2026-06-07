use crate::diff::{diff, print_diff};
use crate::parser::EnvFile;
use anyhow::Result;
use colored::*;
use std::collections::HashSet;

pub fn run_check(dir: &str) -> Result<()> {
    let env_files = find_env_files(dir);

    if env_files.is_empty() {
        println!("{}", "No .env files found.".yellow());
        return Ok(());
    }

    println!(
        "{} Found {} env file(s):",
        "→".cyan().bold(),
        env_files.len()
    );
    for f in &env_files {
        println!("  {}", f.dimmed());
    }
    println!();

    // Load all files
    let mut loaded = Vec::new();
    for path in &env_files {
        match EnvFile::load(path) {
            Ok(f) => loaded.push(f),
            Err(e) => eprintln!("{} Could not load {}: {}", "✗".red(), path, e),
        }
    }

    if loaded.len() < 2 {
        println!("{}", "Need at least 2 env files to compare.".yellow());
        return Ok(());
    }

    // Report all keys across all files
    let all_keys: HashSet<String> = loaded
        .iter()
        .flat_map(|f| f.entries.keys().cloned())
        .collect();

    let mut issues = 0;

    println!("{}", "Key coverage:".bold());
    let mut sorted_keys: Vec<&String> = all_keys.iter().collect();
    sorted_keys.sort();

    for key in &sorted_keys {
        let missing: Vec<&str> = loaded
            .iter()
            .filter(|f| !f.entries.contains_key(*key))
            .map(|f| f.path.as_str())
            .collect();

        if missing.is_empty() {
            println!("  {} {}", "✓".green(), key);
        } else {
            println!(
                "  {} {} — missing in: {}",
                "✗".red(),
                key.yellow(),
                missing.join(", ").red()
            );
            issues += 1;
        }
    }

    println!();

    // Compare adjacent pairs
    for i in 0..loaded.len() - 1 {
        let result = diff(&loaded[i], &loaded[i + 1]);
        if result.has_differences() {
            print_diff(&result, &loaded[i].path, &loaded[i + 1].path);
        }
    }

    if issues == 0 {
        println!(
            "{}",
            "✓ All keys present across all env files.".green().bold()
        );
    } else {
        println!(
            "{}",
            format!("✗ {} missing key(s) detected.", issues)
                .red()
                .bold()
        );
        std::process::exit(1);
    }

    Ok(())
}

pub fn find_env_files(dir: &str) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return vec![];
    };

    let mut files: Vec<String> = entries
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            // match .env, .env.local, .env.staging, .env.production etc
            // but exclude .env.example (template, not real)
            if name.starts_with(".env") && !name.ends_with(".example") {
                Some(format!("{}/{}", dir, name))
            } else {
                None
            }
        })
        .collect();

    files.sort();
    files
}
