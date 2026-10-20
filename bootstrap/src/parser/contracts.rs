//! `requires` and `ensures` lines (contract.decl.1-3, ADR 0210).

use super::Parser;
use crate::ast::{Arg, Block, Contract, Expr, Program, Within};
use crate::diag::Pos;
use crate::lexer::{Part, Tok, Token};

impl Parser {
    /// A function body: an indented block whose first lines may be
    /// `requires COND` and `ensures COND` (contract.decl.1). A trait method
    /// may have contract lines and no body (contract.inh.1): `None`.
    pub(super) fn fn_body(&mut self, after: &str, bodyless_ok: bool) -> Option<(Option<Block>, Vec<Contract>)> {
        if !matches!(self.peek_at(1), Tok::Indent) {
            return Some((Some(self.block(after, "gram.fn.1")?), Vec::new()));
        }
        self.bump(); // Newline
        self.bump(); // Indent
        let mut contracts = Vec::new();
        loop {
            while self.eat(&Tok::Newline) {}
            let ensures = match self.peek() {
                Tok::Kw("requires") => false,
                Tok::Kw("ensures") => true,
                _ => break,
            };
            match self.contract_line(ensures) {
                Some(c) => contracts.push(c),
                None => self.skip_statement(),
            }
        }
        if bodyless_ok && !contracts.is_empty() && self.eat(&Tok::Dedent) {
            return Some((None, contracts));
        }
        Some((Some(self.block_rest()), contracts))
    }

    /// `requires COND`, `ensures COND` or `invariant COND`, up to the end of
    /// the line (not consumed, for `invariant`; consumed otherwise).
    fn contract_line(&mut self, ensures: bool) -> Option<Contract> {
        let pos = self.bump().pos;
        let start = self.i;
        let outer = self.olds.take();
        self.olds = ensures.then(Vec::new);
        let cond = self.expression();
        let olds = std::mem::replace(&mut self.olds, outer).unwrap_or_default();
        let cond = cond?;
        let text = render(&self.tokens[start..self.i]);
        if !matches!(self.peek(), Tok::Newline) {
            self.unexpected("end of line after the condition", "contract.decl.1");
            return None;
        }
        Some(Contract { ensures, cond, text, pos, olds })
    }

    /// A struct's `invariant COND` member (contract.inv.1); `members` eats
    /// the end of the line.
    pub(super) fn invariant_line(&mut self) -> Option<Contract> {
        self.contract_line(false)
    }

    /// contract.ens.3: `old(e)` inside an `ensures` condition names the value
    /// `e` had on entry. It becomes a hidden name that the interpreter binds
    /// before the body runs.
    pub(super) fn old_call(&mut self, callee: &Expr, mut args: Vec<Arg>, pos: Pos) -> Result<Expr, Vec<Arg>> {
        let Expr::Name(name, _) = callee else { return Err(args) };
        if name != "old" || self.olds.is_none() {
            return Err(args);
        }
        let plain = |a: &Arg| a.label.is_none() && !a.mutable && !a.moved && a.param.is_none();
        if args.len() != 1 || !plain(&args[0]) {
            self.error_help(
                "contract.ens.3",
                pos,
                "`old` takes one expression: the value to remember from when the function was called",
                "write `old(x)`",
            );
            return Ok(Expr::Name("old".into(), pos));
        }
        let value = args.remove(0).value;
        // The hidden name is the call as written, which can't be a user's
        // name and is what a trap's values list shows.
        let close = self.i - 1;
        let mut open = close;
        let mut depth = 0;
        while open > 0 {
            match self.tokens[open].tok {
                Tok::Sym(")") => depth += 1,
                Tok::Sym("(") => depth -= 1,
                _ => {}
            }
            if depth == 0 {
                break;
            }
            open -= 1;
        }
        let hidden = format!("old({})", render(&self.tokens[open + 1..close]));
        let olds = self.olds.as_mut().expect("checked above");
        olds.push((hidden.clone(), value));
        Ok(Expr::Name(hidden, pos))
    }

    /// contract.decl.1: a contract line anywhere but directly under a
    /// signature. Reports it and skips the line.
    pub(super) fn misplaced_contract(&mut self) -> bool {
        let Tok::Kw(word @ ("requires" | "ensures")) = *self.peek() else { return false };
        let pos = self.pos();
        self.error_help(
            "contract.decl.1",
            pos,
            format!("`{word}` must come directly under a function's signature, before its first statement"),
            format!("move the `{word}` line up to just below the `fn` line"),
        );
        self.skip_statement();
        true
    }
}

/// contract.inh.1: an implementation's method inherits the contracts of the
/// trait's method of the same name. Records the trait method's index when it
/// has any.
pub fn link(prog: &mut Program) {
    let mut links = Vec::new();
    for (i, f) in prog.fns.iter().enumerate() {
        let Within::Impl(k) = f.within else { continue };
        let Some(t) = prog.traits.iter().find(|t| t.name == prog.impls[k].trait_name) else { continue };
        let found = t.methods.iter().copied().find(|&m| prog.fns[m].name == f.name && !prog.fns[m].contracts.is_empty());
        if let Some(m) = found {
            links.push((i, m));
        }
    }
    for (i, m) in links {
        prog.fns[i].inherits = Some(m);
    }
}

/// The source form of a condition's tokens, with the formatter's spacing
/// (fmt.space): what a trap and `ail api` show (contract.req.1).
pub fn render(tokens: &[Token]) -> String {
    let mut out = String::new();
    let (mut before, mut prev): (Option<&Tok>, Option<&Tok>) = (None, None);
    for t in tokens {
        if matches!(t.tok, Tok::Newline | Tok::Indent | Tok::Dedent | Tok::Eof) {
            continue;
        }
        if let Some(p) = prev
            && spaced(before, p, &t.tok)
        {
            out.push(' ');
        }
        word(&t.tok, &mut out);
        before = prev;
        prev = Some(&t.tok);
    }
    out
}

/// Ends an operand, so a following `(` or `[` is a call or an index and a
/// following `-` is binary.
fn operand(t: &Tok) -> bool {
    match t {
        Tok::Kw(k) => matches!(*k, "self" | "true" | "false" | "none" | "todo"),
        Tok::Sym(s) => matches!(*s, ")" | "]" | "}" | "?"),
        _ => true,
    }
}

/// Whether a space separates `a` and `b`; `before` precedes `a` (fmt.space).
fn spaced(before: Option<&Tok>, a: &Tok, b: &Tok) -> bool {
    if matches!(b, Tok::Sym("," | ")" | "]" | "}" | "." | "?" | ":" | ".." | "..="))
        || matches!(a, Tok::Sym("(" | "[" | "{" | "." | ".." | "..="))
    {
        return false;
    }
    // A prefix `-` or `!`.
    if matches!(a, Tok::Sym("-" | "!")) && before.is_none_or(|t| !operand(t)) {
        return false;
    }
    if matches!(b, Tok::Sym("(" | "[")) {
        return !operand(a);
    }
    true
}

fn word(t: &Tok, out: &mut String) {
    match t {
        Tok::Ident(s) => out.push_str(s),
        Tok::Kw(s) | Tok::Sym(s) => out.push_str(s),
        Tok::Str(s) => {
            out.push('"');
            escape(s, out);
            out.push('"');
        }
        Tok::Interp(parts) => {
            out.push('"');
            for part in parts {
                match part {
                    Part::Lit(s) => escape(s, out),
                    Part::Code(toks) | Part::Fmt(toks, _) => {
                        out.push('{');
                        out.push_str(&render(toks));
                        out.push('}');
                    }
                }
            }
            out.push('"');
        }
        Tok::Char(c) => {
            out.push('\'');
            escape(&c.to_string(), out);
            out.push('\'');
        }
        Tok::Int(v) => out.push_str(&v.to_string()),
        Tok::IntMin => out.push_str("9223372036854775808"),
        Tok::Float(v) => out.push_str(&format!("{v:?}")),
        Tok::Newline | Tok::Indent | Tok::Dedent | Tok::Eof => {}
    }
}

fn escape(s: &str, out: &mut String) {
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '{' => out.push_str("\\{"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::lex;

    fn text(src: &str) -> String {
        super::render(&lex(src).0)
    }

    #[test]
    fn renders_conditions_in_canonical_spacing() {
        assert_eq!(text("b != 0"), "b != 0");
        assert_eq!(text("result * b + a % b == a"), "result * b + a % b == a");
        assert_eq!(text("xs.len() > 0 and xs[0]>=-1"), "xs.len() > 0 and xs[0] >= -1");
        assert_eq!(text("f(a,  -b) == not ok"), "f(a, -b) == not ok");
        assert_eq!(text("s == \"a\\\"b\""), "s == \"a\\\"b\"");
        assert_eq!(text("0..n"), "0..n");
    }
}
