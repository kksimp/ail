//! `std.regex` (spec std-regex.md, ADRs 0438-0441): a linear-time regular
//! expression engine. Patterns are the RE2 subset of std.regex.3; matching
//! is a Pike VM over characters, so no pattern backtracks (std.regex.4).
//! Offsets are character offsets (std.regex.5).

mod parse;
mod props;
#[cfg(test)]
mod vectors;
mod vm;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub use vm::Prog;

/// A pattern that doesn't compile: where, which kind, and what to do.
pub struct CompileError {
    pub at: usize,
    pub kind: &'static str,
    pub message: String,
}

/// The spec rule that explains an error kind (std.regex.11-21).
pub fn kind_rule(kind: &str) -> &'static str {
    match kind {
        "unclosed_group" => "std.regex.11",
        "unopened_group" => "std.regex.12",
        "unclosed_class" => "std.regex.13",
        "bad_escape" => "std.regex.14",
        "bad_repeat" => "std.regex.15",
        "nothing_to_repeat" => "std.regex.16",
        "bad_range" => "std.regex.17",
        "bad_class" => "std.regex.18",
        "bad_group_name" => "std.regex.19",
        "bad_flag" => "std.regex.20",
        "unsupported" => "std.regex.21",
        _ => "std.regex.22",
    }
}

thread_local! {
    /// Compiled patterns by source text (std.regex.2); cleared when it grows.
    static CACHE: RefCell<HashMap<String, Rc<Prog>>> = RefCell::new(HashMap::new());
}

/// Compile `pattern` (std.regex.2), or return the cached program.
pub fn compile(pattern: &str) -> Result<Rc<Prog>, CompileError> {
    if let Some(p) = CACHE.with(|c| c.borrow().get(pattern).cloned()) {
        return Ok(p);
    }
    let parsed = parse::parse(pattern).map_err(|e| CompileError { at: e.at, kind: e.kind, message: e.message })?;
    let prog = vm::compile(&parsed).map_err(|_| CompileError {
        at: 0,
        kind: "too_large",
        message: "the pattern compiles to more than 50000 instructions; use smaller repetition counts".into(),
    })?;
    let prog = Rc::new(prog);
    CACHE.with(|c| {
        let mut c = c.borrow_mut();
        if c.len() >= 256 {
            c.clear();
        }
        c.insert(pattern.to_string(), prog.clone());
    });
    Ok(prog)
}

/// A match: character offsets `[start, end)` per group (std.regex.5).
pub type Caps = Vec<usize>;

pub const UNSET: usize = usize::MAX;

impl Prog {
    pub fn is_match(&self, text: &[char]) -> bool {
        self.exec(text, 0, 0, true).is_some()
    }

    /// Every match, left to right, none overlapping (std.regex.6). `slots`
    /// is 2 for the whole match, or `2 * groups` for the groups too.
    pub fn find_all(&self, text: &[char], slots: usize, limit: usize) -> Vec<Caps> {
        let mut out = Vec::new();
        let mut pos = 0;
        let mut prev_end = None;
        while pos <= text.len() && out.len() < limit {
            let Some(m) = self.exec(text, pos, slots, false) else { break };
            let (s, e) = (m[0], m[1]);
            if s == e && prev_end == Some(e) {
                // An empty match right after the previous match isn't one.
                pos = e + 1;
                continue;
            }
            pos = if e > s { e } else { e + 1 };
            prev_end = Some(e);
            out.push(m);
        }
        out
    }
}

/// The pieces of a replacement template (std.regex.7).
pub enum Part {
    Lit(String),
    Group(usize),
}

/// Parse a replacement template against `prog`'s groups. `Err` names the
/// reference that doesn't exist.
pub fn template(rep: &str, prog: &Prog) -> Result<Vec<Part>, String> {
    let cs: Vec<char> = rep.chars().collect();
    let mut parts = Vec::new();
    let mut lit = String::new();
    let mut i = 0;
    let group_named = |name: &str| -> Result<usize, String> {
        if !name.is_empty() && name.chars().all(|c| c.is_ascii_digit()) {
            return match name.parse::<usize>() {
                Ok(n) if n < prog.groups => Ok(n),
                _ => Err(format!("group {name}; the pattern has groups 0 to {}", prog.groups - 1)),
            };
        }
        match prog.names.iter().find(|(n, _)| n == name) {
            Some((_, i)) => Ok(*i),
            None => {
                let have: Vec<&str> = prog.names.iter().map(|(n, _)| n.as_str()).collect();
                Err(format!("group `{name}`; the pattern's named groups are {have:?}"))
            }
        }
    };
    while i < cs.len() {
        if cs[i] != '$' {
            lit.push(cs[i]);
            i += 1;
            continue;
        }
        match cs.get(i + 1) {
            Some('$') => {
                lit.push('$');
                i += 2;
            }
            Some('{') => {
                let Some(end) = cs[i + 2..].iter().position(|&c| c == '}') else {
                    return Err("`${` without a closing `}`".into());
                };
                let name: String = cs[i + 2..i + 2 + end].iter().collect();
                let g = group_named(&name)?;
                parts.push(Part::Lit(std::mem::take(&mut lit)));
                parts.push(Part::Group(g));
                i += end + 3;
            }
            Some(c) if c.is_ascii_alphanumeric() || *c == '_' => {
                let mut j = i + 1;
                while j < cs.len() && (cs[j].is_ascii_alphanumeric() || cs[j] == '_') {
                    j += 1;
                }
                let name: String = cs[i + 1..j].iter().collect();
                let g = group_named(&name)?;
                parts.push(Part::Lit(std::mem::take(&mut lit)));
                parts.push(Part::Group(g));
                i = j;
            }
            _ => {
                lit.push('$');
                i += 1;
            }
        }
    }
    parts.push(Part::Lit(lit));
    Ok(parts)
}

/// `text[from..to]` as a string.
pub fn slice(text: &[char], from: usize, to: usize) -> String {
    text[from..to].iter().collect()
}

/// Replace the first `limit` matches (std.regex.7).
pub fn replace(prog: &Prog, text: &[char], parts: &[Part], limit: usize) -> String {
    let mut out = String::new();
    let mut last = 0;
    for m in prog.find_all(text, 2 * prog.groups, limit) {
        out.push_str(&slice(text, last, m[0]));
        for p in parts {
            match p {
                Part::Lit(s) => out.push_str(s),
                Part::Group(g) => {
                    if m[2 * g] != UNSET {
                        out.push_str(&slice(text, m[2 * g], m[2 * g + 1]));
                    }
                }
            }
        }
        last = m[1];
    }
    out.push_str(&slice(text, last, text.len()));
    out
}

/// The text between matches (std.regex.8).
pub fn split(prog: &Prog, text: &[char]) -> Vec<String> {
    let mut out = Vec::new();
    let (mut beg, mut end) = (0, 0);
    for m in prog.find_all(text, 2, usize::MAX) {
        end = m[0];
        if m[1] != 0 {
            out.push(slice(text, beg, end));
        }
        beg = m[1];
    }
    if end != text.len() || out.is_empty() {
        out.push(slice(text, beg, text.len()));
    }
    out
}

/// `s` with every character the pattern syntax treats specially escaped
/// (std.regex.1).
pub fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if "\\.+*?()|[]{}^$-".contains(c) {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn find(p: &str, t: &str) -> Option<(usize, usize)> {
        let prog = compile(p).ok()?;
        let cs: Vec<char> = t.chars().collect();
        prog.exec(&cs, 0, 2, false).map(|m| (m[0], m[1]))
    }

    #[test]
    fn leftmost_first() {
        assert_eq!(find("a|ab", "ab"), Some((0, 1)));
        assert_eq!(find("a+?", "aaa"), Some((0, 1)));
        assert_eq!(find("(a+)(b+)?", "xaab"), Some((1, 4)));
    }

    #[test]
    fn linear_on_the_classic_blowup() {
        let text = "a".repeat(5000);
        assert!(find("(a*)*b", &text).is_none());
        assert!(find("(a|aa)+$", &(text + "b")).is_none());
    }
}
