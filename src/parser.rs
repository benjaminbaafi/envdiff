use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct EnvFile {
    pub path: String,
    pub entries: HashMap<String, String>,
    pub ordered_keys: Vec<String>,
    pub raw_lines: Vec<String>,
}

impl EnvFile {
    pub fn load(path: &str) -> Result<Self> {
        let p = Path::new(path);
        anyhow::ensure!(p.exists(), "File not found: {}", path);

        let content =
            std::fs::read_to_string(p).with_context(|| format!("Could not read {}", path))?;

        let raw_lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
        let mut entries = HashMap::new();
        let mut ordered_keys = Vec::new();

        let mut i = 0;
        while i < raw_lines.len() {
            let line = &raw_lines[i];
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                i += 1;
                continue;
            }

            // strip optional `export ` prefix
            let trimmed = trimmed
                .strip_prefix("export ")
                .map(|s| s.trim())
                .unwrap_or(trimmed);

            if let Some(eq_pos) = trimmed.find('=') {
                let key = trimmed[..eq_pos].trim().to_string();
                let raw_val = trimmed[eq_pos + 1..].trim();

                let (val, extra_lines) = parse_value(raw_val, &raw_lines[i + 1..]);

                if !key.is_empty() && !ordered_keys.contains(&key) {
                    ordered_keys.push(key.clone());
                }
                entries.insert(key, val);
                i += 1 + extra_lines;
            } else {
                i += 1;
            }
        }

        Ok(EnvFile {
            path: path.to_string(),
            entries,
            ordered_keys,
            raw_lines,
        })
    }

    pub fn keys(&self) -> Vec<&String> {
        self.ordered_keys.iter().collect()
    }
}

// Returns (parsed_value, number_of_extra_lines_consumed).
fn parse_value(raw: &str, remaining: &[String]) -> (String, usize) {
    if raw.starts_with('"') {
        return parse_double_quoted(&raw[1..], remaining);
    }
    if raw.starts_with('\'') {
        return parse_single_quoted(&raw[1..]);
    }
    // Unquoted: strip trailing inline comment (must be preceded by a space)
    (strip_inline_comment(raw).trim_end().to_string(), 0)
}

// Handles escape sequences and multiline double-quoted values.
fn parse_double_quoted(after_open: &str, remaining: &[String]) -> (String, usize) {
    if let Some(close) = closing_double_quote(after_open) {
        return (after_open[..close].to_string(), 0);
    }
    // Value spans multiple lines — keep reading until the closing quote.
    let mut parts = vec![after_open.to_string()];
    for (idx, line) in remaining.iter().enumerate() {
        if let Some(close) = closing_double_quote(line) {
            parts.push(line[..close].to_string());
            return (parts.join("\n"), idx + 1);
        }
        parts.push(line.to_string());
    }
    // Unclosed quote — return what we have.
    (parts.join("\n"), remaining.len())
}

// Single-quoted values are fully literal: no escapes, no multiline, no comments.
fn parse_single_quoted(after_open: &str) -> (String, usize) {
    if let Some(close) = after_open.find('\'') {
        return (after_open[..close].to_string(), 0);
    }
    (after_open.to_string(), 0)
}

// Finds the index of the closing `"`, respecting `\"` escapes.
fn closing_double_quote(s: &str) -> Option<usize> {
    let mut escaped = false;
    for (i, c) in s.char_indices() {
        match (escaped, c) {
            (true, _) => escaped = false,
            (false, '\\') => escaped = true,
            (false, '"') => return Some(i),
            _ => {}
        }
    }
    None
}

// Strips ` # comment` from unquoted values. A bare `#` with no preceding space is kept.
fn strip_inline_comment(s: &str) -> &str {
    if let Some(pos) = s.find(" #") {
        &s[..pos]
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_tmp(name: &str, content: &str) -> String {
        let path = std::env::temp_dir().join(name);
        std::fs::write(&path, content).unwrap();
        path.to_string_lossy().into_owned()
    }

    #[test]
    fn parses_basic_env() {
        let p = write_tmp("basic.env", "FOO=bar\nBAZ=123\n");
        let f = EnvFile::load(&p).unwrap();
        assert_eq!(f.entries["FOO"], "bar");
        assert_eq!(f.entries["BAZ"], "123");
    }

    #[test]
    fn strips_double_quotes() {
        let p = write_tmp("dquote.env", "FOO=\"hello world\"\n");
        let f = EnvFile::load(&p).unwrap();
        assert_eq!(f.entries["FOO"], "hello world");
    }

    #[test]
    fn strips_single_quotes() {
        let p = write_tmp("squote.env", "BAR='single'\n");
        let f = EnvFile::load(&p).unwrap();
        assert_eq!(f.entries["BAR"], "single");
    }

    #[test]
    fn skips_comments() {
        let p = write_tmp("comments.env", "# comment\nFOO=bar\n");
        let f = EnvFile::load(&p).unwrap();
        assert_eq!(f.ordered_keys.len(), 1);
    }

    #[test]
    fn strips_export_prefix() {
        let p = write_tmp("export.env", "export FOO=bar\nexport BAZ=123\n");
        let f = EnvFile::load(&p).unwrap();
        assert_eq!(f.entries["FOO"], "bar");
        assert_eq!(f.entries["BAZ"], "123");
    }

    #[test]
    fn strips_inline_comments() {
        let p = write_tmp("inline.env", "FOO=bar # this is a comment\nBAZ=val#nospace\n");
        let f = EnvFile::load(&p).unwrap();
        assert_eq!(f.entries["FOO"], "bar");
        // no space before # — not a comment
        assert_eq!(f.entries["BAZ"], "val#nospace");
    }

    #[test]
    fn parses_multiline_double_quoted() {
        let p = write_tmp(
            "multiline.env",
            "CERT=\"-----BEGIN RSA PRIVATE KEY-----\nMIIE...\n-----END RSA PRIVATE KEY-----\"\nFOO=bar\n",
        );
        let f = EnvFile::load(&p).unwrap();
        assert_eq!(
            f.entries["CERT"],
            "-----BEGIN RSA PRIVATE KEY-----\nMIIE...\n-----END RSA PRIVATE KEY-----"
        );
        assert_eq!(f.entries["FOO"], "bar");
    }

    #[test]
    fn single_quoted_keeps_hash_literal() {
        let p = write_tmp("sq_hash.env", "FOO='bar # not a comment'\n");
        let f = EnvFile::load(&p).unwrap();
        assert_eq!(f.entries["FOO"], "bar # not a comment");
    }
}
