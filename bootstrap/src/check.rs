//! Checker: names, bindings, calls and types for the supported subset.
//!
//! Rules enforced: mod.main.1, mod.main.3, mod.unit.1, expr.name.1,
//! expr.bind.1, expr.bind.4, expr.bind.5, expr.block.2, expr.call.1-2.
//! All errors are collected in one pass (ai-author-wishlist 2.4).

use std::collections::HashMap;

use crate::ast::{Expr, FnDecl, Program, Stmt, TypeRef};
use crate::diag::{Diagnostic, Pos};
use crate::lexer::UNSUPPORTED;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ty {
    Str,
    Int,
    /// The result of an expression that produces no value (e.g. `print(...)`).
    Unit,
}

impl Ty {
    fn name(self) -> &'static str {
        match self {
            Ty::Str => "str",
            Ty::Int => "int",
            Ty::Unit => "no value",
        }
    }
}

/// Names available without import (mod.main.3, ADR 0084).
const PRELUDE: &[&str] = &["print"];

/// Type names the spec defines that the bootstrap doesn't support yet.
const SPEC_TYPES_NOT_YET: &[&str] = &[
    "i8", "i16", "i32", "u8", "u16", "u32", "u64", "f32", "f64", "float", "char", "bool", "List", "Map", "Set",
];

pub fn check(prog: &Program) -> Vec<Diagnostic> {
    let mut c = Checker { diags: Vec::new(), fns: HashMap::new() };
    c.collect(prog);
    c.check_main(prog);
    for f in &prog.fns {
        c.check_fn(f);
    }
    c.diags
}

struct Sig {
    params: Vec<Option<Ty>>,
    ret: Option<Ty>,
}

struct Checker {
    diags: Vec<Diagnostic>,
    fns: HashMap<String, Sig>,
}

impl Checker {
    fn collect(&mut self, prog: &Program) {
        for f in &prog.fns {
            if PRELUDE.contains(&f.name.as_str()) {
                self.diags.push(Diagnostic::new(
                    UNSUPPORTED,
                    f.pos,
                    format!("redeclaring the prelude function `{}` is not supported by the bootstrap", f.name),
                ));
                continue;
            }
            if self.fns.contains_key(&f.name) {
                self.diags.push(Diagnostic::new(
                    "mod.unit.1",
                    f.pos,
                    format!("`{}` is declared more than once in this module", f.name),
                ));
                continue;
            }
            let params = f.params.iter().map(|p| self.resolve_type(&p.ty)).collect();
            let ret = f.ret.as_ref().and_then(|t| self.resolve_type(t));
            self.fns.insert(f.name.clone(), Sig { params, ret });
        }
    }

    fn check_main(&mut self, prog: &Program) {
        match prog.fns.iter().find(|f| f.name == "main") {
            None => self.diags.push(
                Diagnostic::new("mod.main.1", Pos { line: 1, col: 1 }, "program has no `main` function")
                    .help("add:\n    fn main()\n        print(\"Hello, world!\")"),
            ),
            Some(f) => {
                if !f.params.is_empty() {
                    self.diags.push(Diagnostic::new(
                        "mod.main.1",
                        f.params[0].pos,
                        "`main` takes no parameters",
                    ));
                }
                if let Some(ret) = &f.ret {
                    self.diags.push(Diagnostic::new(
                        "mod.main.1",
                        ret.pos,
                        format!("`main` can't return `{}`; it returns nothing (or an error with `!`)", ret.name),
                    ));
                }
            }
        }
    }

    fn resolve_type(&mut self, t: &TypeRef) -> Option<Ty> {
        match t.name.as_str() {
            "str" => Some(Ty::Str),
            "int" | "i64" => Some(Ty::Int),
            n if SPEC_TYPES_NOT_YET.contains(&n) => {
                self.diags.push(Diagnostic::new(
                    UNSUPPORTED,
                    t.pos,
                    format!("type `{n}` is not supported by the bootstrap yet (spec 04-types)"),
                ));
                None
            }
            n => {
                let mut d = Diagnostic::new("expr.name.1", t.pos, format!("unknown type `{n}`"));
                if let Some(s) = suggest(n, &["str", "int"]) {
                    d = d.help(format!("did you mean `{s}`?"));
                }
                self.diags.push(d);
                None
            }
        }
    }

    fn check_fn(&mut self, f: &FnDecl) {
        let mut scope: Vec<(String, Option<Ty>)> = Vec::new();
        for (i, p) in f.params.iter().enumerate() {
            let ty = self.fns.get(&f.name).and_then(|s| s.params.get(i).copied().flatten());
            if self.declare(&scope, &p.name, p.pos) {
                scope.push((p.name.clone(), ty));
            }
        }
        let mut last: Option<Option<Ty>> = None;
        for stmt in &f.body {
            match stmt {
                Stmt::Let { name, pos, value } => {
                    let ty = self.expr(value, &scope);
                    if ty == Some(Ty::Unit) {
                        self.diags.push(Diagnostic::new(
                            "expr.bind.1",
                            value.pos(),
                            format!("`{name}` would be bound to an expression that produces no value"),
                        ));
                    }
                    if self.declare(&scope, name, *pos) {
                        scope.push((name.clone(), ty));
                    }
                    last = None;
                }
                Stmt::Expr(e) => last = Some(self.expr(e, &scope)),
            }
        }
        if let Some(ret) = &f.ret {
            let want = self.fns.get(&f.name).and_then(|s| s.ret);
            match (last, want) {
                (None, Some(want)) => self.diags.push(Diagnostic::new(
                    "expr.block.2",
                    ret.pos,
                    format!("`{}` must end with an expression of type `{}`", f.name, want.name()),
                )),
                (Some(Some(got)), Some(want)) if got != want => {
                    let at = match f.body.last() {
                        Some(Stmt::Expr(e)) => e.pos(),
                        _ => ret.pos,
                    };
                    self.diags.push(Diagnostic::new(
                        "expr.block.2",
                        at,
                        format!("`{}` returns `{}`, but its body ends with {}", f.name, want.name(), got.name()),
                    ));
                }
                _ => {}
            }
        }
    }

    /// Check a new local name against no-shadowing rules; true if it can be added.
    fn declare(&mut self, scope: &[(String, Option<Ty>)], name: &str, pos: Pos) -> bool {
        if scope.iter().any(|(n, _)| n == name) {
            self.diags.push(
                Diagnostic::new("expr.bind.4", pos, format!("`{name}` is already declared in this function"))
                    .help("choose a different name; names can't be shadowed"),
            );
            return false;
        }
        if self.fns.contains_key(name) || PRELUDE.contains(&name) {
            self.diags.push(
                Diagnostic::new("expr.bind.5", pos, format!("`{name}` is already a function name"))
                    .help("choose a different name for the local"),
            );
            return false;
        }
        true
    }

    /// Type of an expression; `None` when it couldn't be determined (an error was reported).
    fn expr(&mut self, e: &Expr, scope: &[(String, Option<Ty>)]) -> Option<Ty> {
        match e {
            Expr::Str(..) => Some(Ty::Str),
            Expr::Int(..) => Some(Ty::Int),
            Expr::Name(name, pos) => {
                if let Some((_, ty)) = scope.iter().find(|(n, _)| n == name) {
                    return *ty;
                }
                if self.fns.contains_key(name) || PRELUDE.contains(&name.as_str()) {
                    self.diags.push(Diagnostic::new(
                        UNSUPPORTED,
                        *pos,
                        format!("using the function `{name}` as a value is not supported by the bootstrap yet"),
                    ));
                    return None;
                }
                self.unknown_name(name, *pos, scope);
                None
            }
            Expr::Call { callee, args, pos } => {
                let arg_tys: Vec<Option<Ty>> = args.iter().map(|a| self.expr(a, scope)).collect();
                let Expr::Name(name, name_pos) = callee.as_ref() else {
                    self.diags.push(Diagnostic::new(UNSUPPORTED, *pos, "calling the result of a call is not supported by the bootstrap yet"));
                    return None;
                };
                if name == "print" {
                    return self.print_call(args, &arg_tys, *name_pos);
                }
                if scope.iter().any(|(n, _)| n == name) {
                    self.diags.push(Diagnostic::new("expr.call.1", *name_pos, format!("`{name}` is a value, not a function")));
                    return None;
                }
                let Some(sig) = self.fns.get(name) else {
                    self.unknown_name(name, *name_pos, scope);
                    return None;
                };
                let (params, ret) = (sig.params.clone(), sig.ret);
                if params.len() != args.len() {
                    self.diags.push(Diagnostic::new(
                        "expr.call.1",
                        *name_pos,
                        format!("`{name}` takes {}, but {} given", count(params.len(), "argument"), was_were(args.len())),
                    ));
                } else {
                    for ((arg, got), want) in args.iter().zip(&arg_tys).zip(&params) {
                        if let (Some(got), Some(want)) = (got, want) {
                            if got != want {
                                self.diags.push(Diagnostic::new(
                                    "expr.call.2",
                                    arg.pos(),
                                    format!("expected `{}`, found {}", want.name(), got.name()),
                                ));
                            }
                        }
                    }
                }
                Some(ret.unwrap_or(Ty::Unit))
            }
        }
    }

    fn print_call(&mut self, args: &[Expr], arg_tys: &[Option<Ty>], pos: Pos) -> Option<Ty> {
        if args.len() != 1 {
            self.diags.push(Diagnostic::new(
                "mod.main.3",
                pos,
                format!("`print` takes exactly one argument, but {} given", was_were(args.len())),
            ));
        } else if arg_tys[0] == Some(Ty::Unit) {
            self.diags.push(Diagnostic::new("mod.main.3", args[0].pos(), "this expression produces no value to print"));
        }
        Some(Ty::Unit)
    }

    fn unknown_name(&mut self, name: &str, pos: Pos, scope: &[(String, Option<Ty>)]) {
        let candidates: Vec<&str> = scope
            .iter()
            .map(|(n, _)| n.as_str())
            .chain(self.fns.keys().map(String::as_str))
            .chain(PRELUDE.iter().copied())
            .collect();
        let mut d = Diagnostic::new("expr.name.1", pos, format!("unknown name `{name}`"));
        if let Some(s) = suggest(name, &candidates) {
            d = d.help(format!("did you mean `{s}`?"));
        }
        self.diags.push(d);
    }
}

fn count(n: usize, noun: &str) -> String {
    if n == 1 { format!("1 {noun}") } else { format!("{n} {noun}s") }
}

fn was_were(n: usize) -> String {
    if n == 1 { "1 was".into() } else { format!("{n} were") }
}

/// Closest candidate by edit distance, if it's close enough to be a likely typo.
fn suggest<'a>(name: &str, candidates: &[&'a str]) -> Option<&'a str> {
    let limit = (name.chars().count() / 3).max(1);
    candidates
        .iter()
        .map(|c| (edit_distance(name, c), *c))
        .filter(|(d, _)| *d <= limit)
        .min()
        .map(|(_, c)| c)
}

fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for i in 1..=a.len() {
        let mut cur = vec![i; b.len() + 1];
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            cur[j] = (prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + cost);
        }
        prev = cur;
    }
    prev[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{lexer::lex, parser::parse};

    fn rules(src: &str) -> Vec<&'static str> {
        let (tokens, ld) = lex(src);
        assert!(ld.is_empty(), "{ld:?}");
        let (prog, pd) = parse(tokens);
        assert!(pd.is_empty(), "{pd:?}");
        let mut r: Vec<_> = check(&prog).into_iter().map(|d| d.rule).collect();
        r.sort();
        r
    }

    #[test]
    fn hello_world_is_clean() {
        assert!(rules("fn main()\n    print(\"Hello, world!\")\n").is_empty());
    }

    #[test]
    fn missing_main() {
        assert_eq!(rules("fn start()\n    print(1)\n"), vec!["mod.main.1"]);
    }

    #[test]
    fn unknown_name_suggests() {
        let (tokens, _) = lex("fn main()\n    prnt(\"x\")\n");
        let (prog, _) = parse(tokens);
        let diags = check(&prog);
        assert_eq!(diags[0].rule, "expr.name.1");
        assert_eq!(diags[0].help.as_deref(), Some("did you mean `print`?"));
    }

    #[test]
    fn no_shadowing() {
        assert_eq!(rules("fn main()\n    x := 1\n    x := 2\n"), vec!["expr.bind.4"]);
        assert_eq!(rules("fn main()\n    main := 1\n"), vec!["expr.bind.5"]);
    }

    #[test]
    fn calls_check_arity_and_types() {
        let src = "fn greet(name str) str\n    name\nfn main()\n    print(greet(1))\n    print(greet())\n";
        assert_eq!(rules(src), vec!["expr.call.1", "expr.call.2"]);
    }

    #[test]
    fn return_type_must_match() {
        assert_eq!(rules("fn f() str\n    1\nfn main()\n    print(f())\n"), vec!["expr.block.2"]);
    }

    #[test]
    fn print_needs_one_value() {
        assert_eq!(rules("fn main()\n    print(print(1))\n"), vec!["mod.main.3"]);
        assert_eq!(rules("fn main()\n    print(1, 2)\n"), vec!["mod.main.3"]);
    }

    #[test]
    fn edit_distance_basics() {
        assert_eq!(edit_distance("prnt", "print"), 1);
        assert_eq!(edit_distance("", "abc"), 3);
    }
}
