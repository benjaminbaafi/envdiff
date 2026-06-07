use anyhow::Result;
use colored::*;

pub fn run_mask(path: &str, in_place: bool) -> Result<()> {
    let content = std::fs::read_to_string(path)
        .map_err(|_| anyhow::anyhow!("Cannot read file: {}", path))?;

    let mut out_lines: Vec<String> = Vec::new();
    let mut count = 0;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            out_lines.push(line.to_string());
            continue;
        }
        if let Some(eq_pos) = trimmed.find('=') {
            let key = &trimmed[..eq_pos];
            let val = &trimmed[eq_pos + 1..];
            if val.is_empty() {
                out_lines.push(line.to_string());
            } else {
                out_lines.push(format!("{}=****", key));
                count += 1;
            }
        } else {
            out_lines.push(line.to_string());
        }
    }

    let output = out_lines.join("\n") + "\n";

    if in_place {
        std::fs::write(path, &output)?;
        println!("{} Masked {} value(s) in {} (in-place).", "✓".green().bold(), count, path);
    } else {
        print!("{}", output);
    }

    Ok(())
}
