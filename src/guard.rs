use anyhow::Result;
use colored::*;
use std::path::Path;

const HOOK: &str = r#"#!/bin/sh
# envdiff guard — blocks commits containing secrets in .env files

STAGED=$(git diff --cached --name-only | grep -E '(^|/)\.env(\.[^/]*)?$')

if [ -z "$STAGED" ]; then
    exit 0
fi

echo "envdiff: scanning staged .env files..."

FAILED=0
for f in $STAGED; do
    if ! envdiff audit "$f"; then
        FAILED=1
    fi
done

if [ "$FAILED" -eq 1 ]; then
    echo ""
    echo "Commit blocked: secrets detected. Run \`envdiff mask --in-place\` to redact, then re-stage."
    exit 1
fi
"#;

pub fn run_guard(dir: &str) -> Result<()> {
    let git_dir = Path::new(dir).join(".git");
    anyhow::ensure!(
        git_dir.exists(),
        "No .git directory found in '{}' — is this a git repository?",
        dir
    );

    let hook_path = git_dir.join("hooks").join("pre-commit");

    if hook_path.exists() {
        let existing = std::fs::read_to_string(&hook_path)?;
        if existing.contains("envdiff guard") {
            println!("{} Guard is already installed.", "✓".green().bold());
            return Ok(());
        }
        // Append to existing hook rather than overwriting it
        let updated = format!("{}\n{}", existing.trim_end(), HOOK);
        std::fs::write(&hook_path, updated)?;
        println!(
            "{} Guard appended to existing pre-commit hook at {}.",
            "✓".green().bold(),
            hook_path.display()
        );
    } else {
        std::fs::write(&hook_path, HOOK)?;
        set_executable(&hook_path)?;
        println!(
            "{} Pre-commit hook installed at {}.",
            "✓".green().bold(),
            hook_path.display()
        );
    }

    println!(
        "{}",
        "envdiff will now block commits containing secrets in .env files.".dimmed()
    );
    Ok(())
}

#[cfg(unix)]
fn set_executable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(path)?.permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms)?;
    Ok(())
}

#[cfg(not(unix))]
fn set_executable(_path: &Path) -> Result<()> {
    Ok(())
}
