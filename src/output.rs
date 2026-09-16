use crate::error::Error;
use serde_json::{json, Value};
use std::io::{self, IsTerminal, Write};

pub fn bar(percent: f64, width: usize) -> String {
    let cells = width.saturating_sub(2);
    let filled = ((percent.clamp(0.0, 100.0) / 100.0 * cells as f64).round() as usize).min(cells);
    format!("[{}{}]", "#".repeat(filled), "-".repeat(cells - filled))
}
pub fn terminal_width() -> usize {
    #[cfg(unix)]
    unsafe {
        let mut size: libc::winsize = std::mem::zeroed();
        if libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut size) == 0 && size.ws_col > 0 {
            return (size.ws_col as usize).max(10);
        }
    }
    80
}
fn label(s: &str) -> String {
    let s = s.replace('_', " ");
    let mut c = s.chars();
    match c.next() {
        Some(first) => first.to_uppercase().collect::<String>() + c.as_str(),
        None => s,
    }
}
fn scalar(v: &Value) -> String {
    match v {
        Value::Null => "Unavailable".into(),
        Value::String(s) => s.clone(),
        _ => v.to_string(),
    }
}
fn safe_text(s: &str) -> String {
    s.chars()
        .flat_map(|c| {
            if (c.is_control() && c != '\n' && c != '\t') || matches!(c,'\u{202a}'..='\u{202e}'|'\u{2066}'..='\u{2069}'|'\u{200e}'|'\u{200f}'|'\u{061c}') {
                format!("\\u{{{:04x}}}", c as u32)
                    .chars()
                    .collect::<Vec<_>>()
            } else {
                vec![c]
            }
        })
        .collect()
}
pub fn prompt_text(s: &str) -> String {
    safe_text(s).replace('\n', "\\n").replace('\t', "\\t")
}
fn render(v: &Value, plain: bool, width: usize, depth: usize, lines: &mut Vec<String>) {
    let indent = "  ".repeat(depth);
    match v {
        Value::Object(map) => {
            if let (Some(title), Some(percent)) = (
                map.get("title").and_then(Value::as_str),
                map.get("percent").and_then(Value::as_f64),
            ) {
                lines.push(format!("{indent}{}  {:.0}%", title.to_uppercase(), percent));
                if !plain {
                    lines.push(format!(
                        "{indent}{}",
                        bar(percent, width.saturating_sub(depth * 2).clamp(4, 50))
                    ));
                }
            }
            for (k, v) in map {
                if map.contains_key("title")
                    && map.get("percent").and_then(Value::as_f64).is_some()
                    && (k == "title" || k == "percent")
                {
                    continue;
                }
                if v.is_object() || v.is_array() {
                    lines.push(format!("{indent}{}:", label(k)));
                    render(v, plain, width, depth + 1, lines);
                } else {
                    lines.push(format!("{indent}{}: {}", label(k), scalar(v)));
                }
            }
        }
        Value::Array(values) => {
            if values.is_empty() {
                lines.push(format!("{indent}No results."));
            }
            for (i, v) in values.iter().enumerate() {
                if i > 0 {
                    lines.push(String::new());
                }
                render(v, plain, width, depth, lines);
            }
        }
        _ => lines.push(format!("{indent}{}", scalar(v))),
    }
}
pub fn format_result(v: &Value, plain: bool, width: usize) -> String {
    let mut lines = Vec::new();
    render(v, plain, width, 0, &mut lines);
    let text = safe_text(&lines.join("\n"));
    // Wrap by characters; long paths remain readable and no byte slicing corrupts Unicode.
    text.lines()
        .flat_map(|line| {
            let chars: Vec<char> = line.chars().collect();
            if chars.is_empty() {
                vec![String::new()]
            } else {
                chars
                    .chunks(width.max(1))
                    .map(|c| c.iter().collect())
                    .collect()
            }
        })
        .collect::<Vec<String>>()
        .join("\n")
        + "\n"
}
pub fn emit(command: &str, result: Result<Value, Error>, json_mode: bool, plain: bool) -> i32 {
    let failed = result.is_err();
    let text = if json_mode {
        let envelope = match result {
            Ok(data) => json!({"schema_version":1,"command":command,"ok":true,"data":data}),
            Err(e) => json!({"schema_version":1,"command":command,"ok":false,"error":e}),
        };
        serde_json::to_string(&envelope).unwrap() + "\n"
    } else {
        match result {
            Ok(data) => format_result(
                &data,
                plain || !io::stdout().is_terminal(),
                terminal_width(),
            ),
            Err(e) => {
                let _ = writeln!(
                    io::stderr(),
                    "error: {} [{}]",
                    safe_text(&e.message),
                    e.code
                );
                return 1;
            }
        }
    };
    if let Err(e) = io::stdout().write_all(text.as_bytes()) {
        if e.kind() != io::ErrorKind::BrokenPipe {
            let _ = writeln!(io::stderr(), "error: Could not write output: {e}");
            return 1;
        }
    }
    i32::from(failed)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn prompts_escape_line_breaks_bidi_and_terminal_sequences() {
        let prompt = prompt_text("delete\nYES\t\u{001b}[2J\u{202e}txt");
        assert!(!prompt.contains('\n'));
        assert!(!prompt.contains('\t'));
        assert!(!prompt.contains('\u{001b}'));
        assert!(!prompt.contains('\u{202e}'));
        assert!(prompt.contains("\\n"));
    }
    #[test]
    fn bars_are_proportional() {
        assert_eq!(bar(0., 12), "[----------]");
        assert_eq!(bar(50., 12), "[#####-----]");
        assert_eq!(bar(100., 12), "[##########]");
        assert_eq!(bar(78., 32).matches('#').count(), 23);
    }
    #[test]
    fn plain_and_narrow_outputs() {
        let v = json!({"title":"Battery","percent":78,"name":"İstanbul","remaining":null});
        let s = format_result(&v, true, 12);
        assert!(!s.contains('#'));
        assert!(s.lines().all(|l| l.chars().count() <= 12));
        assert!(format_result(&v, true, 80).contains("Unavailable"));
    }
    #[test]
    fn control_characters_cannot_inject_ansi() {
        assert!(!format_result(&json!({"text":"\u{1b}[31m"}), false, 80).contains('\u{1b}'));
    }
}
