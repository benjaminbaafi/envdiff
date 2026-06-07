use anyhow::Result;
use colored::*;

#[derive(Debug)]
pub struct LintIssue {
    pub line: usize,
    pub key: Option<String>,
    pub severity: Severity,
    pub message: String,
}

#[derive(Debug)]
pub enum Severity {
    Error,
    Warning,
}

pub fn run_lint(path: &str) -> Result<()> {
    let content =
        std::fs::read_to_string(path).map_err(|_| anyhow::anyhow!("Cannot read file: {}", path))?;

    let mut issues: Vec<LintIssue> = Vec::new();
    let mut seen_keys: std::collections::HashMap<String, usize> = Default::default();

    for (i, line) in content.lines().enumerate() {
        let lineno = i + 1;
        let trimmed = line.trim();

        // blank or comment — skip
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Check for BOM character
        if line.contains('\u{FEFF}') {
            issues.push(LintIssue {
                line: lineno,
                key: None,
                severity: Severity::Error,
                message: "BOM character detected — will cause silent parse failures".into(),
            });
        }

        // Must have an = sign
        let Some(eq_pos) = trimmed.find('=') else {
            issues.push(LintIssue {
                line: lineno,
                key: None,
                severity: Severity::Error,
                message: format!("No '=' found: {:?}", trimmed),
            });
            continue;
        };

        let key = trimmed[..eq_pos].trim();
        let val = &trimmed[eq_pos + 1..];

        // Trailing whitespace in key
        if key != key.trim() {
            issues.push(LintIssue {
                line: lineno,
                key: Some(key.into()),
                severity: Severity::Warning,
                message: "Key has leading/trailing whitespace".into(),
            });
        }

        // Non-POSIX key name (hyphens crash systemd/k8s)
        if key.contains('-') {
            issues.push(LintIssue {
                line: lineno,
                key: Some(key.into()),
                severity: Severity::Error,
                message: format!(
                    "Key '{}' contains a hyphen — invalid in POSIX, crashes systemd/k8s",
                    key
                ),
            });
        }

        // Key starts with a digit
        if key
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
        {
            issues.push(LintIssue {
                line: lineno,
                key: Some(key.into()),
                severity: Severity::Error,
                message: format!(
                    "Key '{}' starts with a digit — invalid in most environments",
                    key
                ),
            });
        }

        // Duplicate key
        if let Some(prev_line) = seen_keys.get(key) {
            issues.push(LintIssue {
                line: lineno,
                key: Some(key.into()),
                severity: Severity::Warning,
                message: format!("Duplicate key — also defined on line {}", prev_line),
            });
        } else {
            seen_keys.insert(key.to_string(), lineno);
        }

        // Mismatched quotes
        let has_open_double = val.starts_with('"') && !val.ends_with('"');
        let has_open_single = val.starts_with('\'') && !val.ends_with('\'');
        if has_open_double || has_open_single {
            issues.push(LintIssue {
                line: lineno,
                key: Some(key.into()),
                severity: Severity::Error,
                message: "Mismatched quotes in value".into(),
            });
        }

        // Trailing whitespace after value
        let raw_val = &trimmed[eq_pos + 1..];
        if raw_val != raw_val.trim_end() {
            issues.push(LintIssue {
                line: lineno,
                key: Some(key.into()),
                severity: Severity::Warning,
                message: "Value has trailing whitespace".into(),
            });
        }
    }

    if issues.is_empty() {
        println!("{} {} looks clean!", "✓".green().bold(), path);
        return Ok(());
    }

    println!(
        "{} {} issue(s) in {}:\n",
        issues.len(),
        "lint".yellow().bold(),
        path
    );
    for issue in &issues {
        let sev = match issue.severity {
            Severity::Error => "error".red().bold(),
            Severity::Warning => "warn ".yellow().bold(),
        };
        let loc = format!("line {}", issue.line).dimmed();
        let key_hint = issue.key.as_deref().map(|k| format!(" [{k}]")).unwrap_or_default();
        println!("  [{sev}] {loc}{key_hint}: {}", issue.message);
    }

    let errors = issues
        .iter()
        .filter(|i| matches!(i.severity, Severity::Error))
        .count();
    if errors > 0 {
        std::process::exit(1);
    }

    Ok(())
}
