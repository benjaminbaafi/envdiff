use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn cmd() -> Command {
    Command::cargo_bin("envdiff").unwrap()
}

/// Returns a Command with the Pro gate unlocked for testing.
fn pro_cmd() -> Command {
    let mut c = Command::cargo_bin("envdiff").unwrap();
    c.env("ENVDIFF_PRO_KEY", "1");
    c
}

fn write(dir: &TempDir, name: &str, content: &str) -> String {
    let path = dir.path().join(name);
    fs::write(&path, content).unwrap();
    path.to_string_lossy().into_owned()
}

// ── check ────────────────────────────────────────────────────────────────────

#[test]
fn check_all_keys_match() {
    let dir = TempDir::new().unwrap();
    write(&dir, ".env", "FOO=bar\nBAZ=123\n");
    write(&dir, ".env.local", "FOO=other\nBAZ=456\n");

    cmd()
        .args(["check", dir.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("✓"));
}

#[test]
fn check_missing_keys_exits_nonzero() {
    let dir = TempDir::new().unwrap();
    write(&dir, ".env", "FOO=bar\nSECRET=x\n");
    write(&dir, ".env.local", "FOO=other\n");

    cmd()
        .args(["check", dir.path().to_str().unwrap()])
        .assert()
        .failure()
        .stdout(predicate::str::contains("SECRET"));
}

#[test]
fn check_no_env_files() {
    let dir = TempDir::new().unwrap();

    cmd()
        .args(["check", dir.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("No .env files found"));
}

// ── diff ─────────────────────────────────────────────────────────────────────

#[test]
fn diff_identical_files() {
    let dir = TempDir::new().unwrap();
    let a = write(&dir, "a.env", "FOO=bar\nBAZ=123\n");
    let b = write(&dir, "b.env", "FOO=bar\nBAZ=123\n");

    cmd()
        .args(["diff", &a, &b])
        .assert()
        .success()
        .stdout(predicate::str::contains("No differences found"));
}

#[test]
fn diff_shows_missing_and_changed_keys() {
    let dir = TempDir::new().unwrap();
    let a = write(&dir, "a.env", "FOO=bar\nONLY_A=x\n");
    let b = write(&dir, "b.env", "FOO=different\nONLY_B=y\n");

    cmd()
        .args(["diff", &a, &b])
        .assert()
        .success()
        .stdout(predicate::str::contains("ONLY_A"))
        .stdout(predicate::str::contains("ONLY_B"))
        .stdout(predicate::str::contains("FOO"));
}

// ── lint ─────────────────────────────────────────────────────────────────────

#[test]
fn lint_clean_file() {
    let dir = TempDir::new().unwrap();
    let p = write(&dir, ".env", "FOO=bar\nBAZ=123\n");

    cmd()
        .args(["lint", &p])
        .assert()
        .success()
        .stdout(predicate::str::contains("looks clean"));
}

#[test]
fn lint_catches_hyphen_key() {
    let dir = TempDir::new().unwrap();
    let p = write(&dir, ".env", "bad-key=value\n");

    cmd()
        .args(["lint", &p])
        .assert()
        .failure()
        .stdout(predicate::str::contains("hyphen"));
}

#[test]
fn lint_catches_duplicate_key() {
    let dir = TempDir::new().unwrap();
    let p = write(&dir, ".env", "FOO=first\nFOO=second\n");

    cmd()
        .args(["lint", &p])
        .assert()
        .success() // duplicate is a warning, not an error
        .stdout(predicate::str::contains("Duplicate"));
}

#[test]
fn lint_catches_digit_leading_key() {
    let dir = TempDir::new().unwrap();
    let p = write(&dir, ".env", "1BAD=value\n");

    cmd()
        .args(["lint", &p])
        .assert()
        .failure()
        .stdout(predicate::str::contains("digit"));
}

// ── missing ──────────────────────────────────────────────────────────────────

#[test]
fn missing_all_present() {
    let dir = TempDir::new().unwrap();
    let env = write(&dir, ".env", "FOO=bar\nBAZ=123\n");
    let example = write(&dir, ".env.example", "FOO=\nBAZ=\n");

    cmd()
        .args(["missing", &env, &example])
        .assert()
        .success()
        .stdout(predicate::str::contains("accounted for"));
}

#[test]
fn missing_reports_absent_keys() {
    let dir = TempDir::new().unwrap();
    let env = write(&dir, ".env", "FOO=bar\n");
    let example = write(&dir, ".env.example", "FOO=\nREQUIRED=\n");

    cmd()
        .args(["missing", &env, &example])
        .assert()
        .failure()
        .stdout(predicate::str::contains("REQUIRED"));
}

// ── sync ─────────────────────────────────────────────────────────────────────

#[test]
fn sync_dry_run_does_not_write() {
    let dir = TempDir::new().unwrap();
    let env = write(&dir, ".env", "FOO=bar\nNEW_KEY=secret\n");
    let example = write(&dir, ".env.example", "FOO=\n");
    let before = fs::read_to_string(&example).unwrap();

    cmd()
        .args(["sync", &env, &example, "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("dry run"))
        .stdout(predicate::str::contains("NEW_KEY"));

    assert_eq!(fs::read_to_string(&example).unwrap(), before);
}

#[test]
fn sync_adds_new_keys_without_values() {
    let dir = TempDir::new().unwrap();
    let env = write(&dir, ".env", "FOO=bar\nNEW_KEY=supersecret\n");
    let example = write(&dir, ".env.example", "FOO=\n");

    cmd()
        .args(["sync", &env, &example])
        .assert()
        .success();

    let result = fs::read_to_string(&example).unwrap();
    assert!(result.contains("NEW_KEY="));
    assert!(!result.contains("supersecret"));
}

// ── validate ─────────────────────────────────────────────────────────────────

#[test]
fn validate_valid_keys() {
    let dir = TempDir::new().unwrap();
    let p = write(&dir, ".env", "FOO=bar\nBAR_BAZ=123\n");

    cmd()
        .args(["validate", &p])
        .assert()
        .success()
        .stdout(predicate::str::contains("POSIX-compliant"));
}

#[test]
fn validate_rejects_hyphen() {
    let dir = TempDir::new().unwrap();
    let p = write(&dir, ".env", "bad-key=value\n");

    cmd()
        .args(["validate", &p])
        .assert()
        .failure()
        .stdout(predicate::str::contains("hyphen"));
}

#[test]
fn validate_rejects_digit_prefix() {
    let dir = TempDir::new().unwrap();
    let p = write(&dir, ".env", "1BAD=value\n");

    cmd()
        .args(["validate", &p])
        .assert()
        .failure()
        .stdout(predicate::str::contains("digit"));
}

// ── audit ─────────────────────────────────────────────────────────────────────

#[test]
fn audit_clean_file() {
    let dir = TempDir::new().unwrap();
    let p = write(&dir, ".env", "FOO=bar\nPORT=3000\n");

    pro_cmd()
        .args(["audit", &p])
        .assert()
        .success()
        .stdout(predicate::str::contains("No secrets detected"));
}

#[test]
fn audit_catches_stripe_key() {
    let dir = TempDir::new().unwrap();
    // Split literal so the full pattern doesn't appear in source (GitHub push protection)
    let content = format!("STRIPE_KEY={}FakeKeyForTestingOnlyNotReal12\n", "sk_live_");
    let p = write(&dir, ".env", &content);

    pro_cmd()
        .args(["audit", &p])
        .assert()
        .failure()
        .stdout(predicate::str::contains("Stripe Live Secret Key"));
}

#[test]
fn audit_catches_aws_key() {
    let dir = TempDir::new().unwrap();
    // Split literal so the full pattern doesn't appear in source (GitHub push protection)
    let content = format!("AWS_ACCESS_KEY={}FAKEACCESSKEY0123456\n", "AKIA");
    let p = write(&dir, ".env", &content);

    pro_cmd()
        .args(["audit", &p])
        .assert()
        .failure()
        .stdout(predicate::str::contains("AWS Access Key ID"));
}

#[test]
fn audit_catches_github_token() {
    let dir = TempDir::new().unwrap();
    // Split literal so the full pattern doesn't appear in source (GitHub push protection)
    let content = format!("GH_TOKEN={}FakeGithubTokenForTestingNotReal1234\n", "ghp_");
    let p = write(&dir, ".env", &content);

    pro_cmd()
        .args(["audit", &p])
        .assert()
        .failure()
        .stdout(predicate::str::contains("GitHub"));
}

// ── mask ──────────────────────────────────────────────────────────────────────

#[test]
fn mask_redacts_values_to_stdout() {
    let dir = TempDir::new().unwrap();
    let p = write(&dir, ".env", "FOO=supersecret\nBAR=anothersecret\n");

    cmd()
        .args(["mask", &p])
        .assert()
        .success()
        .stdout(predicate::str::contains("FOO=****"))
        .stdout(predicate::str::contains("BAR=****"))
        .stdout(predicate::str::contains("supersecret").not())
        .stdout(predicate::str::contains("anothersecret").not());
}

#[test]
fn mask_in_place_writes_file() {
    let dir = TempDir::new().unwrap();
    let p = write(&dir, ".env", "FOO=supersecret\n");

    cmd()
        .args(["mask", &p, "--in-place"])
        .assert()
        .success();

    let result = fs::read_to_string(&p).unwrap();
    assert!(result.contains("FOO=****"));
    assert!(!result.contains("supersecret"));
}

// ── guard ─────────────────────────────────────────────────────────────────────

#[test]
fn guard_fails_outside_git_repo() {
    let dir = TempDir::new().unwrap();

    pro_cmd()
        .args(["guard", dir.path().to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("git repository").or(predicate::str::contains(".git")));
}

#[test]
fn guard_installs_hook_in_git_repo() {
    let dir = TempDir::new().unwrap();
    fs::create_dir_all(dir.path().join(".git/hooks")).unwrap();

    pro_cmd()
        .args(["guard", dir.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("installed"));

    let hook = dir.path().join(".git/hooks/pre-commit");
    assert!(hook.exists());
    let content = fs::read_to_string(hook).unwrap();
    assert!(content.contains("envdiff"));
}
