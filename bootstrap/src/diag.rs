//! Diagnostics: every error cites the spec rule it enforces (ADR 0002) and can
//! be reported as human text or as JSON lines for AI tools (ADR 0024).

use std::fmt::Write as _;

/// A source position, 1-based.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Pos {
    pub line: usize,
    pub col: usize,
}

#[derive(Debug)]
pub struct Diagnostic {
    /// Spec rule ID, e.g. `mod.main.1`.
    pub rule: &'static str,
    pub pos: Pos,
    pub message: String,
    pub help: Option<String>,
    /// Warnings are reported but don't stop a program from running.
    pub warning: bool,
}

impl Diagnostic {
    pub fn new(rule: &'static str, pos: Pos, message: impl Into<String>) -> Self {
        Diagnostic { rule, pos, message: message.into(), help: None, warning: false }
    }

    pub fn warning(rule: &'static str, pos: Pos, message: impl Into<String>) -> Self {
        Diagnostic { warning: true, ..Diagnostic::new(rule, pos, message) }
    }

    fn severity(&self) -> &'static str {
        if self.warning { "warning" } else { "error" }
    }

    pub fn help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }
}

/// Write all diagnostics to stderr, sorted by position for stable output.
pub fn report(diags: &mut [Diagnostic], path: &str, src: &str, json: bool) {
    diags.sort_by_key(|d| (d.pos.line, d.pos.col));
    let mut out = String::new();
    for d in diags.iter() {
        if json {
            out.push_str(&to_json(d, path));
        } else {
            out.push_str(&to_text(d, path, src));
        }
        out.push('\n');
    }
    eprint!("{out}");
}

fn to_text(d: &Diagnostic, path: &str, src: &str) -> String {
    let mut s = format!("{path}:{}:{}: {}[{}]: {}\n", d.pos.line, d.pos.col, d.severity(), d.rule, d.message);
    if let Some(line) = src.lines().nth(d.pos.line.saturating_sub(1)) {
        let _ = writeln!(s, "    {line}");
        let _ = writeln!(s, "    {}^", " ".repeat(d.pos.col.saturating_sub(1)));
    }
    if let Some(help) = &d.help {
        let _ = writeln!(s, "help: {help}");
    }
    s.trim_end().to_string()
}

fn to_json(d: &Diagnostic, path: &str) -> String {
    let mut s = format!("{{\"severity\":{}", json_str(d.severity()));
    let _ = write!(s, ",\"rule\":{}", json_str(d.rule));
    let _ = write!(s, ",\"file\":{}", json_str(path));
    let _ = write!(s, ",\"line\":{},\"col\":{}", d.pos.line, d.pos.col);
    let _ = write!(s, ",\"message\":{}", json_str(&d.message));
    if let Some(help) = &d.help {
        let _ = write!(s, ",\"help\":{}", json_str(help));
    }
    s.push('}');
    s
}

fn json_str(v: &str) -> String {
    let mut s = String::with_capacity(v.len() + 2);
    s.push('"');
    for c in v.chars() {
        match c {
            '"' => s.push_str("\\\""),
            '\\' => s.push_str("\\\\"),
            '\n' => s.push_str("\\n"),
            '\r' => s.push_str("\\r"),
            '\t' => s.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(s, "\\u{:04x}", c as u32);
            }
            c => s.push(c),
        }
    }
    s.push('"');
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_escapes_control_and_quotes() {
        assert_eq!(json_str("a\"b\\c\nd\u{1}"), "\"a\\\"b\\\\c\\nd\\u0001\"");
    }

    #[test]
    fn json_diagnostic_has_rule_and_position() {
        let d = Diagnostic::new("mod.main.1", Pos { line: 2, col: 5 }, "msg").help("fix");
        assert_eq!(
            to_json(&d, "x.ail"),
            "{\"severity\":\"error\",\"rule\":\"mod.main.1\",\"file\":\"x.ail\",\"line\":2,\"col\":5,\"message\":\"msg\",\"help\":\"fix\"}"
        );
    }
}
