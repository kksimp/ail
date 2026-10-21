//! Diagnostics: every error cites the spec rule it enforces (ADR 0002) and can
//! be reported as human text or as JSON lines for AI tools (ADR 0024). The
//! JSON form is schema v1 (ADR 0145, `docs/design/semantic-api.md`): spans,
//! related notes and machine-applicable fixes.

use std::fmt::Write as _;

mod rewrite;

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
    /// Insert `text` just after the expression that starts at `pos`, which
    /// ends at the first top-level `,`, `)`, `]`, `}`, blank or comment, or the
    /// end of the line (err.fix.1). Dropped when a bracket stays open at the
    /// end of the line.
    AfterExpr,
    /// Rewrite the call `old(args)` that starts at `pos` on one line; `text`
    /// is a template: `$1` `$2` are the arguments as written, `$r1` the first
    /// as a method receiver (parenthesised if it isn't a plain postfix
    /// expression), `$*` all of them as `{...}` pieces of one string
    /// (ADR 0264); `$%` a `printf` call's arguments as one interpolated string
    /// (ADR 0335). Dropped when the call isn't on one line or the argument
    /// count doesn't fit.
    Call,
    /// Rewrite a bracketed form `old[args]` like `Call` rewrites `old(args)`.
    Brackets,
    /// Like `Call`, but `text` is a comma-separated list of field names and
    /// each argument is prefixed with `name: `.
    CallNamed,
    /// For a chained comparison `a < b < c`: `pos` is the first operator and
    /// `end` the second. Inserts `and b ` before the second, so the middle
    /// operand is written twice.
    Chain,
    /// Delete the character at `pos` and the blanks before it.
    TrimBefore,
    /// Rewrite `l op r` around the operator at `pos` by a template (ADR 0335).
    Operands,
    /// A C-style `for (i := a; i < b; i += 1)` header as `for i in a..b`.
    CFor,
    /// `xs: List[int] = []` as `xs := List[int]()`.
    EmptyDecl,
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

    /// Insert `text` just after the expression that starts at `pos`.
    pub fn after_expr(pos: Pos, text: impl Into<String>) -> Self {
        Edit { pos, end: pos, old: None, anchor: Anchor::AfterExpr, text: text.into() }
    }

    /// Delete the character at `pos` and the blanks before it.
    pub fn trim_before(pos: Pos) -> Self {
        Edit { pos, end: pos, old: None, anchor: Anchor::TrimBefore, text: String::new() }
    }

    /// Replace the span `pos..end` with `text`.
    pub fn span(pos: Pos, end: Pos, text: impl Into<String>) -> Self {
        Edit { pos, end, old: None, anchor: Anchor::Exact, text: text.into() }
    }

    /// Delete the span `pos..end`.
    pub fn delete(pos: Pos, end: Pos) -> Self {
        Edit::span(pos, end, "")
    }

    /// Rewrite the call `callee(...)` at `pos` by a template (`Anchor::Call`).
    pub fn call(pos: Pos, callee: impl Into<String>, template: impl Into<String>) -> Self {
        Edit { pos, end: pos, old: Some(callee.into()), anchor: Anchor::Call, text: template.into() }
    }

    /// Rewrite `callee[args]` by a template (`Anchor::Brackets`).
    pub fn call_with(pos: Pos, callee: impl Into<String>, template: impl Into<String>) -> Self {
        Edit { pos, end: pos, old: Some(callee.into()), anchor: Anchor::Brackets, text: template.into() }
    }

    /// Name the arguments of the call `callee(...)` (`Anchor::CallNamed`).
    pub fn call_named(pos: Pos, callee: impl Into<String>, names: &[&str]) -> Self {
        Edit { pos, end: pos, old: Some(callee.into()), anchor: Anchor::CallNamed, text: names.join(",") }
    }

    /// Split a chained comparison between its two operators (`Anchor::Chain`).
    pub fn chain(first: Pos, second: Pos) -> Self {
        Edit { pos: first, end: second, old: None, anchor: Anchor::Chain, text: String::new() }
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

    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
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
        if self.help.is_none() && !self.warning {
            self.help = default_help(self.rule).map(String::from);
        }
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
            Anchor::AfterExpr => return self.resolve_after_expr(lines),
            a @ (Anchor::Call | Anchor::CallNamed | Anchor::Brackets) => return self.resolve_call(lines, a),
            Anchor::Chain => return self.resolve_chain(lines),
            Anchor::TrimBefore => return self.resolve_trim(lines),
            Anchor::Operands => return self.resolve_operands(lines),
            Anchor::CFor => return self.resolve_c_for(lines),
            Anchor::EmptyDecl => return self.resolve_empty_decl(lines),
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

/// The arguments of the call whose `(` is at `open` in `line`, as written,
/// and the index of its `)`. `None` when the call doesn't close on the line.
fn call_args(line: &[char], open: usize) -> Option<(Vec<String>, usize)> {
    let (mut depth, mut start, mut i) = (0usize, open + 1, open);
    let mut args: Vec<String> = Vec::new();
    while i < line.len() {
        match line[i] {
            '"' | '\'' => {
                let q = line[i];
                i += 1;
                while i < line.len() && line[i] != q {
                    i += if line[i] == '\\' { 2 } else { 1 };
                }
            }
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    let last: String = line[start..i].iter().collect();
                    if !last.trim().is_empty() || !args.is_empty() {
                        args.push(last.trim().to_string());
                    }
                    return Some((args, i));
                }
            }
            ',' if depth == 1 => {
                args.push(line[start..i].iter().collect::<String>().trim().to_string());
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// `a` as a method receiver: parenthesised unless it is a name, a literal
/// or a postfix chain.
fn receiver(a: &str) -> String {
    let plain = a.chars().all(|c| c.is_alphanumeric() || "_.()[]\"".contains(c))
        && a.chars().next().is_some_and(|c| !c.is_ascii_digit());
    if plain { a.to_string() } else { format!("({a})") }
}

impl Edit {
    fn resolve_call(&mut self, lines: &[Vec<char>], anchor: Anchor) -> bool {
        let Some(callee) = self.old.take() else { return false };
        let Some(line) = lines.get(self.pos.line.wrapping_sub(1)) else { return false };
        let start = self.pos.col.saturating_sub(1);
        // A qualified constructor (`date.Date(...)`) is written with its module's
        // alias: the callee is the dotted name at `pos` whose last part is `callee`.
        let callee = match anchor {
            Anchor::CallNamed => {
                let rest = &line[start.min(line.len())..];
                let len = rest.iter().take_while(|c| c.is_alphanumeric() || **c == '_' || **c == '.').count();
                let written: String = rest[..len].iter().collect();
                if written.rsplit('.').next() == Some(callee.as_str()) { written } else { callee }
            }
            _ => callee,
        };
        let name: Vec<char> = callee.chars().collect();
        let open = if anchor == Anchor::Brackets { '[' } else { '(' };
        if !line[start.min(line.len())..].starts_with(&name) || line.get(start + name.len()) != Some(&open) {
            return false;
        }
        let Some((args, close)) = call_args(line, start + name.len()) else { return false };
        let text = if anchor == Anchor::CallNamed {
            let names: Vec<&str> = self.text.split(',').collect();
            if names.len() != args.len() {
                return false;
            }
            let named: Vec<String> = names.iter().zip(&args).map(|(n, a)| format!("{n}: {a}")).collect();
            format!("{callee}({})", named.join(", "))
        } else {
            let t = std::mem::take(&mut self.text);
            let mut out = String::new();
            let mut it = t.chars().peekable();
            while let Some(c) = it.next() {
                if c != '$' {
                    out.push(c);
                    continue;
                }
                let recv = it.next_if_eq(&'r').is_some();
                match it.next() {
                    Some('a') => out.push_str(&args.join(", ")),
                    Some('%') => match rewrite::printf_text(&args) {
                        Some(text) => out.push_str(&text),
                        None => return false,
                    },
                    Some('*') => {
                        let mut s = String::new();
                        for (k, a) in args.iter().enumerate() {
                            if k > 0 {
                                s.push(' ');
                            }
                            match a.strip_prefix('"').and_then(|a| a.strip_suffix('"')) {
                                Some(lit) if !lit.contains('"') => s.push_str(lit),
                                _ if a.contains('"') => return false,
                                _ => s.push_str(&format!("{{{a}}}")),
                            }
                        }
                        out.push('"');
                        out.push_str(&s);
                        out.push('"');
                    }
                    Some(d) if d.is_ascii_digit() => {
                        let Some(a) = args.get(d as usize - '1' as usize) else { return false };
                        if t.contains("\"{$") && a.contains('"') {
                            return false;
                        }
                        out.push_str(&if recv { receiver(a) } else { a.clone() });
                    }
                    _ => return false,
                }
            }
            out
        };
        self.end = Pos { line: self.pos.line, col: close + 2 };
        self.text = text;
        true
    }

    fn resolve_trim(&mut self, lines: &[Vec<char>]) -> bool {
        self.anchor = Anchor::Exact;
        let Some(line) = lines.get(self.pos.line.wrapping_sub(1)) else { return false };
        let at = self.pos.col - 1;
        if at >= line.len() {
            return false;
        }
        let blanks = line[..at].iter().rev().take_while(|c| **c == ' ' || **c == '\t').count();
        self.end = Pos { line: self.pos.line, col: at + 2 };
        self.pos.col = at + 1 - blanks;
        true
    }

    fn resolve_chain(&mut self, lines: &[Vec<char>]) -> bool {
        self.anchor = Anchor::Exact;
        let Some(line) = lines.get(self.pos.line.wrapping_sub(1)) else { return false };
        if self.end.line != self.pos.line {
            return false;
        }
        let op_len = line[self.pos.col - 1..].iter().take_while(|c| "<>=!".contains(**c)).count();
        let from = self.pos.col - 1 + op_len;
        let to = self.end.col - 1;
        if to <= from || to > line.len() {
            return false;
        }
        let mid: String = line[from..to].iter().collect();
        let mid = mid.trim();
        if mid.is_empty() {
            return false;
        }
        self.text = format!("and {mid} ");
        self.pos = self.end;
        true
    }

    fn resolve_after_expr(&mut self, lines: &[Vec<char>]) -> bool {
        let Some(line) = lines.get(self.pos.line.wrapping_sub(1)) else { return false };
        let (mut depth, mut quote) = (0usize, false);
        let mut i = self.pos.col.saturating_sub(1);
        // An operand starts with a name, a bracket or a quote, not an operator.
        if !line.get(i).is_some_and(|c| c.is_alphanumeric() || matches!(c, '_' | '(' | '[' | '"')) {
            return false;
        }
        while i < line.len() {
            let c = line[i];
            if quote {
                match c {
                    '\\' => i += 1,
                    '"' => quote = false,
                    _ => {}
                }
            } else {
                match c {
                    '"' => quote = true,
                    '(' | '[' | '{' => depth += 1,
                    ')' | ']' | '}' | ',' if depth == 0 => break,
                    ')' | ']' | '}' => depth -= 1,
                    '#' => break,
                    c if c.is_whitespace() && depth == 0 => break,
                    _ => {}
                }
            }
            i += 1;
        }
        if depth != 0 || quote || i == 0 {
            return false;
        }
        self.pos.col = i + 1;
        self.end = self.pos;
        true
    }

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
        // The text form repeats the help line when the fix says the same thing
        // (`write `and``); the JSON form (ADR 0145) keeps every fix title.
        if d.help.as_deref() != Some(f.title.as_str()) {
            let _ = writeln!(s, "fix: {}", f.title);
        }
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

/// The help of a rule whose diagnostics carry none of their own, so that
/// every error tells an agent what to write (gram.help.1, ADR 0262). A
/// diagnostic's own help always wins.
pub fn default_help(rule: &str) -> Option<&'static str> {
    Some(match rule {
        "lex.stmt.1" => "a statement is one line; continue a long one after a comma or an operator, or inside brackets",
        "lex.stmt.2" => "close the bracket on the same statement, or remove it",
        "lex.str.1" => "escapes are `\\n \\t \\r \\0 \\\\ \\\" \\' \\{ \\u{...}`; `{expression}` interpolates",
        "lex.num.3" => "write a number that fits, or compute it from smaller parts",
        "lex.block.1" | "lex.block.3" => "indent a block by exactly 4 spaces below the line that opens it",
        "gram.syntax.1" => "docs/language-card.md lists the declarations and statements AIL has",
        "gram.fn.1" | "gram.fn.2" => "a function is `fn name(a int, b int) int`, then an indented body",
        "expr.if.1" | "expr.loop.1" => "a header ends its line, and the block is the lines indented 4 spaces below it",
        "expr.call.1" => "give exactly the arguments the signature lists; `ail api FILE` shows each signature",
        "expr.call.2" => {
            "pass a value of the expected type: convert with `n.to_float()`, `x.to_int()`, or `\"{x}\"` for text"
        }
        "expr.block.2" => {
            "make the last expression the declared return type, change the return type, or add `return value`"
        }
        "expr.match.5" => "delete this arm, or move it below the arms it overlaps",
        "expr.match.7" => "write a pattern of the same type as the matched value",
        "expr.loop.5" => "`break` and `continue` belong inside a `for` or `while` loop",
        "expr.loop.6" => "`for mut x in xs` needs `xs` to be a `mut` list",
        "expr.comp.3" => "a comprehension's element is a value: `[x * 2 for x in xs]`",
        "trait.op.1" | "trait.op.3" => {
            "both operands need one type and an operator the type has: convert with `n.to_float()` or `x.to_int()`"
        }
        "type.coll.2" => "give every element the same type; convert the odd one, e.g. `n.to_float()`",
        "type.struct.4" | "type.struct.5" => {
            "give each field once, as `field: value`, with a value of the field's type"
        }
        "type.bool.1" => "`not`, `and` and `or` take `bool`; compare first, e.g. `n != 0`",
        "err.else.1" => "`else` follows a value that can be absent (`T?`) or fail (`T ! E`); remove it here",
        "err.else.2" => "end the `else` block with `return`, `break` or `continue`, or with a value",
        "err.prop.1" => "`?` follows a call that can fail or a value that can be absent; remove it here",
        "mod.main.3" | "std.io.3" => "print a value; a call to a function without a return type gives nothing",
        "mod.unit.1" => "names are unique within a module; rename one",
        "own.param.2" => "mark `mut` only a variable, or a field or element of one, and only for a `mut` parameter",
        "test.assert.2" => "`assert` takes a `bool`, e.g. `assert n == 3`",
        "contract.decl.2" => "a contract is a pure `bool` expression, e.g. `requires n >= 0`",
        "trait.impl.1" | "trait.impl.4" | "trait.std.2" => {
            "`impl Trait for Type` then each method; `ail api FILE` lists the traits"
        }
        "gen.infer.2" => "give as many type arguments as the type declares, in `[]`",
        "bootstrap.unsupported" => "this part of the spec isn't implemented in the bootstrap yet; see docs/status.md",
        _ => return None,
    })
}

#[cfg(test)]
mod anchor_tests {
    use super::*;

    /// Resolve `edit` against one line and apply it.
    fn apply(line: &str, mut edit: Edit) -> Option<String> {
        let lines = vec![line.chars().collect::<Vec<char>>()];
        if !edit.resolve(&lines) {
            return None;
        }
        let chars: Vec<char> = line.chars().collect();
        let (s, e) = (edit.pos.col - 1, edit.end.col - 1);
        Some(format!("{}{}{}", chars[..s].iter().collect::<String>(), edit.text, chars[e..].iter().collect::<String>()))
    }

    fn at(col: usize) -> Pos {
        Pos { line: 1, col }
    }

    #[test]
    fn call_anchor_rewrites_with_a_template() {
        let e = || Edit::call(at(11), "len", "$r1.len()");
        assert_eq!(apply("    print(len(xs))", e()).as_deref(), Some("    print(xs.len())"));
        // A receiver that isn't a plain postfix expression gets parentheses.
        let e = Edit::call(at(11), "abs", "$r1.abs()");
        assert_eq!(apply("    print(abs(a - b))", e).as_deref(), Some("    print((a - b).abs())"));
        let e = Edit::call(at(5), "max", "$r1.max($2)");
        assert_eq!(apply("    max(1, f(2, 3))", e).as_deref(), Some("    (1).max(f(2, 3))"));
        // Commas inside strings and brackets don't split arguments.
        let e = Edit::call(at(5), "print", "print($*)");
        assert_eq!(apply("    print(\"a, b\", [1, 2])", e).as_deref(), Some("    print(\"a, b {[1, 2]}\")"));
    }

    #[test]
    fn call_anchor_is_dropped_when_it_does_not_fit() {
        let e = Edit::call(at(5), "range", "$1..$2");
        assert_eq!(apply("    range(3)", e), None);
        let e = Edit::call(at(5), "len", "$r1.len()");
        assert_eq!(apply("    len(xs", e), None);
        let e = Edit::call(at(5), "str", "\"{$1}\"");
        assert_eq!(apply("    str(\"a\")", e), None);
    }

    #[test]
    fn named_and_chain_anchors() {
        let e = Edit::call_named(at(1), "Point", &["x", "y"]);
        assert_eq!(apply("Point(1, g(2, 3))", e).as_deref(), Some("Point(x: 1, y: g(2, 3))"));
        let e = Edit::chain(at(6), at(10));
        assert_eq!(apply("if 0 < n < 5", e).as_deref(), Some("if 0 < n and n < 5"));
        assert_eq!(apply("x := 1 ;", Edit::trim_before(at(8))).as_deref(), Some("x := 1"));
    }
}
