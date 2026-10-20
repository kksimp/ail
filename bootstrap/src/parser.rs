//! Parser: tokens to a syntax tree for the subset the bootstrap supports.
//!
//! Grammar rules: gram.fn.1-2, gram.list.1, gram.generic.1, gram.syntax.1,
//! lex.stmt.1, lex.kw.1, type.struct.1-3, type.enum.1, expr.bind.1-3,
//! expr.if.1, expr.loop.1-3, expr.match.1, expr.prec.1-2, expr.op.1-3,
//! gen.decl.1, gen.bound.1, gen.method.1-3, gen.infer.2, trait.decl.1-2,
//! trait.impl.1, trait.any.1.
//! Spec constructs the bootstrap doesn't implement yet report
//! `bootstrap.unsupported` with the rule they belong to.

use std::collections::HashSet;

use crate::ast::{
    Arg, Arm, Attr, BinOp, Block, CompFor, CompKind, Cond, ConstDecl, Else, EnumDecl, Expr, Field, FnDecl,
    GenericParam, ImplDecl, InterpPart, Literal, Owner, PRELUDE_LINE, Param, Pattern, Program, Receiver, Stmt,
    StructDecl, TraitDecl, TypeKind, TypeRef, UnOp, Variant, Within, is_prelude,
};
use crate::diag::{Diagnostic, Edit, Pos};
use crate::lexer::{Part, Tok, Token, UNSUPPORTED, lex};

mod contracts;
mod derive;
pub use contracts::link as link_inherited_contracts;
mod messages;
mod mistakes;

/// Keywords the spec defines that the bootstrap doesn't implement yet,
/// with the spec rule each belongs to.
const NOT_YET: &[(&str, &str)] = &[
    ("unsafe", "unsafe.block.1"),
    ("extern", "ffi.decl.2"),
    ("as", "ffi.decl.1"),
];

const ASSIGN_OPS: &[(&str, Option<BinOp>)] = &[
    ("=", None),
    ("+=", Some(BinOp::Add)),
    ("-=", Some(BinOp::Sub)),
    ("*=", Some(BinOp::Mul)),
    ("/=", Some(BinOp::Div)),
    ("%=", Some(BinOp::Rem)),
];

const COMPARISONS: &[(&str, BinOp)] =
    &[("==", BinOp::Eq), ("!=", BinOp::Ne), ("<", BinOp::Lt), ("<=", BinOp::Le), (">", BinOp::Gt), (">=", BinOp::Ge)];

/// The standard traits and `Ordering` (ADR 0127), read before every program.
const PRELUDE: &str = include_str!("prelude.ail");

/// Names of the optional `std.json` prelude module (ADR 0161).
const JSON_NAMES: &[&str] = &["Json", "JsonMember", "JsonError", "parse_json"];

/// Names of its derivable traits, read with it only when named (ADR 0308).
const JSON_DERIVE_NAMES: &[&str] = &["ToJson", "FromJson"];

/// Names of the optional `derive` prelude module: what a derive generator
/// receives (refl.info.1, ADR 0306).
const DERIVE_NAMES: &[&str] = &["TypeInfo", "TypeKind", "FieldInfo", "VariantInfo", "DeriveError"];

/// The always-read prelude, plus the `json` module (after its `## @module`
/// marker) when the program names one of its items and declares none of
/// them; its `jsonderive` part when the program names `ToJson` or
/// `FromJson`; and the `derive` module when `jsonderive` is read or the
/// program names one of its items or writes `@derive`.
fn prelude_text(tokens: &[Token]) -> String {
    let (core, rest) = PRELUDE.split_once("## @module json\n").expect("prelude has a json module");
    let (json, rest) = rest.split_once("## @module jsonderive\n").expect("prelude has a jsonderive module");
    let (json_derive, derive) = rest.split_once("## @module derive\n").expect("prelude has a derive module");
    let wants = |names: &[&str]| {
        let named = |t: &Token| matches!(&t.tok, Tok::Ident(n) if names.contains(&n.as_str()));
        let declared = tokens.windows(2).any(|w| matches!(w[0].tok, Tok::Kw("fn" | "struct" | "enum" | "trait")) && named(&w[1]));
        tokens.iter().any(named) && !declared
    };
    let want_json_derive = wants(JSON_DERIVE_NAMES);
    let want_json = want_json_derive || wants(JSON_NAMES);
    let at_derive = tokens.windows(2).any(|w| matches!((&w[0].tok, &w[1].tok), (Tok::Sym("@"), Tok::Ident(n)) if n == "derive"));
    let mut text = core.to_string();
    if want_json {
        text.push_str(json);
    }
    if want_json_derive {
        text.push_str(json_derive);
    }
    if want_json_derive || at_derive || wants(DERIVE_NAMES) {
        text.push_str(derive);
    }
    text
}

/// Parse a program after the prelude, whose lines are numbered from
/// `ast::PRELUDE_LINE`.
#[cfg(test)]
pub fn parse_with_prelude(tokens: Vec<Token>) -> (Program, Vec<Diagnostic>) {
    let (mut prog, mut diags) = parse_after_prelude(tokens);
    crate::implicit_self::rewrite(&mut prog, &mut diags);
    link_inherited_contracts(&mut prog);
    (prog, diags)
}

/// `parse_with_prelude` without the implicit-`self` pass, which a package
/// runs after naming its modules' items (`modules.rs`).
pub fn parse_after_prelude(tokens: Vec<Token>) -> (Program, Vec<Diagnostic>) {
    let (mut all, diags) = lex(&prelude_text(&tokens));
    debug_assert!(diags.is_empty(), "{diags:?}");
    all.pop(); // Eof
    fn shift(tokens: &mut [Token]) {
        for t in tokens {
            t.pos.line += PRELUDE_LINE;
            if let Tok::Interp(parts) = &mut t.tok {
                for p in parts {
                    if let Part::Code(code) | Part::Fmt(code, _) = p {
                        shift(code);
                    }
                }
            }
        }
    }
    shift(&mut all);
    all.extend(tokens);
    let (mut prog, diags) = parse(all);
    crate::stdlib::add_declarations(&mut prog);
    (prog, diags)
}

/// Names declared generic (`fn f[`, `struct S[`, `enum E[`): after one of
/// these, `[` starts type arguments rather than an index (gen.infer.2).
fn generic_names(tokens: &[Token]) -> HashSet<String> {
    // The built-in collections: `List[int]()` (type.coll.8).
    let mut names: HashSet<String> = ["List", "Map", "Set", "Channel", "Mutex"].iter().map(|n| n.to_string()).collect();
    for w in tokens.windows(3) {
        if let [
            Token { tok: Tok::Kw("fn" | "struct" | "enum"), .. },
            Token { tok: Tok::Ident(n), .. },
            Token { tok: Tok::Sym("["), .. },
        ] = w
        {
            names.insert(n.clone());
        }
    }
    names
}

pub fn parse(tokens: Vec<Token>) -> (Program, Vec<Diagnostic>) {
    let generic_names = generic_names(&tokens);
    let mut p = Parser {
        tokens,
        i: 0,
        diags: Vec::new(),
        in_interp: false,
        generic_names,
        olds: None,
        aliases: HashSet::new(),
        public: HashSet::new(),
    };
    let mut prog = Program::default();
    loop {
        match p.peek().clone() {
            Tok::Eof => break,
            Tok::Newline | Tok::Dedent => p.i += 1,
            Tok::Indent => {
                p.diags.push(Diagnostic::new("lex.block.1", p.pos(), "unexpected indentation"));
                p.skip_block();
            }
            Tok::Kw("fn") => {
                if let Some(f) = p.function(Within::TopLevel) {
                    prog.fns.push(f);
                }
            }
            Tok::Kw("use") => p.use_decl(&mut prog),
            Tok::Kw("pub") => p.pub_marker(),
            // `@name fn ...` (gram.attr.1-2)
            // `@derive(f) trait Name` (ct.derive.1)
            Tok::Sym("@") if p.derive_trait(&mut prog) => {}
            Tok::Sym("@") => {
                if let Some(f) = p.attributed_function() {
                    prog.fns.push(f);
                }
            }
            Tok::Kw("trait") => p.trait_decl(&mut prog),
            Tok::Kw("impl") => p.impl_decl(&mut prog),
            // `NAME := expr`: a constant (mod.const.1).
            Tok::Ident(name) if matches!(p.peek_at(1), Tok::Sym(":=")) => {
                let pos = p.bump().pos;
                p.i += 1; // `:=`
                match p.expression() {
                    Some(value) if p.eat(&Tok::Newline) => prog.consts.push(ConstDecl { name, pos, doc: None, value }),
                    Some(_) => {
                        p.unexpected("end of line after the constant", "lex.stmt.1");
                        p.skip_statement();
                    }
                    None => p.skip_statement(),
                }
            }
            Tok::Kw("struct") => {
                if let Some(s) = p.struct_decl(false) {
                    prog.structs.push(s);
                }
            }
            // `unique struct` and `unique enum` (own.uniq.1).
            Tok::Kw("unique") => match p.peek_at(1) {
                Tok::Kw("struct") => {
                    p.i += 1;
                    if let Some(mut s) = p.struct_decl(false) {
                        s.unique = true;
                        prog.structs.push(s);
                    }
                }
                Tok::Kw("enum") => {
                    p.i += 1;
                    if let Some(mut e) = p.enum_decl(false) {
                        e.unique = true;
                        prog.enums.push(e);
                    }
                }
                _ => {
                    p.i += 1;
                    p.unexpected("`struct` or `enum` after `unique`", "own.uniq.1");
                    p.skip_statement();
                }
            },
            Tok::Kw("enum") => {
                if let Some(e) = p.enum_decl(false) {
                    prog.enums.push(e);
                }
            }
            // `error struct` and `error enum` declare error types (err.decl.1).
            Tok::Kw("error") => match p.peek_at(1) {
                Tok::Kw("struct") => {
                    p.i += 1;
                    if let Some(s) = p.struct_decl(true) {
                        prog.structs.push(s);
                    }
                }
                Tok::Kw("enum") => {
                    p.i += 1;
                    if let Some(e) = p.enum_decl(true) {
                        prog.enums.push(e);
                    }
                }
                _ => {
                    p.i += 1;
                    p.unexpected("`struct` or `enum` after `error`", "err.decl.1");
                    p.skip_statement();
                }
            },
            Tok::Kw("mut") if matches!(p.peek_at(1), Tok::Ident(_)) => {
                let pos = p.pos();
                p.error_help(
                    "mod.const.1",
                    pos,
                    "there are no mutable top-level variables",
                    "declare a constant `NAME := value`, or pass the value to the functions that change it",
                );
                p.skip_statement();
            }
            Tok::Ident(word) if ["const", "let", "var"].contains(&word.as_str()) && matches!(p.peek_at(1), Tok::Ident(_)) => {
                let pos = p.pos();
                let mut d = Diagnostic::new(
                    "mod.const.1",
                    pos,
                    format!("AIL has no `{word}`; a constant is declared `NAME := value`"),
                )
                .help("remove the keyword and write `NAME := value`");
                if matches!(p.peek_at(2), Tok::Sym("=")) {
                    let eq = p.tokens[p.i + 2].pos;
                    d = d.fix(
                        "declare the constant with `:=`",
                        vec![
                            Edit::delete(pos, p.tokens[p.i + 1].pos),
                            Edit::span(eq, Pos { col: eq.col + 1, ..eq }, ":="),
                        ],
                    );
                }
                p.diags.push(d);
                p.skip_statement();
            }
            Tok::Ident(word) if p.foreign_decl(&word) => {}
            _ => {
                p.unexpected(
                    "a declaration (`fn`, `struct`, `enum`, `error`, `unique`, `trait`, `impl` or a constant)",
                    "gram.syntax.1",
                );
                p.skip_statement();
            }
        }
    }
    prog.public = std::mem::take(&mut p.public);
    (prog, p.diags)
}

mod modules;

struct Parser {
    tokens: Vec<Token>,
    i: usize,
    diags: Vec<Diagnostic>,
    /// Parsing the expression inside a string's `{...}`, which ends at `}`.
    in_interp: bool,
    /// The names bound by `use` so far: `m.Stack[int]` is a qualified generic
    /// type, not an index (gen.infer.2, mod.use.2).
    aliases: HashSet<String>,
    /// The lines of `pub` markers (`Program::public`).
    public: HashSet<usize>,
    generic_names: HashSet<String>,
    /// While parsing an `ensures` condition: its `old(e)` calls (contract.ens.3).
    olds: Option<Vec<(String, Expr)>>,
}

impl Parser {
    fn peek(&self) -> &Tok {
        &self.tokens[self.i].tok
    }

    fn peek_at(&self, n: usize) -> &Tok {
        let last = self.tokens.len() - 1;
        &self.tokens[(self.i + n).min(last)].tok
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

    fn at_sym(&self, s: &str) -> bool {
        matches!(self.peek(), Tok::Sym(x) if *x == s)
    }

    fn eat_sym(&mut self, s: &str) -> bool {
        let found = self.at_sym(s);
        if found {
            self.i += 1;
        }
        found
    }

    fn at_kw(&self, k: &str) -> bool {
        matches!(self.peek(), Tok::Kw(x) if *x == k)
    }

    fn eat_kw(&mut self, k: &str) -> bool {
        let found = self.at_kw(k);
        if found {
            self.i += 1;
        }
        found
    }

    /// The previous token closed a block, so the statement ended there.
    fn after_block(&self) -> bool {
        self.i > 0 && matches!(self.tokens[self.i - 1].tok, Tok::Dedent)
    }

    fn describe(&self, tok: &Tok) -> String {
        match tok {
            Tok::Eof if self.in_interp => "`}`".into(),
            Tok::Ident(n) => format!("`{n}`"),
            Tok::Kw(k) => format!("the keyword `{k}`"),
            Tok::Sym(s) => format!("`{s}`"),
            Tok::Str(_) | Tok::Interp(_) => "a string".into(),
            Tok::Int(_) | Tok::IntMin => "an integer".into(),
            Tok::Float(_) => "a number".into(),
            Tok::Char(_) => "a character".into(),
            Tok::Newline => "end of line".into(),
            Tok::Indent => "an indented block".into(),
            Tok::Dedent => "end of block".into(),
            Tok::Eof => "end of file".into(),
        }
    }

    /// `9223372036854775808` without a `-` before it (lex.num.3).
    fn int_too_big(&mut self, pos: Pos) {
        self.error("lex.num.3", pos, "integer literal `9223372036854775808` doesn't fit in `int` (64-bit signed)");
    }

    fn error(&mut self, rule: &'static str, pos: Pos, msg: impl Into<String>) {
        self.diags.push(Diagnostic::new(rule, pos, msg));
    }

    fn error_help(&mut self, rule: &'static str, pos: Pos, msg: impl Into<String>, help: impl Into<String>) {
        self.diags.push(Diagnostic::new(rule, pos, msg).help(help));
    }

    /// Report an unexpected token, preferring a precise "not supported yet"
    /// message for spec keywords the bootstrap doesn't implement.
    /// `for x in e` or `for mut x in e` (expr.loop.1, expr.loop.6), the
    /// current token being `for`; `par` after `par` (conc.par.5).
    fn for_loop(&mut self, par: bool, pos: Pos) -> Option<Stmt> {
        self.bump();
        let mutable = self.eat_kw("mut");
        if self.at_sym("(") {
            let at = self.pos();
            self.error_help(
                "expr.loop.1",
                at,
                "AIL has no C-style `for`",
                "loop over a range: `for i in 0..n` (excludes `n`) or `for i in 1..=n` (includes it); use `while` for other conditions",
            );
            return None;
        }
        let (var, var_pos) = self.name("a loop variable after `for`", "expr.loop.1")?;
        if self.at_sym(",") {
            let at = self.pos();
            self.error_help(
                "expr.loop.1",
                at,
                "a `for` loop has one variable",
                "for the index and the item, loop over the indexes: `for i in 0..xs.len()` then `xs[i]`; for a map, `for k in m.keys()` then `m[k]`",
            );
            return None;
        }
        if !self.eat_kw("in") {
            self.unexpected("`in` after the loop variable", "expr.loop.1");
            return None;
        }
        let iter = self.range()?;
        // err.else.6: `for x in xs else []` (ADR 0252).
        let iter = self.else_expr(iter)?;
        let body = self.block("the `for` header", "expr.loop.1")?;
        Some(Stmt::For { par, var, var_pos, mutable, iter, body, pos })
    }

    fn unexpected(&mut self, expected: &str, rule: &'static str) {
        let pos = self.pos();
        if let Tok::Kw(k) = self.peek()
            && let Some((_, spec)) = NOT_YET.iter().find(|(n, _)| n == k)
        {
            let k = *k;
            self.error(UNSUPPORTED, pos, format!("`{k}` is not supported by the bootstrap yet (spec {spec})"));
            return;
        }
        let found = self.describe(self.peek());
        self.error(rule, pos, format!("expected {expected}, found {found}"));
    }

    fn expect_sym(&mut self, s: &str, expected: &str, rule: &'static str) -> Option<()> {
        if self.eat_sym(s) {
            Some(())
        } else {
            self.unexpected(expected, rule);
            None
        }
    }

    /// Skip to the end of the current statement, including any indented
    /// blocks that follow it, so one error doesn't cascade.
    fn skip_statement(&mut self) {
        while !matches!(self.peek(), Tok::Newline | Tok::Indent | Tok::Dedent | Tok::Eof) {
            self.i += 1;
        }
        self.eat(&Tok::Newline);
        while matches!(self.peek(), Tok::Indent) {
            self.skip_block();
            // `elif`/`else` continue the statement whose block was skipped.
            if self.at_kw("elif") || self.at_kw("else") {
                while !matches!(self.peek(), Tok::Newline | Tok::Indent | Tok::Dedent | Tok::Eof) {
                    self.i += 1;
                }
                self.eat(&Tok::Newline);
            }
        }
    }

    /// Skip an indented block, starting at its INDENT.
    fn skip_block(&mut self) {
        if !self.eat(&Tok::Indent) {
            return;
        }
        let mut depth = 1;
        while depth > 0 && !matches!(self.peek(), Tok::Eof) {
            match self.bump().tok {
                Tok::Indent => depth += 1,
                Tok::Dedent => depth -= 1,
                _ => {}
            }
        }
    }

    /// A name (not a keyword).
    fn name(&mut self, expected: &str, rule: &'static str) -> Option<(String, Pos)> {
        match self.peek().clone() {
            Tok::Ident(name) => {
                let pos = self.bump().pos;
                Some((name, pos))
            }
            Tok::Kw(k) if NOT_YET.iter().all(|(n, _)| *n != k) => {
                let pos = self.pos();
                self.error_help(
                    "lex.kw.1",
                    pos,
                    format!("`{k}` is a keyword and can't be used as a name"),
                    "choose a different name",
                );
                None
            }
            _ => {
                self.unexpected(expected, rule);
                None
            }
        }
    }

    /// NEWLINE INDENT statement+ DEDENT, after a line that opens a block.
    fn block(&mut self, after: &str, rule: &'static str) -> Option<Block> {
        if !self.eat(&Tok::Newline) {
            match self.header_tail(after) {
                mistakes::Header::Continue => {}
                mistakes::Header::Reported => return None,
                mistakes::Header::No => {
                    self.unexpected(&format!("end of line after {after}"), rule);
                    return None;
                }
            }
        }
        if !self.eat(&Tok::Indent) {
            let pos = self.pos();
            self.error_help(
                rule,
                pos,
                format!("expected an indented block after {after}"),
                "indent the block by 4 spaces",
            );
            return Some(Vec::new());
        }
        Some(self.block_rest())
    }

    /// The statements of a block after its INDENT, through its DEDENT.
    fn block_rest(&mut self) -> Block {
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
        body
    }

    // ---- Declarations ----

    /// `fn [Type.]name(params) [ReturnType]` followed by an indented body.
    /// In a trait, the body is optional (trait.decl.2).
    fn function(&mut self, within: Within) -> Option<FnDecl> {
        let f = self.function_inner(within);
        if f.is_none() {
            self.skip_statement();
        }
        f
    }

    /// `@name` or `@name(args)`, repeated, then a function on the same line
    /// (gram.attr.1-2).
    fn attributed_function(&mut self) -> Option<FnDecl> {
        let mut attrs = Vec::new();
        while self.at_sym("@") {
            self.i += 1;
            let Some((name, pos)) = self.name("an attribute name after `@`", "gram.attr.1") else {
                self.skip_statement();
                return None;
            };
            let mut arg = None;
            if self.at_sym("(") {
                let mut depth = 0usize;
                loop {
                    match self.peek() {
                        Tok::Str(text) if depth == 1 && arg.is_none() => arg = Some(text.clone()),
                        Tok::Sym("(") => depth += 1,
                        Tok::Sym(")") => depth -= 1,
                        Tok::Newline | Tok::Eof => break,
                        _ => {}
                    }
                    self.i += 1;
                    if depth == 0 {
                        break;
                    }
                }
            }
            attrs.push(Attr { name, pos, arg });
            // gram.attr.2: an attribute may sit on its own line above its function.
            while matches!(self.peek(), Tok::Newline)
                && (matches!(self.peek_at(1), Tok::Sym("@")) || matches!(self.peek_at(1), Tok::Kw("fn")))
            {
                self.i += 1;
            }
        }
        if !self.at_kw("fn") {
            let pos = self.pos();
            self.error_help(
                "gram.attr.2",
                pos,
                "an attribute applies to the function declared right after it",
                "write `@test fn name()` or put the function on the next line",
            );
            self.skip_statement();
            return None;
        }
        let mut f = self.function(Within::TopLevel)?;
        f.attrs = attrs;
        Some(f)
    }

    fn function_inner(&mut self, within: Within) -> Option<FnDecl> {
        self.bump(); // fn
        let (first, first_pos) = self.name("a function name after `fn`", "gram.fn.1")?;
        // `fn f[T](...)`, `fn Type.m(...)`, `fn Type[T Bound].m(...)` (gen.method.1-2).
        let mut generics = if self.at_sym("[") { self.generic_params()? } else { Vec::new() };
        let (owner, name, pos) = if self.eat_sym(".") {
            let params = if generics.is_empty() { None } else { Some(std::mem::take(&mut generics)) };
            let (name, pos) = self.name("a method name after `.`", "trait.method.1")?;
            if self.at_sym("[") {
                generics = self.generic_params()?;
            }
            if self.at_sym(".") {
                let at = self.pos();
                self.error_help(
                    "mod.unit.4",
                    at,
                    "a method is declared in the module of its type",
                    format!("move the method into the module that declares `{first}.{name}`"),
                );
                return None;
            }
            (Some(Owner { name: first, pos: first_pos, params }), name, pos)
        } else {
            (None, first, first_pos)
        };
        if owner.is_some() && within != Within::TopLevel {
            let block = if matches!(within, Within::Trait(_)) { "a `trait`" } else { "an `impl`" };
            self.error_help(
                "trait.impl.1",
                pos,
                format!("functions in {block} block are written without a type name"),
                format!("write `fn {name}(...)`"),
            );
            return None;
        }
        self.expect_sym("(", "`(` after the function name", "gram.fn.1")?;
        let mut receiver = None;
        let mut params = Vec::new();
        while !self.at_sym(")") {
            let is_self = self.at_kw("self")
                || ((self.at_kw("mut") || self.at_kw("move")) && matches!(self.peek_at(1), Tok::Kw("self")));
            if is_self {
                let mutable = self.eat_kw("mut");
                let moved = !mutable && self.eat_kw("move");
                let pos = self.bump().pos;
                if owner.is_none() && within == Within::TopLevel {
                    self.error_help(
                        "trait.method.1",
                        pos,
                        "`self` is only allowed in methods",
                        format!("declare a method as `fn Type.{name}(self, ...)`"),
                    );
                } else if receiver.is_some() || !params.is_empty() {
                    self.error("trait.method.1", pos, "`self` must be the first parameter");
                } else if moved && within != Within::TopLevel {
                    self.error_help(
                        UNSUPPORTED,
                        pos,
                        "`move self` in a trait or `impl` block is not supported by the bootstrap yet (spec own.move.6, OQ-050)",
                        format!("declare it as an inherent method, `fn Type.{name}(move self)`"),
                    );
                    return None;
                } else {
                    receiver = Some(Receiver { mutable, moved });
                }
            } else {
                let mutable = self.eat_kw("mut");
                let moved = !mutable && self.eat_kw("move");
                let (pname, ppos) = self.name("a parameter name", "gram.fn.2")?;
                if self.at_sym(":") {
                    let pos = self.pos();
                    let end = self.tokens.get(self.i + 1).filter(|t| t.pos.line == pos.line).map_or(pos, |t| t.pos);
                    self.error_fix(
                        "gram.fn.2",
                        pos,
                        "parameters are written `name Type`, without a colon",
                        format!("write `{pname} <Type>`"),
                        "remove the `:`",
                        vec![Edit::span(pos, end, " ")],
                    );
                    self.i += 1;
                }
                let ty = self.type_ref("the parameter's type (written `name Type`)", "gram.fn.2")?;
                params.push(Param { name: pname, pos: ppos, mutable, moved, ty });
            }
            if !self.eat_sym(",") {
                break;
            }
        }
        self.expect_sym(")", "`)` or `,` in the parameter list", "gram.fn.2")?;
        if self.at_sym("->") || (self.at_sym(":") && !matches!(self.peek_at(1), Tok::Newline)) {
            let pos = self.pos();
            let sym = if self.at_sym("->") { "->" } else { ":" };
            let end = self.tokens.get(self.i + 1).filter(|t| t.pos.line == pos.line).map_or(Pos { col: pos.col + sym.len(), ..pos }, |t| t.pos);
            self.error_fix(
                "gram.fn.2",
                pos,
                format!("there is no `{sym}`; the return type follows `)` directly"),
                format!("remove the `{sym}`"),
                format!("remove the `{sym}`"),
                vec![Edit::delete(pos, end)],
            );
            self.i += 1;
        }
        let ret = match self.peek() {
            Tok::Ident(_) | Tok::Sym("!") | Tok::Kw("any" | "ref") => Some(self.type_ref("a return type", "gram.fn.2")?),
            _ => None,
        };
        let full = match &owner {
            Some(o) => format!("`{}.{name}`", o.name),
            None => format!("`{name}`"),
        };
        // A trait method without a default ends with its signature (trait.decl.2).
        let (body, contracts) = if matches!(within, Within::Trait(_)) && !matches!(self.peek_at(1), Tok::Indent) {
            if !self.eat(&Tok::Newline) {
                self.unexpected("end of line after the signature", "trait.decl.1");
                return None;
            }
            (None, Vec::new())
        } else {
            self.fn_body(&format!("the signature of {full}"), matches!(within, Within::Trait(_)))?
        };
        Some(FnDecl {
            attrs: Vec::new(),
            owner,
            name,
            pos,
            doc: None,
            generics,
            receiver,
            params,
            ret,
            body,
            within,
            contracts,
            inherits: None,
        })
    }

    /// `[T, U Bound + Bound]` (gen.decl.1, gen.bound.1).
    fn generic_params(&mut self) -> Option<Vec<GenericParam>> {
        self.bump(); // [
        let mut params = Vec::new();
        while !self.at_sym("]") {
            let (name, pos) = self.name("a type parameter name", "gen.decl.1")?;
            let mut bounds = Vec::new();
            if matches!(self.peek(), Tok::Ident(_)) {
                bounds.push(self.dotted_name("a trait", "gen.bound.1")?);
                while self.eat_sym("+") {
                    bounds.push(self.dotted_name("a trait after `+`", "gen.bound.1")?);
                }
            }
            params.push(GenericParam { name, pos, bounds });
            if !self.eat_sym(",") {
                break;
            }
        }
        self.expect_sym("]", "`]` or `,` in the type parameters", "gen.decl.1")?;
        Some(params)
    }

    /// `trait Name` followed by an indented block of functions (trait.decl.1).
    fn trait_decl(&mut self, prog: &mut Program) {
        self.bump(); // trait
        let Some((name, pos)) = self.name("a trait name after `trait`", "trait.decl.1") else {
            self.skip_statement();
            return;
        };
        if self.at_sym("[") {
            let at = self.pos();
            self.error(UNSUPPORTED, at, "traits with type parameters are not designed yet (OQ-037)");
            self.skip_statement();
            return;
        }
        let index = prog.traits.len();
        prog.traits.push(TraitDecl { name: name.clone(), pos, doc: None, methods: Vec::new(), derive: None });
        if !self.eat(&Tok::Newline) {
            self.unexpected(&format!("end of line after `trait {name}`"), "trait.decl.1");
            self.skip_statement();
            return;
        }
        // The prelude's `Hash` has no methods: the language provides it (trait.auto.5).
        if is_prelude(pos) && !matches!(self.peek(), Tok::Indent) {
            return;
        }
        let methods = self.fn_block(prog, Within::Trait(index), &format!("`trait {name}`"));
        prog.traits[index].methods = methods;
    }

    /// `impl Trait for Type`, with an optional indented block of functions
    /// (trait.impl.1, trait.derive.1).
    fn impl_decl(&mut self, prog: &mut Program) {
        let pos = self.bump().pos; // impl
        let parsed = (|| {
            let (trait_name, trait_pos) = self.dotted_name("a trait name after `impl`", "trait.impl.1")?;
            if !self.eat_kw("for") {
                if matches!(self.peek(), Tok::Newline) {
                    let at = self.tokens[self.i - 1].pos;
                    self.error_help(
                        "trait.impl.1",
                        at,
                        format!("`impl {trait_name}` needs `for Type`: `impl` implements a trait"),
                        format!("methods of a struct are plain functions: `fn {trait_name}.name(self) T`"),
                    );
                } else {
                    self.unexpected("`for` after the trait name", "trait.impl.1");
                }
                return None;
            }
            let (name, tpos) = self.dotted_name("the implementing type after `for`", "trait.impl.1")?;
            let params = if self.at_sym("[") { Some(self.generic_params()?) } else { None };
            Some((trait_name, trait_pos, Owner { name, pos: tpos, params }))
        })();
        let Some((trait_name, trait_pos, target)) = parsed else {
            self.skip_statement();
            return;
        };
        let index = prog.impls.len();
        let label = format!("`impl {trait_name} for {}`", target.name);
        prog.impls.push(ImplDecl { trait_name, trait_pos, target, pos, methods: Vec::new() });
        if !self.eat(&Tok::Newline) {
            self.unexpected(&format!("end of line after {label}"), "trait.impl.1");
            self.skip_statement();
            return;
        }
        if matches!(self.peek(), Tok::Indent) {
            let methods = self.fn_block(prog, Within::Impl(index), &label);
            prog.impls[index].methods = methods;
        }
    }

    /// The indented functions of a `trait` or `impl` block, after its first
    /// line's end of line. Returns their indexes in `prog.fns`.
    fn fn_block(&mut self, prog: &mut Program, within: Within, what: &str) -> Vec<usize> {
        let mut indexes = Vec::new();
        if !self.eat(&Tok::Indent) {
            if matches!(within, Within::Trait(_)) {
                let pos = self.pos();
                self.error_help(
                    "trait.decl.1",
                    pos,
                    format!("{what} needs an indented block of methods"),
                    "indent each method by 4 spaces",
                );
            }
            return indexes;
        }
        while !matches!(self.peek(), Tok::Dedent | Tok::Eof) {
            if self.eat(&Tok::Newline) {
                continue;
            }
            self.member_pub("trait and `impl` methods");
            if !self.at_kw("fn") {
                self.unexpected(&format!("a method (`fn ...`) in {what}"), "trait.decl.1");
                self.skip_statement();
                continue;
            }
            if let Some(f) = self.function(within) {
                indexes.push(prog.fns.len());
                prog.fns.push(f);
            }
        }
        self.eat(&Tok::Dedent);
        indexes
    }

    /// A type: `T`, `T?`, `T ! E`, `T ! A | B`, `!T`, or a bare `!`
    /// (type.opt.2, err.type.1-2, err.type.5).
    fn type_ref(&mut self, expected: &str, rule: &'static str) -> Option<TypeRef> {
        let pos = self.pos();
        if self.eat_sym("!") {
            let ok = match self.peek() {
                Tok::Ident(_) => Some(Box::new(self.optional_type(expected, rule)?)),
                Tok::Kw("any") => Some(Box::new(self.type_ref(expected, rule)?)),
                _ => None,
            };
            // `! A | B`: no success value, and these errors (err.type.7).
            if let Some(first) = &ok
                && self.at_sym("|")
            {
                let mut errors = vec![(**first).clone()];
                while self.eat_sym("|") {
                    errors.push(self.named_type("an error type after `|`", "err.type.5")?);
                }
                return Some(TypeRef { kind: TypeKind::Fallible { ok: None, errors: Some(errors) }, pos });
            }
            return Some(TypeRef { kind: TypeKind::Fallible { ok, errors: None }, pos });
        }
        if self.eat_kw("any") {
            let (name, _) = self.dotted_name("a trait after `any`", "trait.any.1")?;
            return Some(TypeRef { kind: TypeKind::Any(name), pos });
        }
        let ok = self.optional_type(expected, rule)?;
        if !self.eat_sym("!") {
            return Some(ok);
        }
        let mut errors = vec![self.named_type("an error type after `!`", "err.type.1")?];
        while self.eat_sym("|") {
            errors.push(self.named_type("an error type after `|`", "err.type.5")?);
        }
        Some(TypeRef { kind: TypeKind::Fallible { ok: Some(Box::new(ok)), errors: Some(errors) }, pos })
    }

    /// `T` or `T?` (type.opt.2).
    fn optional_type(&mut self, expected: &str, rule: &'static str) -> Option<TypeRef> {
        let mut t = self.named_type(expected, rule)?;
        // type.opt.6: `T??` is an optional of an optional.
        while self.eat_sym("?") {
            let pos = t.pos;
            t = TypeRef { kind: TypeKind::Optional(Box::new(t)), pos };
        }
        Some(t)
    }

    /// `Name` or `Name[Arg, ...]` (gram.generic.1), or a module's type
    /// `m.Name`, kept as the dotted name (mod.use.2, std.use.2).
    fn named_type(&mut self, expected: &str, rule: &'static str) -> Option<TypeRef> {
        // `ref T` (own.ref.1); `ref T?` is an optional reference.
        if self.at_kw("ref") {
            let pos = self.bump().pos;
            let inner = self.named_type("a type after `ref`", "own.ref.1")?;
            return Some(TypeRef { kind: TypeKind::Ref(Box::new(inner)), pos });
        }
        let (name, pos) = self.dotted_name(expected, rule)?;
        let mut args = Vec::new();
        if self.at_sym("<") {
            let at = self.pos();
            self.error_help(
                "gram.generic.1",
                at,
                "type arguments are written in square brackets",
                format!("write `{name}[...]`"),
            );
            return None;
        }
        if self.eat_sym("[") {
            loop {
                args.push(self.type_ref("a type argument", "gram.generic.1")?);
                if !self.eat_sym(",") || self.at_sym("]") {
                    break;
                }
            }
            self.expect_sym("]", "`]` after the type arguments", "gram.generic.1")?;
        }
        Some(TypeRef { kind: TypeKind::Named { name, args }, pos })
    }

    /// `name Type`, a field of a struct or of a variant (type.struct.2, type.enum.2).
    fn field_decl(&mut self, rule: &'static str) -> Option<Field> {
        self.field_pub(rule);
        if self.at_kw("fn") {
            let at = self.pos();
            self.error_help(
                rule,
                at,
                "methods aren't declared inside a struct",
                "declare the method after the struct, with the type's name: `fn Name.method(self) T`",
            );
            return None;
        }
        let (fname, fpos) = self.name("a field name", rule)?;
        if self.at_sym(":") {
            let at = self.pos();
            let end = self.tokens.get(self.i + 1).filter(|t| t.pos.line == at.line).map_or(at, |t| t.pos);
            self.error_fix(
                rule,
                at,
                "fields are written `name Type`, without a colon",
                format!("write `{fname} <Type>`"),
                "remove the `:`",
                vec![Edit::span(at, end, " ")],
            );
            self.i += 1;
        }
        let ty = self.type_ref("the field's type (written `name Type`)", rule)?;
        Some(Field { name: fname, pos: fpos, doc: None, ty })
    }

    /// `struct Name` followed by one `name Type` field per line (type.struct.1-2).
    fn struct_decl(&mut self, is_error: bool) -> Option<StructDecl> {
        self.bump(); // struct
        let Some((name, pos)) = self.name("a struct name after `struct`", "type.struct.1") else {
            self.skip_statement();
            return None;
        };
        let generics = if self.at_sym("[") {
            match self.generic_params() {
                Some(g) => g,
                None => {
                    self.skip_statement();
                    return None;
                }
            }
        } else {
            Vec::new()
        };
        let whole = self.error_message(is_error);
        let mut fields = Vec::new();
        let mut invariants = Vec::new();
        // err.msg.1: an error struct with a message may have no fields.
        let bare = whole.is_some() && matches!(self.peek(), Tok::Newline) && !matches!(self.peek_at(1), Tok::Indent);
        if bare {
            self.bump();
        } else {
            self.members(&format!("`struct {name}`"), "type.struct.1", |p| {
                if p.at_kw("invariant") {
                    invariants.push(p.invariant_line()?);
                    return Some(());
                }
                fields.push(p.field_decl("type.struct.2")?);
                Some(())
            });
        }
        if whole.is_some() {
            self.splice_messages(&name, pos, !generics.is_empty(), Vec::new(), whole);
        }
        Some(StructDecl { name, pos, doc: None, is_error, unique: false, generics, fields, invariants })
    }

    /// `enum Name` followed by one variant per line (type.enum.1-2).
    fn enum_decl(&mut self, is_error: bool) -> Option<EnumDecl> {
        self.bump(); // enum
        let Some((name, pos)) = self.name("an enum name after `enum`", "type.enum.1") else {
            self.skip_statement();
            return None;
        };
        let generics = if self.at_sym("[") {
            match self.generic_params() {
                Some(g) => g,
                None => {
                    self.skip_statement();
                    return None;
                }
            }
        } else {
            Vec::new()
        };
        let mut variants = Vec::new();
        let mut messages = Vec::new();
        self.members(&format!("`enum {name}`"), "type.enum.1", |p| {
            p.member_pub("enum variants");
            let (vname, vpos) = p.name("a variant name", "type.enum.1")?;
            let mut fields = Vec::new();
            if p.eat_sym("(") {
                while !p.at_sym(")") {
                    fields.push(p.field_decl("type.enum.2")?);
                    if !p.eat_sym(",") {
                        break;
                    }
                }
                p.expect_sym(")", "`)` or `,` in the variant's fields", "type.enum.2")?;
            }
            let field_names = fields.iter().map(|f| f.name.clone()).collect();
            messages.push((vname.clone(), field_names, p.error_message(is_error)));
            variants.push(Variant { name: vname, pos: vpos, doc: None, fields });
            Some(())
        });
        self.splice_messages(&name, pos, !generics.is_empty(), messages, None);
        Some(EnumDecl { name, pos, doc: None, is_error, unique: false, generics, variants })
    }

    /// An indented block of one-line members, each parsed by `member`.
    fn members(&mut self, what: &str, rule: &'static str, mut member: impl FnMut(&mut Self) -> Option<()>) {
        if !self.eat(&Tok::Newline) {
            self.unexpected(&format!("end of line after {what}"), rule);
            self.skip_statement();
            return;
        }
        if !self.eat(&Tok::Indent) {
            let pos = self.pos();
            self.error_help(
                rule,
                pos,
                format!("{what} needs an indented block of members"),
                "indent each member by 4 spaces",
            );
            return;
        }
        while !matches!(self.peek(), Tok::Dedent | Tok::Eof) {
            if self.eat(&Tok::Newline) {
                continue;
            }
            if member(self).is_some() && self.eat(&Tok::Newline) {
                continue;
            }
            if !self.diags.last().is_some_and(|d| d.pos.line == self.pos().line) {
                self.unexpected("end of line", "lex.stmt.1");
            }
            self.skip_statement();
        }
        self.eat(&Tok::Dedent);
    }

    // ---- Statements ----

    /// A statement ends at the end of its line (lex.stmt.1), or with its block.
    fn statement(&mut self) -> Option<Stmt> {
        if matches!(self.peek(), Tok::Indent) {
            let pos = self.pos();
            self.error_help(
                "lex.block.1",
                pos,
                "unexpected indentation",
                "only a line that opens a block (like `if`, `for` or `fn`) can be followed by an indented line",
            );
            self.skip_block();
            return None;
        }
        if self.misplaced_contract() {
            return None;
        }
        let errors_before = self.diags.len();
        let start = self.i;
        match self.statement_inner() {
            Some(stmt) if self.after_block() || self.eat(&Tok::Newline) => Some(stmt),
            Some(_) => {
                if !self.stray_after_statement(start) {
                    self.unexpected("end of line after the statement", "lex.stmt.1");
                }
                self.skip_statement();
                None
            }
            None => {
                if self.diags.len() == errors_before {
                    self.unexpected("a statement", "gram.syntax.1");
                }
                self.skip_statement();
                None
            }
        }
    }

    fn statement_inner(&mut self) -> Option<Stmt> {
        let pos = self.pos();
        match self.peek().clone() {
            // `defer S` (own.defer.1).
            Tok::Kw("defer") => {
                self.bump();
                let inner = self.statement_inner()?;
                Some(Stmt::Defer(Box::new(inner), pos))
            }
            Tok::Kw("mut") => {
                self.bump();
                let (name, pos) = self.name("a binding name after `mut`", "expr.bind.2")?;
                self.expect_sym(":=", "`:=` after the name", "expr.bind.2")?;
                let value = self.expression_plain()?;
                let otherwise = self.otherwise()?;
                Some(Stmt::Let { name, pos, mutable: true, value, otherwise })
            }
            Tok::Ident(name) if matches!(self.peek_at(1), Tok::Sym(":=")) => {
                self.i += 2;
                let value = self.expression_plain()?;
                let otherwise = self.otherwise()?;
                Some(Stmt::Let { name, pos, mutable: false, value, otherwise })
            }
            Tok::Ident(word)
                if ["let", "var", "const"].contains(&word.as_str()) && matches!(self.peek_at(1), Tok::Ident(_)) =>
            {
                let mut d = Diagnostic::new(
                    "expr.bind.1",
                    pos,
                    format!("AIL has no `{word}`; bindings are declared with `:=`"),
                )
                .help("write `name := value`, or `mut name := value` to allow reassignment");
                if matches!(self.peek_at(2), Tok::Sym("=")) {
                    let eq = self.tokens[self.i + 2].pos;
                    let first = if word == "var" {
                        Edit::span(pos, Pos { col: pos.col + 3, ..pos }, "mut")
                    } else {
                        Edit::delete(pos, self.tokens[self.i + 1].pos)
                    };
                    d = d.fix(
                        "declare it with `:=`",
                        vec![first, Edit::span(eq, Pos { col: eq.col + 1, ..eq }, ":=")],
                    );
                }
                self.diags.push(d);
                None
            }
            Tok::Kw("while") => {
                self.bump();
                let cond = self.range()?;
                let body = self.block("the `while` condition", "expr.loop.1")?;
                Some(Stmt::While { cond, body, pos })
            }
            Tok::Kw("for") => self.for_loop(false, pos),
            Tok::Kw("with") => self.with_stmt(pos),
            // conc.par.5: `par for` asserts that the iterations are independent.
            Tok::Kw("par") => {
                self.bump();
                if !matches!(self.peek(), Tok::Kw("for")) {
                    self.unexpected("`for` after `par`", "conc.par.5");
                    return None;
                }
                self.for_loop(true, pos)
            }
            Tok::Kw("break") => {
                self.bump();
                Some(Stmt::Break(pos))
            }
            Tok::Kw("continue") => {
                self.bump();
                Some(Stmt::Continue(pos))
            }
            Tok::Kw("return") => {
                self.bump();
                if matches!(self.peek(), Tok::Newline | Tok::Dedent | Tok::Eof) {
                    Some(Stmt::Return(None, pos))
                } else {
                    Some(Stmt::Return(Some(self.expression()?), pos))
                }
            }
            Tok::Kw(k) if matches!(self.peek_at(1), Tok::Sym(":=")) && NOT_YET.iter().all(|(n, _)| *n != k) => {
                self.error_help(
                    "lex.kw.1",
                    pos,
                    format!("`{k}` is a keyword and can't be used as a name"),
                    "choose a different name",
                );
                None
            }
            Tok::Kw(k @ ("elif" | "else")) => {
                self.error_help(
                    "expr.if.1",
                    pos,
                    format!("`{k}` without a matching `if`"),
                    "line it up with its `if` (or with the statement the `if` starts in)",
                );
                None
            }
            _ => {
                if self.foreign_stmt() {
                    return None;
                }
                let target = self.expression_plain()?;
                // `f(x) else ...` handles a failure with no success value
                // (err.else.5); `e else d` gives a value (err.else.6).
                if self.at_kw("else") {
                    if !self.else_block_follows()
                        && !matches!(self.peek_at(1), Tok::Kw("return" | "break" | "continue"))
                    {
                        return Some(Stmt::Expr(self.else_expr(target)?));
                    }
                    let otherwise = self.otherwise()?.expect("at `else`");
                    return Some(Stmt::Handle { value: target, otherwise });
                }
                if let Tok::Sym(s) = self.peek().clone() {
                    if let Some((_, op)) = ASSIGN_OPS.iter().find(|(sym, _)| *sym == s) {
                        let pos = self.bump().pos;
                        let value = self.expression()?;
                        return Some(Stmt::Assign { target, op: *op, value, pos });
                    }
                    if s == ":=" {
                        let pos = self.pos();
                        self.error("expr.bind.1", pos, "only a plain name can be declared with `:=`");
                        return None;
                    }
                }
                Some(Stmt::Expr(target))
            }
        }
    }

    /// The `else` part of a binding, if any (err.else.1-3). The outer `None`
    /// means a syntax error was reported.
    fn otherwise(&mut self) -> Option<Option<Else>> {
        if !self.at_kw("else") {
            return Some(None);
        }
        let pos = self.bump().pos;
        let part = match self.peek().clone() {
            Tok::Newline => Else::Block { err: None, body: self.block("`else`", "err.else.2")?, pos },
            Tok::Ident(name) if matches!(self.peek_at(1), Tok::Newline) && matches!(self.peek_at(2), Tok::Indent) => {
                let at = self.bump().pos;
                let body = self.block(&format!("`else {name}`"), "err.else.3")?;
                Else::Block { err: Some((name, at)), body, pos }
            }
            Tok::Kw("return" | "break" | "continue") => Else::Leave(Box::new(self.statement_inner()?)),
            _ => Else::Value(self.expression()?),
        };
        Some(Some(part))
    }

    // ---- Expressions, loosest binding first (expr.prec.1) ----

    /// An expression, which may end in an `else` fallback (err.else.6).
    fn expression(&mut self) -> Option<Expr> {
        let value = self.expression_plain()?;
        self.else_expr(value)
    }

    /// An expression without a trailing `else`: a binding or a statement
    /// handles its own `else`, including the block forms.
    fn expression_plain(&mut self) -> Option<Expr> {
        match self.peek() {
            Tok::Kw("if") => self.if_expr(),
            Tok::Kw("match") => self.match_expr(),
            _ => self.range(),
        }
    }

    /// The `for x in xs [if c]` clauses of a comprehension, at the first
    /// `for` (expr.comp.2).
    fn comp_clauses(&mut self) -> Option<Vec<CompFor>> {
        let mut clauses = Vec::new();
        loop {
            // conc.par.12: `[e par for x in xs]` computes the elements on the pool.
            let par = self.at_comp_for() && self.eat_kw("par");
            if !self.eat_kw("for") {
                break;
            }
            if self.at_kw("mut") {
                let pos = self.pos();
                self.error_help(
                    "expr.comp.2",
                    pos,
                    "a comprehension's variable can't be `mut`",
                    "use a `for mut` loop to change elements in place",
                );
                return None;
            }
            let (var, var_pos) = self.name("a variable after `for`", "expr.comp.2")?;
            if !self.eat_kw("in") {
                self.unexpected("`in` after the comprehension's variable", "expr.comp.2");
                return None;
            }
            let iter = self.range()?;
            let cond = if self.eat_kw("if") { Some(self.expression()?) } else { None };
            clauses.push(CompFor { par, var, var_pos, iter, cond });
        }
        Some(clauses)
    }

    /// At `for`, or `par for`, starting a comprehension clause (expr.comp.2, conc.par.12).
    fn at_comp_for(&self) -> bool {
        self.at_kw("for") || (self.at_kw("par") && matches!(self.peek_at(1), Tok::Kw("for")))
    }

    /// `with m as mut x` and its block (conc.share.6), at `with`.
    fn with_stmt(&mut self, pos: Pos) -> Option<Stmt> {
        self.bump();
        let mutex = self.range()?;
        if !self.eat_kw("as") {
            self.unexpected("`as mut NAME` after the mutex", "conc.share.6");
            return None;
        }
        if !self.eat_kw("mut") {
            let at = self.pos();
            self.error_help(
                "conc.share.6",
                at,
                "a `with` block writes its variable back to the mutex, so the variable is `mut`",
                "write `with m as mut x`; to read the value once, use `m.get()`",
            );
            return None;
        }
        let (var, var_pos) = self.name("a variable after `as mut`", "conc.share.6")?;
        let body = self.block("the `with` header", "conc.share.6")?;
        Some(Stmt::With { mutex, var, var_pos, body, pos })
    }

    /// `value else d` or `value else return ...` (err.else.6), the loosest
    /// operator. An `else` that starts a block belongs to a statement.
    fn else_expr(&mut self, value: Expr) -> Option<Expr> {
        if !self.at_kw("else") || self.else_block_follows() {
            return Some(value);
        }
        let pos = self.bump().pos;
        let otherwise = match self.peek() {
            Tok::Kw("return" | "break" | "continue") => Else::Leave(Box::new(self.statement_inner()?)),
            _ => Else::Value(self.expression()?),
        };
        Some(Expr::Else { value: Box::new(value), otherwise: Box::new(otherwise), pos })
    }

    /// At `else`: whether an indented block follows it (`else` or `else err`).
    fn else_block_follows(&self) -> bool {
        match self.peek_at(1) {
            Tok::Newline => true,
            Tok::Ident(_) => matches!(self.peek_at(2), Tok::Newline) && matches!(self.peek_at(3), Tok::Indent),
            _ => false,
        }
    }

    /// `a..b` and `a..=b` (expr.loop.3): the loosest operators. A bound may
    /// be left out before `]`, for a slice (type.coll.10).
    fn range(&mut self) -> Option<Expr> {
        let lo = if self.at_sym("..") || self.at_sym("..=") { None } else { Some(Box::new(self.or()?)) };
        let inclusive = if self.at_sym("..") {
            false
        } else if self.at_sym("..=") {
            true
        } else {
            return lo.map(|e| *e);
        };
        let pos = self.bump().pos;
        let hi = if self.at_sym("]") { None } else { Some(Box::new(self.or()?)) };
        Some(Expr::Range { lo, hi, inclusive, pos })
    }

    /// A C-style logical operator: report it and treat it as its AIL spelling.
    fn logical_typo(&mut self, sym: &str, word: &str) -> bool {
        if !self.at_sym(sym) {
            return false;
        }
        let pos = self.pos();
        // `!x` becomes `not x`; the others replace the symbol.
        let text = if sym == "!" { "not ".to_string() } else { word.to_string() };
        let end = Pos { col: pos.col + sym.len(), ..pos };
        self.error_fix(
            "expr.op.1",
            pos,
            format!("AIL writes `{sym}` as `{word}`"),
            format!("write `{word}`"),
            format!("write `{word}`"),
            vec![Edit::span(pos, end, text)],
        );
        true
    }

    fn or(&mut self) -> Option<Expr> {
        let mut lhs = self.and()?;
        while self.at_kw("or") || self.logical_typo("||", "or") {
            let pos = self.bump().pos;
            let rhs = self.and()?;
            lhs = Expr::Binary { op: BinOp::Or, lhs: Box::new(lhs), rhs: Box::new(rhs), pos };
        }
        Some(lhs)
    }

    fn and(&mut self) -> Option<Expr> {
        let mut lhs = self.not()?;
        while self.at_kw("and") || self.logical_typo("&&", "and") {
            let pos = self.bump().pos;
            let rhs = self.not()?;
            lhs = Expr::Binary { op: BinOp::And, lhs: Box::new(lhs), rhs: Box::new(rhs), pos };
        }
        Some(lhs)
    }

    /// `not` binds more loosely than comparisons (ADR 0112).
    fn not(&mut self) -> Option<Expr> {
        if self.at_kw("not") || self.logical_typo("!", "not") {
            let pos = self.bump().pos;
            let operand = self.not()?;
            return Some(Expr::Unary { op: UnOp::Not, operand: Box::new(operand), pos });
        }
        self.comparison()
    }

    fn comparison_op(&self) -> Option<BinOp> {
        match self.peek() {
            Tok::Sym(s) => COMPARISONS.iter().find(|(sym, _)| sym == s).map(|(_, op)| *op),
            _ => None,
        }
    }

    /// Comparisons don't chain (expr.prec.2).
    fn comparison(&mut self) -> Option<Expr> {
        let lhs = self.additive()?;
        let Some(op) = self.comparison_op() else {
            return Some(lhs);
        };
        let pos = self.bump().pos;
        if op == BinOp::Lt && self.at_sym(">") && self.adjacent(self.i - 1) {
            self.error_fix(
                "gram.foreign.1",
                pos,
                "AIL writes `<>` as `!=`",
                "write `!=`",
                "write `!=`",
                vec![Edit::span(pos, Pos { col: pos.col + 2, ..pos }, "!=")],
            );
            return None;
        }
        let rhs = self.additive()?;
        let expr = Expr::Binary { op, lhs: Box::new(lhs), rhs: Box::new(rhs), pos };
        if self.comparison_op().is_some() {
            let at = self.pos();
            // `List<int>()`: type arguments use `[]`.
            if let Expr::Binary { op: BinOp::Lt, lhs: name, .. } = &expr
                && matches!(&**name, Expr::Name(n, _) if n.chars().next().is_some_and(char::is_uppercase))
                && self.at_sym(">")
            {
                self.error_fix(
                    "gram.foreign.1",
                    pos,
                    "type arguments are written in `[]`, not `<>`",
                    "write `List[int]()`",
                    "use brackets",
                    vec![
                        Edit::span(pos, Pos { col: pos.col + 1, ..pos }, "["),
                        Edit::span(at, Pos { col: at.col + 1, ..at }, "]"),
                    ],
                );
                return None;
            }
            self.diags.push(
                Diagnostic::new("expr.prec.2", at, "comparisons can't be chained")
                    .help("combine them with `and`, e.g. `a < b and b < c`")
                    .fix("repeat the middle operand with `and`", vec![Edit::chain(pos, at)]),
            );
            return None;
        }
        Some(expr)
    }

    fn additive(&mut self) -> Option<Expr> {
        let mut lhs = self.multiplicative()?;
        loop {
            let op = match self.peek() {
                Tok::Sym("+") => BinOp::Add,
                Tok::Sym("-") => BinOp::Sub,
                _ => return Some(lhs),
            };
            let pos = self.bump().pos;
            // `n++` and `n--`
            if matches!((&op, self.peek()), (BinOp::Add, Tok::Sym("+")) | (BinOp::Sub, Tok::Sym("-")))
                && self.adjacent(self.i - 1)
                && self.line_ends_at(self.i)
            {
                let (sym, assign) = if op == BinOp::Add { ("++", "+=") } else { ("--", "-=") };
                self.error_fix(
                    "gram.foreign.1",
                    pos,
                    format!("AIL has no `{sym}`"),
                    format!("write `n {assign} 1`"),
                    format!("write `{assign} 1`"),
                    vec![Edit::span(pos, Pos { col: pos.col + 2, ..pos }, format!(" {assign} 1"))],
                );
                return None;
            }
            let rhs = self.multiplicative()?;
            lhs = Expr::Binary { op, lhs: Box::new(lhs), rhs: Box::new(rhs), pos };
        }
    }

    fn multiplicative(&mut self) -> Option<Expr> {
        let mut lhs = self.unary()?;
        loop {
            let op = match self.peek() {
                Tok::Sym("*") => BinOp::Mul,
                Tok::Sym("/") => BinOp::Div,
                Tok::Sym("%") => BinOp::Rem,
                _ => return Some(lhs),
            };
            let pos = self.bump().pos;
            let rhs = self.unary()?;
            lhs = Expr::Binary { op, lhs: Box::new(lhs), rhs: Box::new(rhs), pos };
        }
    }

    fn unary(&mut self) -> Option<Expr> {
        // lex.num.3: `-9223372036854775808` is the minimum `int`, unless a
        // postfix operator binds the literal first.
        if self.at_sym("-")
            && matches!(self.peek_at(1), Tok::IntMin)
            && !matches!(self.peek_at(2), Tok::Sym("." | "[" | "("))
        {
            let pos = self.bump().pos;
            self.i += 1;
            return Some(Expr::Int(i64::MIN, pos));
        }
        if self.at_sym("-") {
            let pos = self.bump().pos;
            let operand = self.unary()?;
            return Some(Expr::Unary { op: UnOp::Neg, operand: Box::new(operand), pos });
        }
        // `ref e` (own.ref.1, ADR 0067).
        if self.at_kw("ref") {
            let pos = self.bump().pos;
            let value = self.unary()?;
            return Some(Expr::Ref { value: Box::new(value), pos });
        }
        // conc.task.1: `spawn f(args)`; the checker requires the call form.
        if self.at_kw("spawn") {
            let pos = self.bump().pos;
            let operand = self.postfix()?;
            return Some(Expr::Unary { op: UnOp::Spawn, operand: Box::new(operand), pos });
        }
        self.postfix()
    }

    /// Calls, fields, methods and indexing (expr.prec.1 level 1).
    fn postfix(&mut self) -> Option<Expr> {
        let mut expr = self.primary()?;
        // `f[int](x)`, `Stack[int].new()`: type arguments after a generic name (gen.infer.2).
        if let Expr::Name(name, pos) = &expr
            && self.at_sym("[")
            && self.generic_names.contains(name)
        {
            let (name, pos) = (name.clone(), *pos);
            self.bump(); // [
            let mut args = Vec::new();
            while !self.at_sym("]") {
                args.push(self.type_ref("a type argument", "gen.infer.2")?);
                if !self.eat_sym(",") {
                    break;
                }
            }
            self.expect_sym("]", "`]` or `,` in the type arguments", "gen.infer.2")?;
            expr = Expr::TypeArgs { name, args, pos };
        }
        loop {
            if self.at_sym("(") {
                let pos = self.bump().pos;
                let args = self.args()?;
                expr = match self.old_call(&expr, args, pos) {
                    Ok(old) => old,
                    Err(args) => Expr::Call { callee: Box::new(expr), args, pos },
                };
            } else if self.eat_sym(".") {
                let (name, name_pos) = self.name("a field or method name after `.`", "gram.syntax.1")?;
                if let Some(qualified) = self.qualified_type_args(&expr, &name)? {
                    expr = qualified;
                } else if self.eat_sym("(") {
                    let args = self.args()?;
                    expr = Expr::Method { recv: Box::new(expr), name, name_pos, args };
                } else {
                    expr = Expr::Field { recv: Box::new(expr), name, name_pos };
                }
            } else if self.at_sym("[") {
                let pos = self.bump().pos;
                let index = self.range()?;
                if self.at_sym(":") {
                    let at = self.pos();
                    self.error_fix(
                        "type.coll.4",
                        at,
                        "a slice is written with `..`, not `:`",
                        "write `xs[a..b]`, `xs[a..]` or `xs[..b]`",
                        "write `..`",
                        vec![Edit::span(at, Pos { col: at.col + 1, ..at }, "..")],
                    );
                    return None;
                }
                self.expect_sym("]", "`]` after the index", "type.coll.4")?;
                expr = Expr::Index { recv: Box::new(expr), index: Box::new(index), pos };
            } else if self.at_sym("?") {
                // `e?` propagates; `e? "message"` also adds context (err.prop.1, err.ctx.2).
                let pos = self.bump().pos;
                let context = match self.peek() {
                    Tok::Str(_) | Tok::Interp(_) => Some(Box::new(self.primary()?)),
                    _ => None,
                };
                expr = Expr::Try { expr: Box::new(expr), context, pos };
            } else {
                return Some(expr);
            }
        }
    }

    /// Arguments after `(`, up to and including `)` (gram.list.1).
    fn args(&mut self) -> Option<Vec<Arg>> {
        let mut args = Vec::new();
        while !self.at_sym(")") {
            let label = if matches!(self.peek(), Tok::Ident(_)) && matches!(self.peek_at(1), Tok::Sym(":")) {
                let l = self.name("a field name", "type.struct.3")?;
                self.i += 1; // `:`
                Some(l)
            } else {
                None
            };
            let mutable = self.eat_kw("mut");
            let moved = !mutable && self.eat_kw("move");
            // expr.key.1: `by: |x| e`, a key function.
            let param = if self.eat_sym("|") {
                let p = self.name("a parameter name after `|`", "expr.key.1")?;
                self.expect_sym("|", "`|` after the key function's parameter", "expr.key.1")?;
                Some(p)
            } else {
                None
            };
            let value = self.expression()?;
            args.push(Arg { label, mutable, moved, value, param });
            if !self.eat_sym(",") {
                break;
            }
        }
        self.expect_sym(")", "`)` or `,` in the argument list", "expr.call.1")?;
        Some(args)
    }

    fn primary(&mut self) -> Option<Expr> {
        let pos = self.pos();
        match self.peek().clone() {
            Tok::Str(s) => {
                self.i += 1;
                Some(Expr::Str(s, pos))
            }
            Tok::Interp(parts) => {
                self.i += 1;
                self.interpolation(parts, pos)
            }
            Tok::Int(v) => {
                self.i += 1;
                Some(Expr::Int(v, pos))
            }
            Tok::IntMin => {
                self.i += 1;
                self.int_too_big(pos);
                Some(Expr::Int(0, pos))
            }
            Tok::Float(v) => {
                self.i += 1;
                Some(Expr::Float(v, pos))
            }
            Tok::Char(c) => {
                self.i += 1;
                Some(Expr::Char(c, pos))
            }
            Tok::Kw("true") => {
                self.i += 1;
                Some(Expr::Bool(true, pos))
            }
            Tok::Kw("false") => {
                self.i += 1;
                Some(Expr::Bool(false, pos))
            }
            Tok::Kw("none") => {
                self.i += 1;
                Some(Expr::None(pos))
            }
            Tok::Kw("todo") => {
                self.i += 1;
                Some(Expr::Hole(pos))
            }
            Tok::Kw("self") => {
                self.i += 1;
                Some(Expr::Name("self".into(), pos))
            }
            Tok::Ident(name) => {
                self.i += 1;
                Some(Expr::Name(name, pos))
            }
            Tok::Sym("(") => {
                self.i += 1;
                let inner = self.range()?;
                let inner = self.else_expr(inner)?;
                self.expect_sym(")", "`)`", "gram.syntax.1")?;
                Some(inner)
            }
            Tok::Sym("[") => {
                self.i += 1;
                let mut elems = Vec::new();
                while !self.at_sym("]") {
                    elems.push(self.expression()?);
                    if elems.len() == 1 && self.at_comp_for() {
                        let elem = Box::new(elems.pop()?);
                        let clauses = self.comp_clauses()?;
                        self.expect_sym("]", "`]` after the comprehension", "expr.comp.1")?;
                        return Some(Expr::Comp { kind: CompKind::List, elem, value: None, clauses, pos });
                    }
                    if !self.eat_sym(",") {
                        break;
                    }
                }
                self.expect_sym("]", "`]` or `,` in the list", "type.coll.2")?;
                Some(Expr::List(elems, pos))
            }
            // `{k: v, ...}`, `{a, b, ...}`, or `{}` (type.coll.2).
            Tok::Sym("{") => {
                self.i += 1;
                if self.eat_sym("}") {
                    return Some(Expr::Map(Vec::new(), pos));
                }
                let first = self.expression()?;
                if self.eat_sym(":") {
                    let value = self.expression()?;
                    if self.at_comp_for() {
                        let clauses = self.comp_clauses()?;
                        self.expect_sym("}", "`}` after the comprehension", "expr.comp.1")?;
                        let (elem, value) = (Box::new(first), Some(Box::new(value)));
                        return Some(Expr::Comp { kind: CompKind::Map, elem, value, clauses, pos });
                    }
                    let mut pairs = vec![(first, value)];
                    while self.eat_sym(",") && !self.at_sym("}") {
                        let k = self.expression()?;
                        self.expect_sym(":", "`:` after the key", "type.coll.2")?;
                        pairs.push((k, self.expression()?));
                    }
                    self.expect_sym("}", "`}` or `,` in the map", "type.coll.2")?;
                    return Some(Expr::Map(pairs, pos));
                }
                if self.at_comp_for() {
                    let clauses = self.comp_clauses()?;
                    self.expect_sym("}", "`}` after the comprehension", "expr.comp.1")?;
                    return Some(Expr::Comp { kind: CompKind::Set, elem: Box::new(first), value: None, clauses, pos });
                }
                let mut elems = vec![first];
                while self.eat_sym(",") && !self.at_sym("}") {
                    elems.push(self.expression()?);
                }
                self.expect_sym("}", "`}` or `,` in the set", "type.coll.2")?;
                Some(Expr::Set(elems, pos))
            }
            Tok::Kw(k @ ("if" | "match")) => {
                self.error_help(
                    "gram.syntax.1",
                    pos,
                    format!("`{k}` can't be part of a larger expression"),
                    format!("bind the `{k}` to a name first, then use the name"),
                );
                None
            }
            _ => {
                self.unexpected("an expression", "gram.syntax.1");
                None
            }
        }
    }

    /// Parse each `{...}` part of an interpolated string (lex.str.2).
    fn interpolation(&mut self, parts: Vec<Part>, pos: Pos) -> Option<Expr> {
        let mut out = Vec::new();
        let mut ok = true;
        for part in parts {
            let part_spec = if let Part::Fmt(_, f) = &part { Some(f.clone()) } else { None };
            match part {
                Part::Lit(s) => out.push(InterpPart::Lit(s)),
                // lex.str.2: `{{` isn't an escape; `\{` is.
                Part::Code(tokens) if matches!(tokens.first().map(|t| &t.tok), Some(Tok::Sym("{"))) => {
                    let inner = tokens[0].pos;
                    let mut edits = vec![Edit::span(Pos { col: inner.col - 1, ..inner }, Pos { col: inner.col + 1, ..inner }, "\\{")];
                    // `{{x}}`: the last token is the inner `}`, then the outer one.
                    if let Some(Token { tok: Tok::Sym("}"), pos: last }) = tokens.iter().rev().nth(1) {
                        edits.push(Edit::delete(Pos { col: last.col + 1, ..*last }, Pos { col: last.col + 2, ..*last }));
                    }
                    self.diags.push(
                        Diagnostic::new("lex.str.2", inner, "`{{` in a string isn't an escaped brace")
                            .help("write `\\{` for a literal `{`, or use a raw string `r\"...\"`")
                            .fix("escape the brace with a backslash", edits),
                    );
                    ok = false;
                }
                Part::Code(tokens) | Part::Fmt(tokens, _) => {
                    let spec = part_spec;
                    let generic_names = self.generic_names.clone();
                    let mut sub = Parser {
                        tokens,
                        i: 0,
                        diags: Vec::new(),
                        in_interp: true,
                        generic_names,
                        olds: None,
                        aliases: self.aliases.clone(),
                        public: HashSet::new(),
                    };
                    let expr = sub.expression();
                    if expr.is_some() && !matches!(sub.peek(), Tok::Eof) {
                        sub.unexpected("`}` after the interpolated expression", "lex.str.2");
                    }
                    let clean = sub.diags.is_empty();
                    self.diags.append(&mut sub.diags);
                    match expr {
                        Some(e) if clean => out.push(match spec {
                            Some(f) => InterpPart::Fmt(e, f),
                            None => InterpPart::Expr(e),
                        }),
                        _ => ok = false,
                    }
                }
            }
        }
        ok.then_some(Expr::Interp(out, pos))
    }

    /// `if c` block, any `elif c` blocks, optional `else` block (expr.if.1, expr.if.3).
    fn if_expr(&mut self) -> Option<Expr> {
        let pos = self.bump().pos;
        let cond = self.condition()?;
        if matches!(self.peek(), Tok::Ident(t) if t == "then") {
            return self.if_then(cond, pos);
        }
        let body = self.block("the `if` condition", "expr.if.1")?;
        let mut branches = vec![(cond, body)];
        let mut els = None;
        let mut else_err = None;
        loop {
            if self.eat_kw("elif") {
                let cond = self.condition()?;
                let body = self.block("the `elif` condition", "expr.if.1")?;
                branches.push((cond, body));
            } else if self.at_kw("else") {
                let else_pos = self.bump().pos;
                if self.at_kw("if") {
                    let d = Diagnostic::new("expr.if.1", else_pos, "AIL writes `else if` as `elif`")
                        .help("write `elif`")
                        .fix("write `elif`", vec![Edit::replace(else_pos, "else if", "elif")]);
                    self.diags.push(d);
                    return None;
                }
                // `else err` binds the error (expr.if.6).
                if let Tok::Ident(name) = self.peek().clone()
                    && matches!(self.peek_at(1), Tok::Newline)
                {
                    else_err = Some((name, self.bump().pos));
                }
                els = Some(self.block("`else`", "expr.if.1")?);
                break;
            } else {
                break;
            }
        }
        Some(Expr::If { branches, els, else_err, pos })
    }

    /// expr.if.8: the one-line form `if c then a else b`, after `c`. `then`
    /// is a keyword only here, so it stays usable as a name.
    fn if_then(&mut self, cond: Cond, pos: Pos) -> Option<Expr> {
        self.bump(); // then
        // Both values are plain: the `else` here is the `if`'s, not err.else.6.
        let a = self.expression_plain()?;
        if !self.eat_kw("else") {
            let at = self.pos();
            self.error_help(
                "expr.if.8",
                at,
                "`if c then a` needs `else b` on the same line",
                "add `else` and the other value; for a statement with no `else`, put the body on its own line",
            );
            return None;
        }
        let b = self.expression_plain()?;
        Some(Expr::If { branches: vec![(cond, vec![Stmt::Expr(a)])], els: Some(vec![Stmt::Expr(b)]), else_err: None, pos })
    }

    /// A `bool` condition, or a binding `name := e` (expr.if.5).
    fn condition(&mut self) -> Option<Cond> {
        if let Tok::Ident(name) = self.peek().clone()
            && matches!(self.peek_at(1), Tok::Sym(":="))
        {
            let pos = self.bump().pos;
            self.i += 1; // `:=`
            return Some(Cond::Bind { name, pos, value: self.range()? });
        }
        Some(Cond::Expr(self.range()?))
    }

    /// `match e` followed by one `pattern => result` arm per line (expr.match.1).
    fn match_expr(&mut self) -> Option<Expr> {
        let pos = self.bump().pos;
        let scrutinee = self.range()?;
        if !self.eat(&Tok::Newline) {
            self.unexpected("end of line after the `match` value", "expr.match.1");
            return None;
        }
        if !self.eat(&Tok::Indent) {
            let at = self.pos();
            self.error_help(
                "expr.match.1",
                at,
                "a `match` needs an indented block of arms",
                "write one `pattern => result` per line",
            );
            return None;
        }
        let mut arms = Vec::new();
        while !matches!(self.peek(), Tok::Dedent | Tok::Eof) {
            if self.eat(&Tok::Newline) {
                continue;
            }
            match self.arm() {
                Some(arm) => arms.push(arm),
                None => self.skip_statement(),
            }
        }
        self.eat(&Tok::Dedent);
        Some(Expr::Match { scrutinee: Box::new(scrutinee), arms, pos })
    }

    fn arm(&mut self) -> Option<Arm> {
        let pat = self.pattern()?;
        if !self.eat_sym("=>") {
            if self.at_sym(":") || self.at_sym("->") {
                let at = self.pos();
                let len = if self.at_sym(":") { 1 } else { 2 };
                // `Red: 1` needs a blank before the arrow; `Red -> 1` has one.
                let attached = self.tok_end(self.i - 1) == Some(at);
                self.error_fix(
                    "expr.match.1",
                    at,
                    "arms are written `pattern => result`",
                    "write `=>`",
                    "write `=>`",
                    vec![Edit::span(at, Pos { col: at.col + len, ..at }, if attached { " =>" } else { "=>" })],
                );
                self.i += 1; // go on as if it were `=>`
            } else {
                self.unexpected("`=>` after the pattern", "expr.match.1");
                return None;
            }
        }
        if matches!(self.peek(), Tok::Newline) {
            let body = self.block("`=>`", "expr.match.1")?;
            return Some(Arm { pat, body });
        }
        // expr.match.9: one statement on the arm's line, such as a value,
        // `return x`, `continue` or `total += n`.
        let stmt = self.statement_inner()?;
        if !self.after_block() && !self.eat(&Tok::Newline) {
            self.unexpected("end of line after the arm", "expr.match.1");
            return None;
        }
        Some(Arm { pat, body: vec![stmt] })
    }

    /// A pattern, or several joined by `|` (expr.match.8).
    fn pattern(&mut self) -> Option<Pattern> {
        let pos = self.pos();
        let first = self.single_pattern()?;
        if !self.at_sym("|") {
            return Some(first);
        }
        let mut alts = vec![first];
        while self.eat_sym("|") {
            alts.push(self.single_pattern()?);
        }
        Some(Pattern::Or(alts, pos))
    }

    /// `_`, `Variant`, `Enum.Variant`, or a literal (expr.match.4, expr.match.7).
    fn single_pattern(&mut self) -> Option<Pattern> {
        let pos = self.pos();
        match self.peek().clone() {
            Tok::Ident(n) if n == "_" => {
                self.i += 1;
                Some(Pattern::Wildcard(pos))
            }
            Tok::Ident(n) => {
                self.i += 1;
                // `Enum.Variant`, or `m.Enum.Variant` for an imported enum (mod.use.2).
                let mut path = vec![n];
                while self.eat_sym(".") {
                    path.push(self.name("a variant name after `.`", "expr.match.4")?.0);
                }
                let name = path.pop().expect("a name");
                let qualifier = (!path.is_empty()).then(|| path.join("."));
                // `Variant(a, _)`: one binding per field (expr.match.6).
                let mut bindings = None;
                if self.eat_sym("(") {
                    let mut names = Vec::new();
                    while !self.at_sym(")") {
                        names.push(self.name("a name or `_` for the field", "expr.match.6")?);
                        if !self.eat_sym(",") {
                            break;
                        }
                    }
                    self.expect_sym(")", "`)` or `,` in the pattern", "expr.match.6")?;
                    bindings = Some(names);
                }
                Some(Pattern::Variant { qualifier, name, bindings, pos })
            }
            Tok::Int(v) => {
                self.i += 1;
                Some(Pattern::Literal(Literal::Int(v), pos))
            }
            Tok::Sym("-") if matches!(self.peek_at(1), Tok::Int(_) | Tok::IntMin) => {
                self.i += 1;
                let v = match self.bump().tok {
                    Tok::Int(v) => -v,
                    _ => i64::MIN,
                };
                Some(Pattern::Literal(Literal::Int(v), pos))
            }
            Tok::IntMin => {
                self.i += 1;
                self.int_too_big(pos);
                Some(Pattern::Literal(Literal::Int(0), pos))
            }
            Tok::Char(c) => {
                self.i += 1;
                Some(Pattern::Literal(Literal::Char(c), pos))
            }
            Tok::Str(s) => {
                self.i += 1;
                Some(Pattern::Literal(Literal::Str(s), pos))
            }
            Tok::Kw(b @ ("true" | "false")) => {
                self.i += 1;
                Some(Pattern::Literal(Literal::Bool(b == "true"), pos))
            }
            Tok::Interp(_) | Tok::Float(_) => {
                self.error_help(
                    "expr.match.7",
                    pos,
                    "a pattern can be an integer, character, string or `bool` literal",
                    "compare with `==` in an `if` instead",
                );
                None
            }
            _ => {
                self.unexpected("a pattern (a variant name or `_`)", "expr.match.4");
                None
            }
        }
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

    fn body(src: &str) -> Block {
        let (mut prog, rules) = parse_src(src);
        assert!(rules.is_empty(), "{rules:?}");
        prog.fns.remove(0).body.expect("a body")
    }

    #[test]
    fn parses_hello_world() {
        let (prog, rules) = parse_src("fn main()\n    print(\"Hello, world!\")\n");
        assert!(rules.is_empty());
        assert_eq!(prog.fns.len(), 1);
        assert_eq!(prog.fns[0].name, "main");
        assert_eq!(prog.fns[0].body.as_ref().map(Vec::len), Some(1));
    }

    #[test]
    fn parses_params_return_type_and_bindings() {
        let (prog, rules) = parse_src("fn greet(name str) str\n    greeting := name\n    greeting\n");
        assert!(rules.is_empty(), "{rules:?}");
        let f = &prog.fns[0];
        assert_eq!(f.params.len(), 1);
        assert!(matches!(&f.ret.as_ref().unwrap().kind, TypeKind::Named { name, .. } if name == "str"));
        assert!(matches!(f.body.as_ref().unwrap()[0], Stmt::Let { .. }));
    }

    #[test]
    fn precedence_follows_adr_0112() {
        let b = body("fn main()\n    x := not 1 + 2 * 3 == 7 and true\n");
        let Stmt::Let { value, .. } = &b[0] else { panic!() };
        // and(not(==( +(1, *(2,3)), 7)), true)
        let Expr::Binary { op: BinOp::And, lhs, .. } = value else { panic!("{value:?}") };
        let Expr::Unary { op: UnOp::Not, operand, .. } = &**lhs else { panic!("{lhs:?}") };
        let Expr::Binary { op: BinOp::Eq, lhs, .. } = &**operand else { panic!() };
        let Expr::Binary { op: BinOp::Add, rhs, .. } = &**lhs else { panic!() };
        assert!(matches!(**rhs, Expr::Binary { op: BinOp::Mul, .. }));
    }

    #[test]
    fn if_expression_layout_follows_adr_0111() {
        let b = body("fn main()\n    k := if a\n        1\n    elif b\n        2\n    else\n        3\n    print(k)\n");
        assert_eq!(b.len(), 2);
        let Stmt::Let { value: Expr::If { branches, els, .. }, .. } = &b[0] else { panic!() };
        assert_eq!(branches.len(), 2);
        assert!(els.is_some());
    }

    #[test]
    fn match_arms_inline_and_block() {
        let b = body(
            "fn main()\n    match g\n        A => 1\n        Grade.B =>\n            x := 2\n            x\n        _ => 3\n",
        );
        let Stmt::Expr(Expr::Match { arms, .. }) = &b[0] else { panic!() };
        assert_eq!(arms.len(), 3);
        assert_eq!(arms[1].body.len(), 2);
        assert!(matches!(arms[2].pat, Pattern::Wildcard(_)));
    }

    #[test]
    fn declarations_methods_and_literals() {
        let src = "struct P\n    x int\nenum E\n    A\n    B\nfn P.get(self) int\n    self.x\nfn main()\n    ps := [\n        P(x: 1),\n    ]\n    mut q := ps[0]\n    q.x += 1\n    print(\"{q.get()}\")\n";
        let (prog, rules) = parse_src(src);
        assert!(rules.is_empty(), "{rules:?}");
        assert_eq!((prog.structs.len(), prog.enums.len(), prog.fns.len()), (1, 1, 2));
        assert!(prog.fns[0].receiver.is_some());
        assert!(matches!(prog.fns[1].body.as_ref().unwrap()[2], Stmt::Assign { op: Some(BinOp::Add), .. }));
    }

    #[test]
    fn generics_traits_and_impls() {
        let src = "struct Stack[T]\n    items List[T]\nfn Stack[T Ord].max(self) T?\n    none\ntrait Shape\n    fn area(self) int\n    fn name(self) str\n        \"s\"\nimpl Shape for Stack\n    fn area(self) int\n        0\nimpl Ord for Stack[T Ord]\nfn main()\n    s := Stack[int].new()\n    x := f[int](s)\n";
        let (prog, rules) = parse_src(src);
        assert!(rules.is_empty(), "{rules:?}");
        assert_eq!(prog.structs[0].generics.len(), 1);
        let max = &prog.fns[0];
        assert_eq!(max.owner.as_ref().and_then(|o| o.params.as_ref()).map(|p| p[0].bounds.len()), Some(1));
        assert_eq!(prog.traits[0].methods.len(), 2);
        assert!(prog.fns[prog.traits[0].methods[0]].body.is_none());
        assert!(prog.fns[prog.traits[0].methods[1]].body.is_some());
        assert_eq!((prog.impls.len(), prog.impls[1].methods.len()), (2, 0));
        let main = prog.fns.last().unwrap().body.as_ref().unwrap();
        let Stmt::Let { value: Expr::Method { recv, .. }, .. } = &main[0] else { panic!("{:?}", main[0]) };
        assert!(matches!(&**recv, Expr::TypeArgs { name, .. } if name == "Stack"));
    }

    #[test]
    fn unsupported_keywords_are_reported_precisely() {
        let (_, rules) = parse_src("fn main()\n    unsafe x\n");
        assert_eq!(rules, vec![UNSUPPORTED]);
    }

    #[test]
    fn rust_style_signatures_get_precise_fixes() {
        let (_, rules) = parse_src("fn f(a: int) -> int\n    a\n");
        assert_eq!(rules, vec!["gram.fn.2", "gram.fn.2"]);
    }

    #[test]
    fn foreign_syntax_gets_precise_fixes() {
        assert_eq!(parse_src("fn main()\n    x := a < b < c\n").1, vec!["expr.prec.2"]);
        assert_eq!(parse_src("fn main()\n    x := a && b\n").1, vec!["expr.op.1"]);
        assert_eq!(parse_src("fn main()\n    let x = 1\n").1, vec!["expr.bind.1"]);
        assert_eq!(parse_src("fn main()\n    if := 1\n").1, vec!["lex.kw.1"]);
        assert_eq!(parse_src("fn main()\n    if a\n        1\n    else if b\n        2\n").1, vec!["expr.if.1"]);
    }
}
