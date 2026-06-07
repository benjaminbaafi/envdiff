use crate::parser::EnvFile;
use anyhow::Result;
use colored::*;
use regex::Regex;

struct Pattern {
    name: &'static str,
    regex: &'static str,
    severity: Severity,
}

#[derive(Clone, Copy)]
enum Severity {
    Critical,
    High,
    Medium,
}

static PATTERNS: &[Pattern] = &[
    Pattern { name: "AWS Access Key ID",           regex: r"AKIA[0-9A-Z]{16}",                                          severity: Severity::Critical },
    Pattern { name: "GitHub Personal Access Token",regex: r"ghp_[A-Za-z0-9]{36}",                                       severity: Severity::Critical },
    Pattern { name: "GitHub Fine-grained Token",   regex: r"github_pat_[A-Za-z0-9_]{82}",                               severity: Severity::Critical },
    Pattern { name: "GitHub OAuth Token",          regex: r"gho_[A-Za-z0-9]{36}",                                       severity: Severity::Critical },
    Pattern { name: "Stripe Live Secret Key",      regex: r"sk_live_[0-9a-zA-Z]{24,}",                                  severity: Severity::Critical },
    Pattern { name: "Stripe Live Publishable Key", regex: r"pk_live_[0-9a-zA-Z]{24,}",                                  severity: Severity::High     },
    Pattern { name: "Slack Token",                 regex: r"xox[baprs]-[0-9A-Za-z-]+",                                  severity: Severity::Critical },
    Pattern { name: "SendGrid API Key",            regex: r"SG\.[a-zA-Z0-9_-]{22}\.[a-zA-Z0-9_-]{43}",                 severity: Severity::Critical },
    Pattern { name: "Google API Key",              regex: r"AIza[0-9A-Za-z\-_]{35}",                                    severity: Severity::Critical },
    Pattern { name: "NPM Token",                   regex: r"npm_[A-Za-z0-9]{36}",                                       severity: Severity::Critical },
    Pattern { name: "Twilio Auth Token",           regex: r"SK[0-9a-fA-F]{32}",                                         severity: Severity::Critical },
    Pattern { name: "Twilio Account SID",          regex: r"AC[a-fA-F0-9]{32}",                                         severity: Severity::High     },
    Pattern { name: "JWT",                         regex: r"eyJ[A-Za-z0-9-_]+\.eyJ[A-Za-z0-9-_]+\.[A-Za-z0-9-_.+/=]+", severity: Severity::High     },
    Pattern { name: "PEM Private Key",             regex: r"-----BEGIN [A-Z ]*PRIVATE KEY-----",                        severity: Severity::Critical },
    Pattern { name: "Firebase URL",                regex: r"https://[a-z0-9-]+\.firebaseio\.com",                       severity: Severity::Medium   },
];

struct Finding {
    key: String,
    pattern_name: &'static str,
    severity: Severity,
}

pub fn run_audit(path: &str) -> Result<()> {
    let file = EnvFile::load(path)?;

    let compiled: Vec<(&Pattern, Regex)> = PATTERNS
        .iter()
        .filter_map(|p| Regex::new(p.regex).ok().map(|r| (p, r)))
        .collect();

    let mut findings: Vec<Finding> = Vec::new();

    for (key, val) in &file.entries {
        for (pattern, regex) in &compiled {
            if regex.is_match(val) {
                findings.push(Finding {
                    key: key.clone(),
                    pattern_name: pattern.name,
                    severity: pattern.severity,
                });
            }
        }
    }

    if findings.is_empty() {
        println!("{} No secrets detected in {}.", "✓".green().bold(), path);
        return Ok(());
    }

    println!("{} {} secret(s) detected in {}:\n", "✗".red().bold(), findings.len(), path);
    for f in &findings {
        let sev = match f.severity {
            Severity::Critical => "critical".red().bold(),
            Severity::High     => "high    ".yellow().bold(),
            Severity::Medium   => "medium  ".cyan().bold(),
        };
        println!("  [{}] {} — {}", sev, f.key.bold(), f.pattern_name);
    }

    println!();
    println!("{}", "Run `envdiff mask` to redact values for safe sharing.".dimmed());

    let has_critical = findings
        .iter()
        .any(|f| matches!(f.severity, Severity::Critical));
    if has_critical {
        std::process::exit(1);
    }

    Ok(())
}
