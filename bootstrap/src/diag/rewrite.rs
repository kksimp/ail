//! Anchored edits that read the source line (fix.edit.2, ADR 0335): the
//! operands around an operator, a C-style `for` header and a typed empty
//! collection. Each is dropped, not guessed, when the line isn't a plain case.

use super::{Anchor, Edit, Pos};

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// The index just past the bracket that closes the one at `open`, if the
/// bracketed text has no string in it.
fn skip_brackets(line: &[char], open: usize) -> Option<usize> {
    let mut depth = 0i32;
    for (i, c) in line.iter().enumerate().skip(open) {
        match c {
            '"' | '\'' => return None,
            '(' | '[' => depth += 1,
            ')' | ']' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i + 1);
                }
            }
            _ => {}
        }
    }
    None
}

/// Start of the postfix expression (name, literal, call, index, `.` chain)
/// that ends just before `end`.
fn atom_before(line: &[char], end: usize) -> Option<usize> {
    let mut i = end;
    loop {
        match *line.get(i.checked_sub(1)?)? {
            '"' => i = line[..i - 1].iter().rposition(|c| *c == '"')?,
            ')' | ']' => {
                let mut j = i - 1;
                let mut depth = 0i32;
                loop {
                    match line[j] {
                        '"' | '\'' => return None,
                        ')' | ']' => depth += 1,
                        '(' | '[' => {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                        _ => {}
                    }
                    j = j.checked_sub(1)?;
                }
                i = j;
                while i > 0 && is_word(line[i - 1]) {
                    i -= 1;
                }
            }
            c if is_word(c) => {
                while i > 0 && is_word(line[i - 1]) {
                    i -= 1;
                }
            }
            _ => return None,
        }
        if i > 0 && line[i - 1] == '.' {
            i -= 1;
            continue;
        }
        return Some(i);
    }
}

/// End (exclusive) of the postfix expression that starts at `from`.
fn atom_after(line: &[char], from: usize) -> Option<usize> {
    let mut i = from;
    loop {
        match *line.get(i)? {
            '"' => {
                i += 1;
                while *line.get(i)? != '"' {
                    i += if line[i] == '\\' { 2 } else { 1 };
                }
                i += 1;
            }
            c if is_word(c) => {
                while i < line.len() && is_word(line[i]) {
                    i += 1;
                }
            }
            _ => return None,
        }
        while matches!(line.get(i), Some('(' | '[')) {
            i = skip_brackets(line, i)?;
        }
        if line.get(i) == Some(&'.') && line.get(i + 1).is_some_and(|c| is_word(*c)) {
            i += 1;
            continue;
        }
        return Some(i);
    }
}

/// The word that ends just before `end`, if any.
fn word_before(line: &[char], end: usize) -> String {
    let start = (0..end).rev().take_while(|k| is_word(line[*k])).last().unwrap_or(end);
    line[start..end].iter().collect()
}

/// Does the operand starting at `start` begin an expression of its own, so
/// that nothing binds tighter on its left?
fn starts_expression(line: &[char], start: usize) -> bool {
    let mut k = start;
    while k > 0 && line[k - 1] == ' ' {
        k -= 1;
    }
    k == 0
        || matches!(line[k - 1], '(' | ',' | '[' | '=')
        || matches!(word_before(line, k).as_str(), "if" | "while" | "elif" | "return" | "and" | "or")
}

/// Does the operand ending at `end` end an expression of its own?
fn ends_expression(line: &[char], end: usize) -> bool {
    let mut k = end;
    while line.get(k) == Some(&' ') {
        k += 1;
    }
    let Some(&c) = line.get(k) else { return true };
    if matches!(c, ')' | ']' | ',' | '#') {
        return true;
    }
    let word: String = line[k..].iter().take_while(|c| is_word(**c)).collect();
    matches!(word.as_str(), "and" | "or")
}

/// An operand as a piece of an interpolated string.
fn piece(s: &str) -> Option<String> {
    if let Some(inner) = s.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
        return (!inner.contains(['"', '{', '}'])).then(|| inner.to_string());
    }
    (!s.contains(['"', '{', '}'])).then(|| format!("{{{s}}}"))
}

impl Edit {
    /// Rewrite `l op r` around the operator `op` at `pos`; `text` is a
    /// template: `$l` `$r` are the operands as written, `$lp` `$rp` as pieces
    /// of one string, `$f` the left string with its one `%d` or `%s` replaced
    /// by the right operand (fix.edit.6).
    pub fn operands(pos: Pos, op: &str, template: impl Into<String>) -> Self {
        Edit { pos, end: pos, old: Some(op.into()), anchor: Anchor::Operands, text: template.into() }
    }

    /// `for (i := a; i < b; i += 1)` as `for i in a..b`, the header being
    /// the parenthesis at `pos` through the end of the line.
    pub fn c_for(pos: Pos) -> Self {
        Edit { pos, end: pos, old: None, anchor: Anchor::CFor, text: String::new() }
    }

    /// `xs: List[int] = []` as `xs := List[int]()`, from the statement start.
    pub fn empty_decl(pos: Pos) -> Self {
        Edit { pos, end: pos, old: None, anchor: Anchor::EmptyDecl, text: String::new() }
    }

    pub(super) fn resolve_operands(&mut self, lines: &[Vec<char>]) -> bool {
        let Some(op) = self.old.take() else { return false };
        let Some(line) = lines.get(self.pos.line.wrapping_sub(1)) else { return false };
        let at = self.pos.col.saturating_sub(1);
        let opc: Vec<char> = op.chars().collect();
        if !line.get(at..).is_some_and(|s| s.starts_with(&opc)) {
            return false;
        }
        let mut le = at;
        while le > 0 && line[le - 1] == ' ' {
            le -= 1;
        }
        let Some(ls) = atom_before(line, le) else { return false };
        let mut rs = at + opc.len();
        while line.get(rs) == Some(&' ') {
            rs += 1;
        }
        let Some(re) = atom_after(line, rs) else { return false };
        if !starts_expression(line, ls) || !ends_expression(line, re) {
            return false;
        }
        let l: String = line[ls..le].iter().collect();
        let r: String = line[rs..re].iter().collect();
        let t = std::mem::take(&mut self.text);
        let mut out = String::new();
        let mut it = t.chars().peekable();
        while let Some(c) = it.next() {
            if c != '$' {
                out.push(c);
                continue;
            }
            match it.next() {
                Some('l') if it.next_if_eq(&'p').is_some() => match piece(&l) {
                    Some(p) => out.push_str(&p),
                    None => return false,
                },
                Some('r') if it.next_if_eq(&'p').is_some() => match piece(&r) {
                    Some(p) => out.push_str(&p),
                    None => return false,
                },
                Some('l') => out.push_str(&l),
                Some('r') => out.push_str(&r),
                Some('f') => {
                    let Some(lit) = l.strip_prefix('"').and_then(|s| s.strip_suffix('"')) else { return false };
                    if lit.matches('%').count() != 1 || lit.contains(['{', '}']) {
                        return false;
                    }
                    let Some(arg) = piece(&r) else { return false };
                    let filled = ["%d", "%s"].iter().find_map(|s| lit.contains(s).then(|| lit.replacen(s, &arg, 1)));
                    let Some(filled) = filled else { return false };
                    out.push_str(&format!("\"{filled}\""));
                }
                _ => return false,
            }
        }
        self.pos.col = ls + 1;
        self.end = Pos { line: self.pos.line, col: re + 1 };
        self.text = out;
        true
    }

    pub(super) fn resolve_c_for(&mut self, lines: &[Vec<char>]) -> bool {
        let Some(line) = lines.get(self.pos.line.wrapping_sub(1)) else { return false };
        let open = self.pos.col.saturating_sub(1);
        let Some(close) = line.iter().rposition(|c| !c.is_whitespace()) else { return false };
        if line.get(open) != Some(&'(') || line[close] != ')' || close <= open {
            return false;
        }
        let inner: String = line[open + 1..close].iter().collect();
        let parts: Vec<&str> = inner.split(';').map(str::trim).collect();
        let [init, cond, step] = parts.as_slice() else { return false };
        let Some((var, from)) = init.split_once(":=").or_else(|| init.split_once('=')) else { return false };
        let (var, from) = (var.trim(), from.trim());
        if var.is_empty() || !var.chars().all(is_word) || from.is_empty() {
            return false;
        }
        let Some(rest) = cond.strip_prefix(var).map(str::trim_start) else { return false };
        let (dots, to) = match (rest.strip_prefix("<="), rest.strip_prefix('<')) {
            (Some(b), _) => ("..=", b.trim()),
            (_, Some(b)) => ("..", b.trim()),
            _ => return false,
        };
        let step: String = step.split_whitespace().collect();
        if to.is_empty() || ![format!("{var}++"), format!("{var}+=1"), format!("{var}={var}+1")].contains(&step) {
            return false;
        }
        self.pos.col = open + 1;
        self.end = Pos { line: self.pos.line, col: close + 2 };
        self.text = format!("{var} in {from}{dots}{to}");
        true
    }

    pub(super) fn resolve_empty_decl(&mut self, lines: &[Vec<char>]) -> bool {
        let Some(line) = lines.get(self.pos.line.wrapping_sub(1)) else { return false };
        let start = self.pos.col.saturating_sub(1);
        let text: String = line.get(start..).unwrap_or(&[]).iter().collect();
        if text.contains('#') {
            return false;
        }
        let Some((name, rest)) = text.split_once(':') else { return false };
        let Some((ty, value)) = rest.split_once('=') else { return false };
        let (name, ty, value) = (name.trim(), ty.trim(), value.trim());
        let known = ["List[", "Map[", "Set["].iter().any(|p| ty.starts_with(p)) && ty.ends_with(']');
        if name.is_empty() || !name.chars().all(is_word) || !known || !matches!(value, "[]" | "{}") {
            return false;
        }
        self.end = Pos { line: self.pos.line, col: start + text.trim_end().chars().count() + 1 };
        self.text = format!("{name} := {ty}()");
        true
    }
}

/// The arguments of a `printf(format, args...)` call as one interpolated
/// string: `%d` `%i` `%s` `%f` take the next argument, `%.Nf` a precision,
/// `%%` is `%`, a final `\n` is dropped because `print` ends the line. `None`
/// when the format isn't a plain literal or the argument count doesn't fit.
pub(super) fn printf_text(args: &[String]) -> Option<String> {
    let lit = args.first()?.strip_prefix('"')?.strip_suffix('"')?;
    if lit.contains(['"', '{', '}']) {
        return None;
    }
    let lit = lit.strip_suffix("\\n").unwrap_or(lit);
    let mut rest = args[1..].iter();
    let mut out = String::from("\"");
    let mut it = lit.chars().peekable();
    while let Some(c) = it.next() {
        if c != '%' {
            out.push(c);
            continue;
        }
        let mut spec = String::new();
        while let Some(d) = it.next_if(|d| d.is_ascii_digit() || *d == '.') {
            spec.push(d);
        }
        match it.next()? {
            '%' if spec.is_empty() => out.push('%'),
            'd' | 'i' | 's' if spec.is_empty() => out.push_str(&piece(rest.next()?)?),
            'f' if spec.is_empty() || spec.starts_with('.') => {
                let arg = rest.next()?;
                if arg.contains(['"', '{', '}']) {
                    return None;
                }
                out.push_str(&format!("{{{arg}{}}}", if spec.is_empty() { String::new() } else { format!(":{spec}") }));
            }
            _ => return None,
        }
    }
    if rest.next().is_some() {
        return None;
    }
    out.push('"');
    Some(out)
}
