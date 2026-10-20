//! Diagnostics: every error cites the spec rule it enforces (ADR 0002) and can
//! be reported as human text or as JSON lines for AI tools (ADR 0024). The
//! JSON form is schema v1 (ADR 0145, `docs/design/semantic-api.md`): spans,
//! related notes and machine-applicable fixes.

use std::fmt::Write as _;

/// A source position, 1-based.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Pos {
    pub line: usize,
    pub col: usize,
}

/// The version of the JSON diagnostics schema (ADR 0145).
pub const API_VERSION: u32 = 1;

#[derive(Debug)]
pub struct Diagnostic {
    /// Spec rule ID, e.g. `mod.main.1`.
    pub rule: &'static str,
    pub pos: Pos,
    /// End of the span, exclusive. Without one, `report` uses the end of the
    /// token at `pos`.
    pub end: Option<Pos>,
    pub message: String,
    pub help: Option<String>,
    /// Warnings are reported but don't stop a program from running.
    pub warning: bool,
    /// Related positions, e.g. where a name was first declared.
    pub notes: Vec<Note>,
    /// Machine-applicable fixes, in order of preference.
    pub fixes: Vec<Fix>,
}

#[derive(Debug)]
pub struct Note {
    pub pos: Pos,
    pub message: String,
}

#[derive(Debug)]
pub struct Fix {
    pub title: String,
    pub edits: Vec<Edit>,
}

/// A position the checker names by a rule instead of by coordinates.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Anchor {
    /// The edit's `pos` and `end` are the position.
    Exact,
    /// Insert `text` (whole lines) after the indented block that starts at
    /// `pos`: the line after the last line below `pos.line` that is blank or
    /// indented deeper than `pos.col`.
    AfterBlock,
    /// Insert `text` at the end of the code on `pos.line`; dropped when the
    /// line has a comment or doesn't end like a complete expression.
    LineEnd,
}

/// Replace the text from `pos` to `end` (exclusive) with `text`.
///
/// When `old` is set, the edit is anchored instead: it replaces the first
/// whole-word occurrence of `old` at or after `pos` on the same line, so the
/// checker can propose an edit without the source text. A fix with an anchor
/// that doesn't match is dropped rather than reported wrong.
#[derive(Debug)]
pub struct Edit {
    pub pos: Pos,
    pub end: Pos,
    pub old: Option<String>,
    /// Where the checker can't name the position itself (fix.edit.2).
    pub anchor: Anchor,
    pub text: String,
}

impl Edit {
    pub fn replace(pos: Pos, old: impl Into<String>, text: impl Into<String>) -> Self {
        Edit { pos, end: pos, old: Some(old.into()), anchor: Anchor::Exact, text: text.into() }
    }

    /// Insert `text` at `pos`.
    pub fn insert(pos: Pos, text: impl Into<String>) -> Self {
        Edit { pos, end: pos, old: None, anchor: Anchor::Exact, text: text.into() }
    }

    /// Insert `text` at the end of the code on `pos.line`.
    pub fn at_line_end(pos: Pos, text: impl Into<String>) -> Self {
        Edit { pos, end: pos, old: None, anchor: Anchor::LineEnd, text: text.into() }
    }

    /// Append the lines `text` (each ending in a newline) after the block
    /// starting at `pos`.
    pub fn after_block(pos: Pos, text: impl Into<String>) -> Self {
        Edit { pos, end: pos, old: None, anchor: Anchor::AfterBlock, text: text.into() }
    }
}

impl Diagnostic {
    pub fn new(rule: &'static str, pos: Pos, message: impl Into<String>) -> Self {
        Diagnostic {
            rule,
            pos,
            end: None,
            message: message.into(),
            help: None,
            warning: false,
            notes: Vec::new(),
            fixes: Vec::new(),
        }
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

    pub fn note(mut self, pos: Pos, message: impl Into<String>) -> Self {
        self.notes.push(Note { pos, message: message.into() });
        self
    }

    pub fn fix(mut self, title: impl Into<String>, edits: Vec<Edit>) -> Self {
        self.fixes.push(Fix { title: title.into(), edits });
        self
    }

    /// "did you mean `s`?" as help, plus the fix that replaces `name` by `s`.
    pub fn did_you_mean(self, name: &str, s: &str) -> Self {
        let pos = self.pos;
        self.help(format!("did you mean `{s}`?"))
            .fix(format!("replace `{name}` with `{s}`"), vec![Edit::replace(pos, name, s)])
    }

    /// Settle the span and the anchored edits against the source.
    fn resolve(&mut self, lines: &[Vec<char>]) {
        if self.end.is_none() {
            self.end = Some(token_end(lines, self.pos));
        }
        self.fixes.retain_mut(|f| f.edits.iter_mut().all(|e| e.resolve(lines)));
    }
}

impl Edit {
    fn resolve(&mut self, lines: &[Vec<char>]) -> bool {
        match std::mem::replace(&mut self.anchor, Anchor::Exact) {
            Anchor::AfterBlock => return self.resolve_after_block(lines),
            Anchor::LineEnd => return self.resolve_line_end(lines),
            Anchor::Exact => {}
        }
        let Some(old) = self.old.take() else { return true };
        let Some(line) = lines.get(self.pos.line.wrapping_sub(1)) else { return false };
        let old: Vec<char> = old.chars().collect();
        let word = |c: char| c.is_alphanumeric() || c == '_';
        let (word_start, word_end) = (old.first().is_some_and(|c| word(*c)), old.last().is_some_and(|c| word(*c)));
        let start = self.pos.col.saturating_sub(1);
        for i in start..line.len() {
            if line[i..].starts_with(&old)
                && !(word_start && i > 0 && word(line[i - 1]))
                && !(word_end && line.get(i + old.len()).is_some_and(|c| word(*c)))
            {
                self.pos.col = i + 1;
                self.end = Pos { line: self.pos.line, col: i + old.len() + 1 };
                return true;
            }
        }
        false
    }
}

impl Edit {
    fn resolve_line_end(&mut self, lines: &[Vec<char>]) -> bool {
        let Some(line) = lines.get(self.pos.line.wrapping_sub(1)) else { return false };
        let code = line.iter().rposition(|c| !c.is_whitespace()).map_or(0, |i| i + 1);
        let Some(last) = code.checked_sub(1).map(|i| line[i]) else { return false };
        if line.contains(&'#') || !(last == ')' || last == ']' || last.is_alphanumeric() || last == '_') {
            return false;
        }
        self.pos.col = code + 1;
        self.end = self.pos;
        true
    }

    fn resolve_after_block(&mut self, lines: &[Vec<char>]) -> bool {
        let start = self.pos.line.wrapping_sub(1);
        if lines.get(start).is_none() {
            return false;
        }
        let indent = self.pos.col - 1;
        let mut last = start;
        for (i, l) in lines.iter().enumerate().skip(start + 1) {
            let blank = l.iter().all(|c| c.is_whitespace());
            let deeper = l.iter().take_while(|c| **c == ' ').count() > indent;
            if !blank && !deeper {
                break;
            }
            if !blank {
                last = i;
            }
        }
        // A final line with no newline after it is the last element of
        // `split('\n')`: its successor doesn't exist.
        let at = Pos { line: last + 2, col: 1 };
        if last + 1 < lines.len() {
            self.pos = at;
        } else {
            self.pos = Pos { line: last + 1, col: lines[last].len() + 1 };
            self.text.insert(0, '\n');
            self.text.pop();
        }
        self.end = self.pos;
        true
    }
}

/// The exclusive end of the token starting at `pos`: a word, a number, a
/// string literal or a symbol. Past the end of the line, the start itself.
fn token_end(lines: &[Vec<char>], pos: Pos) -> Pos {
    let Some(line) = lines.get(pos.line.wrapping_sub(1)) else { return pos };
    let i = pos.col.saturating_sub(1);
    let Some(&c) = line.get(i) else { return pos };
    let len = if c.is_alphanumeric() || c == '_' {
        line[i..].iter().take_while(|c| c.is_alphanumeric() || **c == '_').count()
    } else if c == '"' || c == '\'' {
        let mut j = i + 1;
        while j < line.len() && line[j] != c {
            j += if line[j] == '\\' { 2 } else { 1 };
        }
        (j + 1).min(line.len()) - i
    } else {
        crate::lexer::SYMBOLS
            .iter()
            .find(|s| line[i..].iter().take(s.chars().count()).copied().eq(s.chars()))
            .map_or(1, |s| s.chars().count())
    };
    Pos { line: pos.line, col: pos.col + len }
}

/// Sort diagnostics and settle their spans and anchored edits against
/// `src`, as `report` does before printing; `ail fix` applies the result.
pub fn settle(diags: &mut [Diagnostic], src: &str) {
    diags.sort_by_key(|d| (d.pos.line, d.pos.col));
    let lines: Vec<Vec<char>> = src.split('\n').map(|l| l.strip_suffix('\r').unwrap_or(l).chars().collect()).collect();
    for d in diags.iter_mut() {
        d.resolve(&lines);
    }
}

/// Write all diagnostics to stderr, sorted by position for stable output.
pub fn report(diags: &mut [Diagnostic], path: &str, src: &str, json: bool) {
    settle(diags, src);
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
    for n in &d.notes {
        let _ = writeln!(s, "note: {path}:{}:{}: {}", n.pos.line, n.pos.col, n.message);
    }
    for f in &d.fixes {
        let _ = writeln!(s, "fix: {}", f.title);
    }
    s.trim_end().to_string()
}

/// One diagnostic as a schema v1 JSON object (ADR 0145). Keys appear in a
/// fixed order; optional keys are omitted when empty.
fn to_json(d: &Diagnostic, path: &str) -> String {
    let mut s = format!("{{\"api\":{API_VERSION},\"severity\":{}", json_str(d.severity()));
    let _ = write!(s, ",\"rule\":{}", json_str(d.rule));
    let _ = write!(s, ",\"file\":{}", json_str(path));
    let _ = write!(s, ",\"line\":{},\"col\":{}", d.pos.line, d.pos.col);
    let _ = write!(s, ",\"message\":{}", json_str(&d.message));
    if let Some(help) = &d.help {
        let _ = write!(s, ",\"help\":{}", json_str(help));
    }
    let end = d.end.unwrap_or(d.pos);
    let _ = write!(s, ",\"end_line\":{},\"end_col\":{}", end.line, end.col);
    if !d.notes.is_empty() {
        s.push_str(",\"notes\":[");
        for (i, n) in d.notes.iter().enumerate() {
            let sep = if i > 0 { "," } else { "" };
            let _ = write!(
                s,
                "{sep}{{\"line\":{},\"col\":{},\"message\":{}}}",
                n.pos.line,
                n.pos.col,
                json_str(&n.message)
            );
        }
        s.push(']');
    }
    if !d.fixes.is_empty() {
        s.push_str(",\"fixes\":[");
        for (i, f) in d.fixes.iter().enumerate() {
            let sep = if i > 0 { "," } else { "" };
            let _ = write!(s, "{sep}{{\"title\":{},\"edits\":[", json_str(&f.title));
            for (j, e) in f.edits.iter().enumerate() {
                let sep = if j > 0 { "," } else { "" };
                let _ = write!(
                    s,
                    "{sep}{{\"line\":{},\"col\":{},\"end_line\":{},\"end_col\":{},\"text\":{}}}",
                    e.pos.line,
                    e.pos.col,
                    e.end.line,
                    e.end.col,
                    json_str(&e.text)
                );
            }
            s.push_str("]}");
        }
        s.push(']');
    }
    s.push('}');
    s
}

pub fn json_str(v: &str) -> String {
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
            "{\"api\":1,\"severity\":\"error\",\"rule\":\"mod.main.1\",\"file\":\"x.ail\",\"line\":2,\"col\":5,\"message\":\"msg\",\"help\":\"fix\",\"end_line\":2,\"end_col\":5}"
        );
    }

    fn lines(src: &str) -> Vec<Vec<char>> {
        src.lines().map(|l| l.chars().collect()).collect()
    }

    #[test]
    fn span_defaults_to_the_token_end() {
        let ls = lines("    prnt(\"hi\") := x");
        assert_eq!(token_end(&ls, Pos { line: 1, col: 5 }), Pos { line: 1, col: 9 });
        assert_eq!(token_end(&ls, Pos { line: 1, col: 10 }), Pos { line: 1, col: 14 });
        assert_eq!(token_end(&ls, Pos { line: 1, col: 16 }), Pos { line: 1, col: 18 });
        assert_eq!(token_end(&ls, Pos { line: 1, col: 99 }), Pos { line: 1, col: 99 });
    }

    #[test]
    fn json_has_notes_and_fixes() {
        let mut d = Diagnostic::new("expr.name.1", Pos { line: 1, col: 1 }, "unknown name `prnt`")
            .did_you_mean("prnt", "print")
            .note(Pos { line: 3, col: 2 }, "declared here");
        d.resolve(&lines("prnt(1)"));
        assert_eq!(
            to_json(&d, "x.ail"),
            "{\"api\":1,\"severity\":\"error\",\"rule\":\"expr.name.1\",\"file\":\"x.ail\",\"line\":1,\"col\":1,\
             \"message\":\"unknown name `prnt`\",\"help\":\"did you mean `print`?\",\"end_line\":1,\"end_col\":5,\
             \"notes\":[{\"line\":3,\"col\":2,\"message\":\"declared here\"}],\
             \"fixes\":[{\"title\":\"replace `prnt` with `print`\",\"edits\":[{\"line\":1,\"col\":1,\"end_line\":1,\"end_col\":5,\"text\":\"print\"}]}]}"
        );
    }

    #[test]
    fn anchored_edit_matches_whole_words_or_is_dropped() {
        let mut d = Diagnostic::new("r", Pos { line: 1, col: 1 }, "m").did_you_mean("x", "y");
        d.resolve(&lines("xx := x"));
        assert_eq!((d.fixes[0].edits[0].pos.col, d.fixes[0].edits[0].end.col), (7, 8));
        let mut d = Diagnostic::new("r", Pos { line: 1, col: 1 }, "m").did_you_mean("z", "y");
        d.resolve(&lines("xx := x"));
        assert!(d.fixes.is_empty());
    }
}
