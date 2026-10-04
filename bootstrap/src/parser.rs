//! Parser: tokens to a syntax tree for the subset the bootstrap supports.
//!
//! Supported so far: functions (gram.fn.1-2), immutable bindings (expr.bind.1),
//! calls, string and integer literals, and names. Other constructs report
//! `bootstrap.unsupported` with the spec rule they belong to.

use crate::ast::{Expr, FnDecl, Param, Program, Stmt, TypeRef};
use crate::diag::{Diagnostic, Pos};
use crate::lexer::{Tok, Token, UNSUPPORTED};

/// Keywords the spec defines that the bootstrap doesn't implement yet,
/// with the spec rule or chapter each belongs to.
const NOT_YET: &[(&str, &str)] = &[
    ("if", "expr.if.1"),
    ("elif", "expr.if.1"),
    ("else", "expr.if.1"),
    ("for", "expr.loop.1"),
    ("while", "expr.loop.1"),
    ("break", "expr.loop.2"),
    ("continue", "expr.loop.2"),
    ("match", "expr.match.1"),
    ("return", "expr.block.2"),
    ("mut", "expr.bind.2"),
    ("pub", "mod.vis.1"),
    ("use", "mod.use.1"),
    ("struct", "type.struct.1"),
    ("enum", "type.enum.1"),
    ("trait", "trait.decl.1"),
    ("impl", "trait.impl.1"),
    ("unique", "own.uniq.1"),
    ("move", "own.move.1"),
    ("ref", "own.ref.1"),
    ("defer", "own.defer.1"),
    ("spawn", "conc.task.1"),
    ("unsafe", "unsafe.block.1"),
    ("extern", "ffi.decl.2"),
    ("and", "expr.op.1"),
    ("or", "expr.op.1"),
    ("not", "expr.op.1"),
    ("none", "type.opt.4"),
];

pub fn parse(tokens: Vec<Token>) -> (Program, Vec<Diagnostic>) {
    let mut p = Parser { tokens, i: 0, diags: Vec::new() };
    let mut fns = Vec::new();
    loop {
        match p.peek().clone() {
            Tok::Eof => break,
            Tok::Newline => p.i += 1,
            Tok::Fn => {
                if let Some(f) = p.function() {
                    fns.push(f);
                }
            }
            _ => {
                p.unexpected("a top-level declaration (`fn ...`)", "gram.fn.1");
                p.skip_statement();
            }
        }
    }
    (Program { fns }, p.diags)
}

struct Parser {
    tokens: Vec<Token>,
    i: usize,
    diags: Vec<Diagnostic>,
}

impl Parser {
    fn peek(&self) -> &Tok {
        &self.tokens[self.i].tok
    }

    fn pos(&self) -> Pos {
        self.tokens[self.i].pos
    }

    fn bump(&mut self) -> Token {
        let t = self.tokens[self.i].clone();
        if !matches!(t.tok, Tok::Eof) {
            self.i += 1;
        }
        t
    }

    fn eat(&mut self, tok: &Tok) -> bool {
        if self.peek() == tok {
            self.i += 1;
            true
        } else {
            false
        }
    }

    fn describe(tok: &Tok) -> String {
        match tok {
            Tok::Ident(n) => format!("`{n}`"),
            Tok::Fn => "`fn`".into(),
            Tok::Str(_) => "a string".into(),
            Tok::Int(_) => "an integer".into(),
            Tok::LParen => "`(`".into(),
            Tok::RParen => "`)`".into(),
            Tok::Comma => "`,`".into(),
            Tok::ColonEq => "`:=`".into(),
            Tok::Colon => "`:`".into(),
            Tok::Arrow => "`->`".into(),
            Tok::Newline => "end of line".into(),
            Tok::Indent => "an indented block".into(),
            Tok::Dedent => "end of block".into(),
            Tok::Eof => "end of file".into(),
        }
    }

    /// Report an unexpected token, preferring a precise "not supported yet"
    /// message for spec keywords the bootstrap doesn't implement.
    fn unexpected(&mut self, expected: &str, rule: &'static str) {
        let pos = self.pos();
        if let Tok::Ident(name) = self.peek() {
            if let Some((_, spec)) = NOT_YET.iter().find(|(k, _)| k == name) {
                self.diags.push(Diagnostic::new(
                    UNSUPPORTED,
                    pos,
                    format!("`{name}` is not supported by the bootstrap yet (spec {spec})"),
                ));
                return;
            }
        }
        let found = Self::describe(self.peek());
        self.diags.push(Diagnostic::new(rule, pos, format!("expected {expected}, found {found}")));
    }

    /// Skip to the end of the current statement, including any indented
    /// block that follows it, so one error doesn't cascade.
    fn skip_statement(&mut self) {
        while !matches!(self.peek(), Tok::Newline | Tok::Eof) {
            self.i += 1;
        }
        self.eat(&Tok::Newline);
        if self.eat(&Tok::Indent) {
            let mut depth = 1;
            while depth > 0 && !matches!(self.peek(), Tok::Eof) {
                match self.bump().tok {
                    Tok::Indent => depth += 1,
                    Tok::Dedent => depth -= 1,
                    _ => {}
                }
            }
        }
    }

    fn ident(&mut self, expected: &str, rule: &'static str) -> Option<(String, Pos)> {
        if let Tok::Ident(name) = self.peek().clone() {
            if NOT_YET.iter().all(|(k, _)| *k != name) {
                let pos = self.pos();
                self.i += 1;
                return Some((name, pos));
            }
        }
        self.unexpected(expected, rule);
        None
    }

    /// `fn name(param Type, ...) [ReturnType]` followed by an indented body.
    fn function(&mut self) -> Option<FnDecl> {
        let fn_pos = self.bump().pos;
        let parsed = self.function_header(fn_pos);
        let Some((name, name_pos, params, ret)) = parsed else {
            self.skip_statement();
            return None;
        };
        if !self.eat(&Tok::Newline) {
            self.unexpected("end of line after the function signature", "gram.fn.1");
            self.skip_statement();
            return None;
        }
        if !self.eat(&Tok::Indent) {
            self.diags.push(
                Diagnostic::new("gram.fn.1", self.pos(), format!("function `{name}` has no body"))
                    .help("indent the body by 4 spaces under the signature"),
            );
            return None;
        }
        let mut body = Vec::new();
        while !matches!(self.peek(), Tok::Dedent | Tok::Eof) {
            if self.eat(&Tok::Newline) {
                continue;
            }
            if let Some(stmt) = self.statement() {
                body.push(stmt);
            }
        }
        self.eat(&Tok::Dedent);
        Some(FnDecl { name, pos: name_pos, params, ret, body })
    }

    #[allow(clippy::type_complexity)]
    fn function_header(&mut self, _fn_pos: Pos) -> Option<(String, Pos, Vec<Param>, Option<TypeRef>)> {
        let (name, name_pos) = self.ident("a function name after `fn`", "gram.fn.1")?;
        if !self.eat(&Tok::LParen) {
            self.unexpected("`(` after the function name", "gram.fn.1");
            return None;
        }
        let mut params = Vec::new();
        while !matches!(self.peek(), Tok::RParen) {
            let (pname, ppos) = self.ident("a parameter name", "gram.fn.2")?;
            if matches!(self.peek(), Tok::Colon) {
                self.diags.push(
                    Diagnostic::new("gram.fn.2", self.pos(), "parameters are written `name Type`, without a colon")
                        .help(format!("write `{pname} <Type>`")),
                );
                self.i += 1;
            }
            let (tname, tpos) = self.ident("the parameter's type (written `name Type`)", "gram.fn.2")?;
            params.push(Param { name: pname, pos: ppos, ty: TypeRef { name: tname, pos: tpos } });
            if !self.eat(&Tok::Comma) {
                break;
            }
        }
        if !self.eat(&Tok::RParen) {
            self.unexpected("`)` or `,` in the parameter list", "gram.fn.2");
            return None;
        }
        if matches!(self.peek(), Tok::Arrow) {
            self.diags.push(
                Diagnostic::new("gram.fn.2", self.pos(), "there is no `->`; the return type follows `)` directly")
                    .help("remove the `->`"),
            );
            self.i += 1;
        }
        let ret = match self.peek() {
            Tok::Ident(_) => {
                let (tname, tpos) = self.ident("a return type", "gram.fn.2")?;
                Some(TypeRef { name: tname, pos: tpos })
            }
            _ => None,
        };
        Some((name, name_pos, params, ret))
    }

    /// A statement ends at the end of its line (lex.stmt.1).
    fn statement(&mut self) -> Option<Stmt> {
        if matches!(self.peek(), Tok::Indent) {
            self.diags.push(Diagnostic::new("lex.block.1", self.pos(), "unexpected indentation"));
            self.skip_statement();
            return None;
        }
        let is_binding = matches!(self.peek(), Tok::Ident(_))
            && matches!(self.tokens.get(self.i + 1).map(|t| &t.tok), Some(Tok::ColonEq));
        let stmt = if is_binding {
            let (name, pos) = match self.ident("a binding name", "expr.bind.1") {
                Some(v) => v,
                None => {
                    self.skip_statement();
                    return None;
                }
            };
            self.i += 1; // `:=`
            match self.expression() {
                Some(value) => Stmt::Let { name, pos, value },
                None => {
                    self.skip_statement();
                    return None;
                }
            }
        } else {
            match self.expression() {
                Some(e) => Stmt::Expr(e),
                None => {
                    self.skip_statement();
                    return None;
                }
            }
        };
        if !self.eat(&Tok::Newline) {
            self.unexpected("end of line after the statement", "lex.stmt.1");
            self.skip_statement();
            return None;
        }
        Some(stmt)
    }

    fn expression(&mut self) -> Option<Expr> {
        let pos = self.pos();
        let mut expr = match self.peek().clone() {
            Tok::Str(s) => {
                self.i += 1;
                Expr::Str(s, pos)
            }
            Tok::Int(v) => {
                self.i += 1;
                Expr::Int(v, pos)
            }
            Tok::Ident(_) => {
                let (name, pos) = self.ident("an expression", "expr.name.1")?;
                Expr::Name(name, pos)
            }
            Tok::LParen => {
                self.i += 1;
                let inner = self.expression()?;
                if !self.eat(&Tok::RParen) {
                    self.unexpected("`)`", "lex.stmt.2");
                    return None;
                }
                inner
            }
            _ => {
                self.unexpected("an expression", "gram.syntax.1");
                return None;
            }
        };
        while matches!(self.peek(), Tok::LParen) {
            let call_pos = self.pos();
            self.i += 1;
            let mut args = Vec::new();
            while !matches!(self.peek(), Tok::RParen) {
                args.push(self.expression()?);
                if !self.eat(&Tok::Comma) {
                    break;
                }
            }
            if !self.eat(&Tok::RParen) {
                self.unexpected("`)` or `,` in the argument list", "expr.call.1");
                return None;
            }
            expr = Expr::Call { callee: Box::new(expr), args, pos: call_pos };
        }
        Some(expr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex;

    fn parse_src(src: &str) -> (Program, Vec<&'static str>) {
        let (tokens, lex_diags) = lex(src);
        assert!(lex_diags.is_empty(), "{lex_diags:?}");
        let (prog, diags) = parse(tokens);
        (prog, diags.into_iter().map(|d| d.rule).collect())
    }

    #[test]
    fn parses_hello_world() {
        let (prog, rules) = parse_src("fn main()\n    print(\"Hello, world!\")\n");
        assert!(rules.is_empty());
        assert_eq!(prog.fns.len(), 1);
        assert_eq!(prog.fns[0].name, "main");
        assert_eq!(prog.fns[0].body.len(), 1);
    }

    #[test]
    fn parses_params_return_type_and_bindings() {
        let (prog, rules) = parse_src("fn greet(name str) str\n    greeting := name\n    greeting\n");
        assert!(rules.is_empty(), "{rules:?}");
        let f = &prog.fns[0];
        assert_eq!(f.params.len(), 1);
        assert_eq!(f.ret.as_ref().map(|t| t.name.as_str()), Some("str"));
        assert!(matches!(f.body[0], Stmt::Let { .. }));
    }

    #[test]
    fn unsupported_keywords_are_reported_precisely() {
        let (_, rules) = parse_src("fn main()\n    while x\n        print(1)\n");
        assert_eq!(rules, vec![UNSUPPORTED]);
    }

    #[test]
    fn rust_style_signatures_get_precise_fixes() {
        let (_, rules) = parse_src("fn f(a: int) -> int\n    a\n");
        assert_eq!(rules, vec!["gram.fn.2", "gram.fn.2"]);
    }
}
