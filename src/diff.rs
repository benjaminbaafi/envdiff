use crate::parser::EnvFile;
use colored::*;
use std::collections::HashSet;

#[derive(Debug)]
pub struct DiffResult {
    pub only_in_a: Vec<String>,
    pub only_in_b: Vec<String>,
    pub value_differs: Vec<(String, String, String)>, // (key, val_a, val_b)
}

impl DiffResult {
    pub fn has_differences(&self) -> bool {
        !self.only_in_a.is_empty() || !self.only_in_b.is_empty() || !self.value_differs.is_empty()
    }
}

pub fn diff(a: &EnvFile, b: &EnvFile) -> DiffResult {
    let keys_a: HashSet<&String> = a.entries.keys().collect();
    let keys_b: HashSet<&String> = b.entries.keys().collect();

    let mut only_in_a: Vec<String> = keys_a.difference(&keys_b).map(|k| (*k).clone()).collect();
    let mut only_in_b: Vec<String> = keys_b.difference(&keys_a).map(|k| (*k).clone()).collect();
    let mut value_differs = Vec::new();

    for key in keys_a.intersection(&keys_b) {
        let va = &a.entries[*key];
        let vb = &b.entries[*key];
        if va != vb {
            value_differs.push(((*key).clone(), va.clone(), vb.clone()));
        }
    }

    only_in_a.sort();
    only_in_b.sort();
    value_differs.sort_by(|a, b| a.0.cmp(&b.0));

    DiffResult {
        only_in_a,
        only_in_b,
        value_differs,
    }
}

pub fn print_diff(result: &DiffResult, a_path: &str, b_path: &str) {
    if !result.has_differences() {
        println!("{}", "✓ No differences found.".green().bold());
        return;
    }

    println!("{} {}", "---".red(), a_path);
    println!("{} {}", "+++".green(), b_path);
    println!();

    for key in &result.only_in_a {
        println!("{} {} (only in {})", "−".red().bold(), key.red(), a_path);
    }
    for key in &result.only_in_b {
        println!(
            "{} {} (only in {})",
            "+".green().bold(),
            key.green(),
            b_path
        );
    }
    for (key, va, vb) in &result.value_differs {
        println!(
            "{} {} value differs:",
            "~".yellow().bold(),
            key.yellow().bold()
        );
        println!("    {} {}", "−".red(), mask_secret(va).red());
        println!("    {} {}", "+".green(), mask_secret(vb).green());
    }

    println!();
    let total = result.only_in_a.len() + result.only_in_b.len() + result.value_differs.len();
    println!("{}", format!("{} difference(s) found", total).yellow());
}

/// Mask values that look like secrets for safe display
fn mask_secret(val: &str) -> String {
    if val.len() > 8 {
        let visible = &val[..4];
        format!("{}••••••••", visible)
    } else if !val.is_empty() {
        "••••••••".to_string()
    } else {
        "(empty)".to_string()
    }
}
