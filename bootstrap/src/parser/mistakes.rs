//! Recovery from constructs borrowed from other languages (gram.foreign.1,
//! gram.rec.1, ADRs 0262-0263). Each function recognises one family of
//! mistakes at a place the parser has already failed, reports it once with
//! the AIL spelling, and, where the edit is mechanical, with a fix.

use super::Parser;
use crate::diag::{Diagnostic, Edit, Pos};
use crate::foreign::{self, Fix};
use crate::lexer::{Tok, Token};

impl Parser {
    /// Report `msg` at `pos` with `help` and one fix.
    pub(super) fn error_fix(
        &mut self,
        rule: &'static str,
        pos: Pos,
        msg: impl Into<String>,
        help: impl Into<String>,
        title: impl Into<String>,
        edits: Vec<Edit>,
    ) {
        let msg = msg.into();
        // The same mistake again on one line is one diagnostic with one fix
        // (gram.rec.1): its edits join the first report's.
        if let Some(last) = self.diags.last_mut()
            && last.rule == rule
            && last.pos.line == pos.line
            && last.message == msg
            && let Some(fix) = last.fixes.first_mut()
        {
            fix.edits.extend(edits);
            return;
        }
        self.diags.push(Diagnostic::new(rule, pos, msg).help(help).fix(title, edits));
    }

    fn tok_pos(&self, i: usize) -> Pos {
        self.tokens[i.min(self.tokens.len() - 1)].pos
    }

    /// The position just past token `i`, when it is a name, keyword or
    /// symbol (whose width is known).
    pub(super) fn tok_end(&self, i: usize) -> Option<Pos> {
        let t: &Token = self.tokens.get(i)?;
        let len = match &t.tok {
            Tok::Ident(n) => n.chars().count(),
            Tok::Kw(k) => k.len(),
            Tok::Sym(s) => s.len(),
            _ => return None,
        };
        Some(Pos { line: t.pos.line, col: t.pos.col + len })
    }

    /// The start of the token after `i` when it is on the same line, else the
    /// end of `i`: a span that removes `i` and the blank after it.
    fn through_blank(&self, i: usize) -> Option<Pos> {
        let next = self.tokens.get(i + 1)?;
        if next.pos.line == self.tokens[i].pos.line && !matches!(next.tok, Tok::Newline | Tok::Eof) {
            Some(next.pos)
        } else {
            self.tok_end(i)
        }
    }

    /// Whether token `i + 1` follows token `i` with no blank between.
    pub(super) fn adjacent(&self, i: usize) -> bool {
        match (self.tokens.get(i), self.tokens.get(i + 1)) {
            (Some(a), Some(b)) => a.pos.line == b.pos.line && b.pos.col == a.pos.col + 1,
            _ => false,
        }
    }

    /// The index of the closing bracket matching the opening one at `open`.
    fn matching(&self, open: usize) -> Option<usize> {
        let mut depth = 0usize;
        for (k, t) in self.tokens.iter().enumerate().skip(open) {
            match &t.tok {
                Tok::Sym("(" | "[" | "{") => depth += 1,
                Tok::Sym(")" | "]" | "}") => {
                    depth = depth.checked_sub(1)?;
                    if depth == 0 {
                        return Some(k);
                    }
                }
                Tok::Eof => return None,
                _ => {}
            }
        }
        None
    }

    /// A top-level word from another language's declarations (`def`,
    /// `class`, `import`, `async`, ...). Reports it and skips the line.
    pub(super) fn foreign_decl(&mut self, word: &str) -> bool {
        let at = self.i;
        if matches!(self.peek_at(1), Tok::Sym(":=" | "=" | "(" | "." | "[")) {
            return false;
        }
        // `from a.b import c` becomes `use a.b.c`.
        if word == "from" {
            return self.from_import();
        }
        let Some(info) = foreign::decl_word(word) else { return false };
        let pos = self.tok_pos(at);
        let d = Diagnostic::new("gram.foreign.1", pos, info.msg).help(info.help);
        let d = match info.fix {
            Fix::None => d,
            Fix::Replace(new) => {
                d.fix(format!("write `{new}`"), vec![Edit::span(pos, self.tok_end(at).unwrap_or(pos), new)])
            }
            Fix::Delete => {
                let end = self.through_blank(at).unwrap_or(pos);
                d.fix(format!("remove `{word}`"), vec![Edit::delete(pos, end)])
            }
        };
        self.diags.push(d);
        // A modifier is followed by the declaration itself, which the next
        // round (or the same parse, for `fn`) handles.
        if matches!(info.fix, Fix::Delete | Fix::Replace("pub"))
            && matches!(self.peek_at(1), Tok::Kw(_) | Tok::Ident(_))
        {
            self.i += 1;
            return true;
        }
        self.skip_statement();
        true
    }

    /// `from a.b import c`: Python's import.
    fn from_import(&mut self) -> bool {
        let at = self.i;
        let mut k = at + 1;
        let mut path = Vec::new();
        loop {
            let Tok::Ident(n) = &self.tokens[k].tok else { return false };
            path.push(n.clone());
            k += 1;
            if matches!(self.tokens[k].tok, Tok::Sym(".")) {
                k += 1;
            } else {
                break;
            }
        }
        if !matches!(&self.tokens[k].tok, Tok::Ident(w) if w == "import") {
            return false;
        }
        let pos = self.tok_pos(at);
        let mut d = Diagnostic::new("gram.foreign.1", pos, "AIL has no `from ... import`").help(format!(
            "import the module with `use {}` and write `{}.name` for its items",
            path.join("."),
            path.last().unwrap()
        ));
        if let (Tok::Ident(item), Some(end)) = (&self.tokens[k + 1].tok, self.tok_end(k + 1))
            && matches!(self.tokens[k + 2].tok, Tok::Newline | Tok::Eof)
        {
            d = d.fix("write a `use`", vec![Edit::span(pos, end, format!("use {}.{item}", path.join(".")))]);
        }
        self.diags.push(d);
        self.skip_statement();
        true
    }

    /// A statement word from another language (`try`, `switch`, `pass`,
    /// `throw`, ...), unless the word is used as a name.
    pub(super) fn foreign_stmt(&mut self) -> bool {
        let Tok::Ident(word) = self.peek().clone() else { return false };
        if !matches!(
            self.peek_at(1),
            Tok::Newline | Tok::Ident(_) | Tok::Str(_) | Tok::Interp(_) | Tok::Int(_) | Tok::Kw(_)
        ) {
            return false;
        }
        let Some(info) = foreign::stmt_word(&word) else { return false };
        let at = self.i;
        let pos = self.tok_pos(at);
        let d = Diagnostic::new("gram.foreign.1", pos, info.msg).help(info.help);
        let d = match info.fix {
            Fix::Replace(new) => {
                d.fix(format!("write `{new}`"), vec![Edit::span(pos, self.tok_end(at).unwrap_or(pos), new)])
            }
            _ => d,
        };
        self.diags.push(d);
        self.skip_statement();
        true
    }

    /// After a block header the line didn't end where it should (`if c:`,
    /// `else:`, `if a = b`, `if x is none`, `if a && b {`). Returns true when
    /// the mistake was reported; for a colon the parser goes on, as the
    /// mistake is only the colon.
    pub(super) fn header_tail(&mut self, after: &str) -> Header {
        let at = self.i;
        let pos = self.tok_pos(at);
        let cond = after.contains("condition");
        match self.peek().clone() {
            Tok::Sym(":") if !cond || matches!(self.peek_at(1), Tok::Newline) => {
                // `if c:` ends the line; `fn f() : int` goes on to the type.
                let colon_only = matches!(self.peek_at(1), Tok::Newline);
                let end = match self.through_blank(at) {
                    Some(e) if !colon_only && e.line == pos.line => e,
                    _ => Pos { col: pos.col + 1, ..pos },
                };
                self.error_fix(
                    "gram.foreign.1",
                    pos,
                    "AIL doesn't put a `:` after a block header",
                    "a line that opens a block ends where the header ends; the block is the indented lines below",
                    "remove the `:`",
                    vec![Edit::delete(pos, end)],
                );
                self.i += 1;
                if colon_only {
                    self.i += 1; // the Newline
                    Header::Continue
                } else {
                    Header::Reported
                }
            }
            Tok::Sym("=") if cond => {
                self.error_fix(
                    "expr.bind.3",
                    pos,
                    "`=` assigns; it can't be a condition",
                    "compare with `==`",
                    "write `==`",
                    vec![Edit::span(pos, Pos { col: pos.col + 1, ..pos }, "==")],
                );
                Header::Reported
            }
            Tok::Ident(w) if cond && w == "is" => {
                let not = matches!(self.peek_at(1), Tok::Kw("not"));
                let end = if not { self.tok_end(at + 1) } else { self.tok_end(at) };
                let (op, text) = if not { ("!=", "is not") } else { ("==", "is") };
                self.error_fix(
                    "gram.foreign.1",
                    pos,
                    format!("AIL has no `{text}`"),
                    format!("compare with `{op}`, e.g. `x {op} none`"),
                    format!("write `{op}`"),
                    vec![Edit::span(pos, end.unwrap_or(pos), op)],
                );
                Header::Reported
            }
            Tok::Kw("in") if cond => {
                self.error("gram.foreign.1", pos, "AIL has no `in` operator for membership");
                if let Some(d) = self.diags.last_mut() {
                    d.help = Some("write the test as a method on the collection: `xs.contains(x)`, `m.contains(key)`, `s.contains(sub)`".into());
                }
                Header::Reported
            }
            Tok::Sym("{") if matches!(self.peek_at(1), Tok::Newline) || self.line_ends_at(at) => self.braces(at),
            _ => Header::No,
        }
    }

    /// Whether the token at `i` is the last one on its line.
    pub(super) fn line_ends_at(&self, i: usize) -> bool {
        self.tokens.get(i + 1).is_none_or(|n| n.pos.line > self.tokens[i].pos.line || matches!(n.tok, Tok::Eof))
    }

    /// Braces around blocks (`fn f() {`, `if c {`, `} else {`): one report
    /// for the file, with a fix that removes every brace pair.
    fn braces(&mut self, open: usize) -> Header {
        let pos = self.tok_pos(open);
        let mut edits = Vec::new();
        let mut ok = true;
        for k in open..self.tokens.len() {
            if !matches!(self.tokens[k].tok, Tok::Sym("{")) || !self.line_ends_at(k) || !self.opens_block(k) {
                continue;
            }
            edits.push(Edit::trim_before(self.tokens[k].pos));
            let Some(close) = self.matching(k) else {
                ok = false;
                break;
            };
            let cp = self.tokens[close].pos;
            let alone = close > 0 && self.tokens[close - 1].pos.line < cp.line;
            let next = self.tokens.get(close + 1);
            match next {
                Some(n) if n.pos.line == cp.line && matches!(n.tok, Tok::Kw("else" | "elif")) && alone => {
                    // `}` then `else` on one line: drop the `}`.
                    edits.push(Edit::delete(cp, n.pos));
                }
                Some(n) if n.pos.line == cp.line && !matches!(n.tok, Tok::Newline | Tok::Eof) => ok = false,
                _ if alone => {
                    edits.push(Edit::delete(Pos { line: cp.line, col: 1 }, Pos { line: cp.line + 1, col: 1 }))
                }
                _ => ok = false,
            }
        }
        let mut d = Diagnostic::new("gram.foreign.1", pos, "AIL doesn't use braces around blocks")
            .help("a block is the lines indented 4 spaces below its header, with no closing line");
        if ok && !edits.is_empty() {
            d = d.fix("remove the braces", edits);
        }
        self.diags.push(d);
        Header::Reported
    }

    /// Whether the `{` at `k` ends a block header: its line starts with a
    /// keyword that opens a block, or with the `}` of the previous branch.
    fn opens_block(&self, k: usize) -> bool {
        let line = self.tokens[k].pos.line;
        let first = self.tokens.iter().find(|t| t.pos.line == line);
        matches!(
            first.map(|t| &t.tok),
            Some(Tok::Kw(
                "fn" | "if"
                    | "elif"
                    | "else"
                    | "while"
                    | "for"
                    | "match"
                    | "struct"
                    | "enum"
                    | "impl"
                    | "trait"
                    | "error"
                    | "pub"
            )) | Some(Tok::Sym("}" | "@"))
        )
    }

    /// The generic "expected end of line after the statement" at a token that
    /// betrays another language. `start` is the index of the statement's
    /// first token. Returns true when it reported something specific.
    pub(super) fn stray_after_statement(&mut self, start: usize) -> bool {
        let at = self.i;
        let pos = self.tok_pos(at);
        let cur = self.peek().clone();
        let first = self.tokens[start].tok.clone();
        // `x: int = 5`
        if let (Tok::Ident(name), Tok::Sym(":")) = (&first, &cur)
            && at == start + 1
        {
            let eq = (at..self.tokens.len())
                .take_while(|k| !matches!(self.tokens[*k].tok, Tok::Newline | Tok::Eof))
                .find(|k| matches!(self.tokens[*k].tok, Tok::Sym("=")));
            let empty = eq.is_some_and(|e| {
                matches!(self.tokens.get(e + 1).map(|t| &t.tok), Some(Tok::Sym("[" | "{")))
                    && matches!(self.tokens.get(e + 2).map(|t| &t.tok), Some(Tok::Sym("]" | "}")))
            });
            let mut d = Diagnostic::new("expr.bind.1", self.tok_pos(start), format!("`{name}` is declared without a type annotation"))
                .help(format!("write `{name} := value`; the type is inferred. An empty collection names its type in the value: `List[int]()`"));
            if let (Some(e), false) = (eq, empty) {
                d = d.fix(
                    "declare it with `:=`",
                    vec![Edit::span(pos, Pos { col: self.tokens[e].pos.col + 1, ..self.tokens[e].pos }, " :=")],
                );
            }
            self.diags.push(d);
            return true;
        }
        // `Color::Red`, `std::fs`
        if matches!(cur, Tok::Sym(":")) && matches!(self.peek_at(1), Tok::Sym(":")) && self.adjacent(at) {
            self.error_fix(
                "gram.foreign.1",
                pos,
                "AIL writes paths with `.`, not `::`",
                "write `Color.Red`, `std.fs`",
                "write `.`",
                vec![Edit::span(pos, Pos { col: pos.col + 2, ..pos }, ".")],
            );
            return true;
        }
        // `new Point(...)`
        if at >= 1 && matches!(&self.tokens[at - 1].tok, Tok::Ident(n) if n == "new") && matches!(cur, Tok::Ident(_)) {
            let np = self.tok_pos(at - 1);
            self.error_fix(
                "gram.foreign.1",
                np,
                "AIL has no `new`",
                "construct a struct by calling its name: `Point(x: 1, y: 2)`",
                "remove `new`",
                vec![Edit::delete(np, pos)],
            );
            return true;
        }
        // `n > 0 ? a : b`
        if at >= 1 && matches!(self.tokens[at - 1].tok, Tok::Sym("?")) && !matches!(cur, Tok::Newline) {
            self.error("gram.foreign.1", self.tok_pos(at - 1), "AIL has no `?:` conditional operator");
            if let Some(d) = self.diags.last_mut() {
                d.help = Some(
                    "use an `if` expression: `x := if n > 0` with the value indented below, then `else` and its value"
                        .into(),
                );
            }
            return true;
        }
        // `(x) => x + 1`
        if matches!(cur, Tok::Sym("=>")) {
            self.error_help(
                "gram.foreign.1",
                pos,
                "AIL has no anonymous functions yet",
                "declare a named `fn` and pass its name",
            );
            return true;
        }
        // `Point{x: 1}`
        if matches!(cur, Tok::Sym("{"))
            && at >= 1
            && matches!(&self.tokens[at - 1].tok, Tok::Ident(n) if n.chars().next().is_some_and(char::is_uppercase))
        {
            let mut d = Diagnostic::new("gram.foreign.1", pos, "AIL builds a struct with parentheses, not braces")
                .help("write `Point(x: 1, y: 2)`: the name, then `field: value` pairs in parentheses");
            if let Some(close) = self.matching(at) {
                let cp = self.tokens[close].pos;
                d = d.fix(
                    "use parentheses",
                    vec![
                        Edit::span(pos, Pos { col: pos.col + 1, ..pos }, "("),
                        Edit::span(cp, Pos { col: cp.col + 1, ..cp }, ")"),
                    ],
                );
            }
            self.diags.push(d);
            return true;
        }
        false
    }
}

/// How `header_tail` left the parser.
pub(super) enum Header {
    /// Not recognised: report the generic error.
    No,
    /// Reported; stop parsing this statement.
    Reported,
    /// Reported, and the line's end was consumed: the block follows.
    Continue,
}
