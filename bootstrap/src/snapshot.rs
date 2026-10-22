//! `ail test --update` (snap.update.1-6, ADR 0395): rewrite expected values
//! after a reviewed change. A failing `assert_eq(actual, LITERAL)` records
//! the literal that would pass and the test goes on; a doc example's
//! `# =>` lines take its output; `NAME.out` next to the file takes what
//! `ail run` prints. Each changed file is shown as a diff.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::diag::{Pos, json_str};

static ON: AtomicBool = AtomicBool::new(false);
static FOUND: Mutex<Vec<(Pos, Option<String>)>> = Mutex::new(Vec::new());

pub fn enable() {
    ON.store(true, Ordering::Relaxed);
}

/// Whether `ail test --update` is running (snap.update.1).
pub fn enabled() -> bool {
    ON.load(Ordering::Relaxed)
}

/// Record that the literal at `pos` should read `text` (snap.update.2).
/// `false` when the literal was reached before with another value: it is
/// then left alone and the assertion fails as usual (snap.update.3).
pub fn record(pos: Pos, text: String) -> bool {
    let mut found = FOUND.lock().unwrap_or_else(|e| e.into_inner());
    match found.iter_mut().find(|(p, _)| *p == pos) {
        Some((_, t)) if t.as_deref() != Some(text.as_str()) => {
            *t = None;
            false
        }
        Some(_) => true,
        None => {
            found.push((pos, Some(text)));
            true
        }
    }
}

/// What the tests recorded, in program positions; `None` marks a conflict.
pub fn take() -> Vec<(Pos, Option<String>)> {
    std::mem::take(&mut *FOUND.lock().unwrap_or_else(|e| e.into_inner()))
}

static DRY: AtomicBool = AtomicBool::new(false);

/// `--dry-run`: show the diffs, write nothing (snap.update.6).
pub fn set_dry_run() {
    DRY.store(true, Ordering::Relaxed);
}

/// A literal the tests asked to change: file, line, column and new text
/// (`None`: two runs disagreed).
pub type Located = (String, usize, usize, Option<String>);

/// Write what `ail test --update FILE` found and print the diffs, then one
/// summary line (snap.update.1-6). `src` is FILE's text; `docs` holds each
/// doc example's `# =>` lines and its output.
pub fn write(path: &str, src: &str, found: Vec<Located>, docs: Vec<(Vec<usize>, String)>, json: bool) {
    let dry = DRY.load(Ordering::Relaxed);
    let lines = src.split('\n').count();
    let mut files: Vec<(String, Vec<(usize, usize, String)>)> = Vec::new();
    let mut skipped = 0;
    for (file, line, col, text) in found {
        // Doc examples are compiled after the file's last line; their `# =>` lines are updated instead.
        if file.starts_with("gen:") || (file == path && line > lines) {
            continue;
        }
        let Some(text) = text else {
            skipped += 1;
            continue;
        };
        match files.iter_mut().find(|(f, _)| *f == file) {
            Some((_, at)) => at.push((line, col, text)),
            None => files.push((file, vec![(line, col, text)])),
        }
    }
    if !docs.is_empty() && !files.iter().any(|(f, _)| f == path) {
        files.insert(0, (path.to_string(), Vec::new()));
    }
    let mut out = String::new();
    let (mut count, mut changed) = (0, 0);
    for (file, at) in &files {
        let old = if file == path { src.to_string() } else { std::fs::read_to_string(file).unwrap_or_default() };
        let (mut new, mut changes) = rewrite_literals(&old, at);
        if file == path {
            // Later examples first, so earlier line numbers stay valid.
            for (lines, output) in docs.iter().rev() {
                if let Some((n, c)) = rewrite_doc(&new, lines, output) {
                    new = n;
                    changes.push(c);
                }
            }
        }
        if changes.is_empty() {
            continue;
        }
        changes.sort_by_key(|c| (c.line, c.col));
        out.push_str(&report(file, &old, &new, &changes, json));
        count += changes.len();
        changed += 1;
        if !dry && let Err(e) = std::fs::write(file, &new) {
            eprintln!("error: can't write {file}: {e}");
        }
    }
    // snap.update.5: `NAME.out` next to the file takes what `ail run` prints.
    let expected = std::path::Path::new(path).with_extension("out");
    if expected.is_file()
        && let Ok(exe) = std::env::current_exe()
        && let Ok(run) = std::process::Command::new(exe).args(["run", path]).stdin(std::process::Stdio::null()).output()
    {
        let old = std::fs::read_to_string(&expected).unwrap_or_default();
        let new = String::from_utf8_lossy(&run.stdout).into_owned();
        if old != new {
            let file = crate::shown::shown(&expected);
            let change = Change { kind: "out", line: 1, col: 1, old: old.clone(), new: new.clone() };
            out.push_str(&report(&file, &old, &new, std::slice::from_ref(&change), json));
            count += 1;
            changed += 1;
            if !dry && let Err(e) = std::fs::write(&expected, &new) {
                eprintln!("error: can't write {file}: {e}");
            }
        }
    }
    let verb = if dry { "would update" } else { "updated" };
    if json {
        out.push_str(&format!(
            "{{\"snapshots\":true,\"dry_run\":{dry},\"updated\":{count},\"files\":{changed},\"skipped\":{skipped}}}\n"
        ));
    } else {
        let s = |n: usize| if n == 1 { "" } else { "s" };
        out.push_str(&format!("{verb} {count} snapshot{} in {changed} file{}", s(count), s(changed)));
        if skipped > 0 {
            out.push_str(&format!("; left {skipped} whose value differed between runs (snap.update.3)"));
        }
        out.push('\n');
    }
    print!("{out}");
}

/// The AIL literal of a string (lex.str.1-2).
pub fn quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '{' => out.push_str("\\{"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            '\0' => out.push_str("\\0"),
            c if c.is_control() => out.push_str(&format!("\\u{{{:x}}}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// The AIL literal of a character (lex.str.4).
pub fn quote_char(c: char) -> String {
    match c {
        '\'' => "'\\''".into(),
        '\\' => "'\\\\'".into(),
        '\n' => "'\\n'".into(),
        '\t' => "'\\t'".into(),
        '\r' => "'\\r'".into(),
        '\0' => "'\\0'".into(),
        c if c.is_control() => format!("'\\u{{{:x}}}'", c as u32),
        c => format!("'{c}'"),
    }
}

/// The byte offset where the literal starting at `at` ends (snap.update.2):
/// a string (plain, raw or `"""`), a character, a number with an optional
/// `-`, `true`, `false`, or a list of these.
fn literal_end(s: &str, at: usize) -> Option<usize> {
    let rest = &s[at..];
    let b = rest.as_bytes();
    let first = *b.first()?;
    if rest.starts_with("\"\"\"") || rest.starts_with("r\"\"\"") {
        let open = if first == b'r' { 4 } else { 3 };
        let mut end = rest[open..].find("\"\"\"")? + open + 3;
        while b.get(end) == Some(&b'"') {
            end += 1;
        }
        return Some(at + end);
    }
    if rest.starts_with("r\"") {
        return Some(at + 2 + rest[2..].find('"')? + 1);
    }
    if first == b'"' || first == b'\'' {
        let mut i = 1;
        while i < b.len() {
            match b[i] {
                b'\\' => i += 2,
                c if c == first => return Some(at + i + 1),
                b'\n' => return None,
                _ => i += 1,
            }
        }
        return None;
    }
    if rest.starts_with("true") {
        return Some(at + 4);
    }
    if rest.starts_with("false") {
        return Some(at + 5);
    }
    if first == b'-' || first.is_ascii_digit() {
        let mut i = usize::from(first == b'-');
        while i < b.len() {
            let c = b[i];
            let sign = (c == b'+' || c == b'-') && i > 0 && matches!(b[i - 1], b'e' | b'E');
            if c.is_ascii_alphanumeric() || c == b'_' || c == b'.' || sign {
                i += 1;
            } else {
                break;
            }
        }
        return (i > usize::from(first == b'-')).then_some(at + i);
    }
    if first == b'[' {
        let mut depth = 0usize;
        let mut i = 0;
        while i < b.len() {
            match b[i] {
                b'[' => depth += 1,
                b']' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(at + i + 1);
                    }
                }
                b'"' | b'\'' => {
                    i = literal_end(s, at + i)? - at;
                    continue;
                }
                _ => {}
            }
            i += 1;
        }
    }
    None
}

/// The byte offset of 1-based `line` and character column `col`.
fn offset(s: &str, line: usize, col: usize) -> Option<usize> {
    let start = if line == 1 { 0 } else { s.match_indices('\n').nth(line - 2)?.0 + 1 };
    let text = &s[start..];
    let line_len = text.find('\n').unwrap_or(text.len());
    text[..line_len].char_indices().nth(col - 1).map(|(i, _)| start + i)
}

/// One change to one file.
pub struct Change {
    pub kind: &'static str,
    pub line: usize,
    pub col: usize,
    pub old: String,
    pub new: String,
}

/// Replace each literal at `(line, col)` with its new text. Returns the new
/// source and the changes; a literal that can't be found is skipped.
pub fn rewrite_literals(src: &str, at: &[(usize, usize, String)]) -> (String, Vec<Change>) {
    let mut spans: Vec<(usize, usize, usize, usize, &String)> = Vec::new();
    for (line, col, text) in at {
        let Some(start) = offset(src, *line, *col) else { continue };
        let Some(end) = literal_end(src, start) else { continue };
        if &src[start..end] != text {
            spans.push((start, end, *line, *col, text));
        }
    }
    spans.sort_by_key(|s| s.0);
    let mut out = src.to_string();
    let mut changes = Vec::new();
    for &(start, end, line, col, text) in spans.iter().rev() {
        changes.push(Change { kind: "assert_eq", line, col, old: src[start..end].to_string(), new: text.clone() });
        out.replace_range(start..end, text);
    }
    changes.reverse();
    (out, changes)
}

/// Replace a doc example's `# =>` lines (1-based `lines`, in order) with
/// `output` (snap.update.4).
pub fn rewrite_doc(src: &str, lines: &[usize], output: &str) -> Option<(String, Change)> {
    let first = *lines.first()?;
    let all: Vec<&str> = src.split('\n').collect();
    let model = all.get(first - 1)?;
    let prefix = &model[..model.find("# =>")?];
    let old: Vec<&str> = lines.iter().filter_map(|l| all.get(l - 1).copied()).collect();
    let new: Vec<String> = output
        .lines()
        .map(|l| if l.is_empty() { format!("{prefix}# =>") } else { format!("{prefix}# => {l}") })
        .collect();
    if old.iter().map(|s| s.trim_end()).eq(new.iter().map(|s| s.trim_end())) {
        return None;
    }
    let mut out: Vec<String> = Vec::with_capacity(all.len());
    for (i, l) in all.iter().enumerate() {
        if i + 1 == first {
            out.extend(new.iter().cloned());
        } else if !lines.contains(&(i + 1)) {
            out.push(l.to_string());
        }
    }
    let change = Change { kind: "doc", line: first, col: 1, old: old.join("\n"), new: new.join("\n") };
    Some((out.join("\n"), change))
}

/// The report of one changed file: its diff, or one JSON line per change.
pub fn report(path: &str, old: &str, new: &str, changes: &[Change], json: bool) -> String {
    if !json {
        let diff = crate::fmt::diff(path, old, new);
        return diff.replacen(" (formatted)\n", " (updated)\n", 1);
    }
    let mut out = String::new();
    for c in changes {
        out.push_str(&format!(
            "{{\"snapshot\":{},\"file\":{},\"line\":{},\"col\":{},\"old\":{},\"new\":{}}}\n",
            json_str(c.kind),
            json_str(path),
            c.line,
            c.col,
            json_str(&c.old),
            json_str(&c.new)
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal_ends() {
        let s = "f(x, \"a\\\"b\", 'c', -1.5e-3, true, [1, \"]\"], r\"x\\\", 42)";
        let ends: Vec<&str> = ["\"a", "'c", "-1", "true", "[1", "r\"", "42"]
            .iter()
            .map(|p| {
                let at = s.find(p).unwrap();
                &s[at..literal_end(s, at).unwrap()]
            })
            .collect();
        assert_eq!(ends, ["\"a\\\"b\"", "'c'", "-1.5e-3", "true", "[1, \"]\"]", "r\"x\\\"", "42"]);
    }

    #[test]
    fn quotes() {
        assert_eq!(quote("a\"{b}\n"), "\"a\\\"\\{b}\\n\"");
        assert_eq!(quote_char('\''), "'\\''");
    }

    #[test]
    fn rewrites() {
        let src = "@test fn t()\n    assert_eq(f(), \"old\")\n";
        let (out, ch) = rewrite_literals(src, &[(2, 20, "\"new\"".to_string())]);
        assert_eq!(out, "@test fn t()\n    assert_eq(f(), \"new\")\n");
        assert_eq!(ch[0].old, "\"old\"");
        let doc = "## ```ail\n## print(2)\n## # => 1\n## ```\n";
        let (out, _) = rewrite_doc(doc, &[3], "2\n3\n").unwrap();
        assert_eq!(out, "## ```ail\n## print(2)\n## # => 2\n## # => 3\n## ```\n");
    }
}
