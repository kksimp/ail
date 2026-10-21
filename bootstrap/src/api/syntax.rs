//! `ail api --tokens FILE` and `ail api --ast FILE`: the lexer's tokens and
//! the parser's syntax tree for one file, as text the self-hosted front end
//! (`compiler/`, ADR 0371) reproduces byte for byte.
//!
//! Spec rules: api.tok.1-3 (the token listing), api.ast.1-3 (the tree as
//! JSON). See `docs/design/semantic-api.md` and ADR 0371.
//!
//! Both read the file alone: no prelude, no imported modules, and none of
//! the passes after parsing (implicit `self`, inherited contracts, derive
//! output, the standard declarations). The diagnostics are the lexer's,
//! then the parser's, in the order they were found, before
//! one-per-line filtering (api.tok.3, api.ast.1).

use std::fmt::Write as _;
use std::process::ExitCode;

use crate::ast::*;
use crate::diag::{Diagnostic, Pos};
use crate::lexer::{self, FormatSpec, Part, Tok, Token};

/// `ail api --tokens FILE` or `ail api --ast FILE` (api.tok.1, api.ast.1).
pub fn command(args: &[&str]) -> ExitCode {
    let ast = args.contains(&"--ast");
    let whole = args.contains(&"--program");
    let files: Vec<&&str> = args.iter().filter(|a| !a.starts_with("--")).collect();
    let known = args.iter().all(|a| !a.starts_with("--") || matches!(*a, "--ast" | "--tokens" | "--program"));
    let ([path], true, true) = (&files[..], known, ast || !whole) else {
        eprint!("{}", super::USAGE);
        return ExitCode::from(2);
    };
    if whole {
        return match program_text(path) {
            Ok(out) => {
                print!("{out}");
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("error: {e}");
                ExitCode::from(2)
            }
        };
    }
    let src = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: can't read {path}: {e}");
            return ExitCode::from(2);
        }
    };
    print!("{}", if ast { ast_text(&src) } else { tokens_text(&src) });
    ExitCode::SUCCESS
}

// ---- Tokens (api.tok.1-3) ----

/// One line per token, then one per comment, then one per diagnostic.
pub fn tokens_text(src: &str) -> String {
    let (tokens, diags, comments) = lexer::lex_with_comments(src);
    let mut out = String::new();
    for t in &tokens {
        token_line(t, &mut out);
    }
    for c in &comments {
        let _ = writeln!(out, "{}:{} comment {}", c.pos.line, c.pos.col, json_text(&c.text));
    }
    diag_lines(&diags, &mut out);
    out
}

fn diag_lines(diags: &[Diagnostic], out: &mut String) {
    for d in diags {
        let _ = writeln!(out, "{}:{} diag {}", d.pos.line, d.pos.col, d.rule);
    }
}

/// `LINE:COL KIND [VALUE]` (api.tok.2); an interpolated string lists its
/// parts on the lines after it, each code part's tokens ending in `eof`,
/// and then `end`.
fn token_line(t: &Token, out: &mut String) {
    let _ = write!(out, "{}:{} ", t.pos.line, t.pos.col);
    match &t.tok {
        Tok::Ident(n) => {
            let _ = writeln!(out, "id {n}");
        }
        Tok::Kw(k) => {
            let _ = writeln!(out, "kw {k}");
        }
        Tok::Sym(s) => {
            let _ = writeln!(out, "sym {s}");
        }
        Tok::Str(s) => {
            let _ = writeln!(out, "str {}", json_text(s));
        }
        Tok::Bytes(b) => {
            let _ = writeln!(out, "bytes {}", json_text(&hex_of(b)));
        }
        Tok::Char(c) => {
            let _ = writeln!(out, "char {}", json_text(&c.to_string()));
        }
        Tok::Int(v) => {
            let _ = writeln!(out, "int {v}");
        }
        Tok::IntMin => out.push_str("intmin\n"),
        Tok::Float(v) => {
            let _ = writeln!(out, "float {}", crate::interp::float_text(*v));
        }
        Tok::Newline => out.push_str("nl\n"),
        Tok::Indent => out.push_str("indent\n"),
        Tok::Dedent => out.push_str("dedent\n"),
        Tok::Eof => out.push_str("eof\n"),
        Tok::Interp(parts) => {
            out.push_str("interp\n");
            for p in parts {
                match p {
                    Part::Lit(s) => {
                        let _ = writeln!(out, "lit {}", json_text(s));
                    }
                    Part::Code(code) => {
                        out.push_str("code\n");
                        code.iter().for_each(|t| token_line(t, out));
                    }
                    Part::Fmt(code, spec) => {
                        let _ = writeln!(out, "fmt {}", format_spec(spec).text());
                        code.iter().for_each(|t| token_line(t, out));
                    }
                }
            }
            out.push_str("end\n");
        }
    }
}

// ---- The syntax tree (api.ast.1-3) ----

/// A JSON value, written as the derived `ToJson` of `compiler/ast/ast.ail`
/// writes it (ADR 0376): a number as AIL prints a `float`, a variant with
/// fields as `{"kind":"Name",...}`, one without as `"Name"`, `none` as `null`.
enum J {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<J>),
    Obj(Vec<(&'static str, J)>),
}

impl J {
    fn text(&self) -> String {
        let mut out = String::new();
        self.write(&mut out);
        out
    }

    fn write(&self, out: &mut String) {
        match self {
            J::Null => out.push_str("null"),
            J::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
            // json.print.1: whole numbers below 1e15 print without a fraction, as `Json.to_text` does.
            J::Num(n) if n.is_finite() && n.abs() < 1e15 && n.fract() == 0.0 => out.push_str(&format!("{}", *n as i64)),
            J::Num(n) => out.push_str(&crate::interp::float_text(*n)),
            J::Str(s) => quote(s, out),
            J::Arr(items) => {
                out.push('[');
                for (k, x) in items.iter().enumerate() {
                    if k > 0 {
                        out.push(',');
                    }
                    x.write(out);
                }
                out.push(']');
            }
            J::Obj(members) => {
                out.push('{');
                for (k, (name, x)) in members.iter().enumerate() {
                    if k > 0 {
                        out.push(',');
                    }
                    quote(name, out);
                    out.push(':');
                    x.write(out);
                }
                out.push('}');
            }
        }
    }
}

/// A string as `json_quote` in the prelude writes it (json.print.1).
fn quote(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

fn s(v: &str) -> J {
    J::Str(v.to_string())
}

fn n(v: usize) -> J {
    J::Num(v as f64)
}

fn opt<T>(v: &Option<T>, f: impl FnOnce(&T) -> J) -> J {
    v.as_ref().map_or(J::Null, f)
}

fn arr<T>(items: &[T], f: impl Fn(&T) -> J) -> J {
    J::Arr(items.iter().map(f).collect())
}

/// `{"kind":"Name",...fields}`
fn variant(name: &str, mut fields: Vec<(&'static str, J)>) -> J {
    fields.insert(0, ("kind", s(name)));
    J::Obj(fields)
}

fn pos(p: Pos) -> J {
    J::Obj(vec![("line", n(p.line)), ("col", n(p.col))])
}

fn ident(name: &str, p: Pos) -> J {
    J::Obj(vec![("name", s(name)), ("pos", pos(p))])
}

fn pair(v: &(String, Pos)) -> J {
    ident(&v.0, v.1)
}

fn format_spec(f: &FormatSpec) -> J {
    J::Obj(vec![
        ("fill", opt(&f.fill, |c| s(&c.to_string()))),
        ("align", opt(&f.align, |c| s(&c.to_string()))),
        ("width", opt(&f.width, |w| n(*w))),
        ("precision", opt(&f.precision, |w| n(*w))),
        ("hex", J::Bool(f.hex)),
    ])
}

/// `ail api --ast FILE`: the parse of one file as one line of JSON, then
/// its diagnostics (api.ast.1).
pub fn ast_text(src: &str) -> String {
    let (tokens, mut diags) = lexer::lex(src);
    let (prog, parse_diags) = crate::parser::parse(tokens);
    diags.extend(parse_diags);
    let mut out = program(&prog).text();
    out.push('\n');
    diag_lines(&diags, &mut out);
    out
}

/// `ail api --ast --program FILE|DIR`: the program as the checker sees it,
/// with the prelude, the imported modules and the derive output (api.ast.3).
pub fn program_text(path: &str) -> Result<String, String> {
    let (prog, diags) = if std::path::Path::new(path).is_dir() {
        crate::modules::load(path)?
    } else {
        let src = std::fs::read_to_string(path).map_err(|e| format!("can't read {path}: {e}"))?;
        crate::modules::load_src(path, &src)
    };
    let mut out = program(&prog).text();
    out.push('\n');
    diag_lines(&diags, &mut out);
    Ok(out)
}

fn program(p: &Program) -> J {
    let mut public: Vec<usize> = p.public.iter().copied().collect();
    public.sort_unstable();
    J::Obj(vec![
        ("fns", arr(&p.fns, function)),
        ("structs", arr(&p.structs, struct_decl)),
        ("enums", arr(&p.enums, enum_decl)),
        ("traits", arr(&p.traits, trait_decl)),
        ("impls", arr(&p.impls, impl_decl)),
        ("consts", arr(&p.consts, |c| {
            J::Obj(vec![("name", s(&c.name)), ("pos", pos(c.pos)), ("doc", doc(&c.doc)), ("value", expr(&c.value))])
        })),
        ("uses", arr(&p.uses, |u| {
            J::Obj(vec![("path", arr(&u.path, pair)), ("alias", opt(&u.alias, pair)), ("pos", pos(u.pos))])
        })),
        ("public", arr(&public, |l| n(*l))),
        ("files", arr(&p.files, |f| {
            J::Obj(vec![("path", s(&f.path)), ("module", s(&f.module)), ("base", n(f.base))])
        })),
        ("grants", arr(&p.grants, |g| {
            J::Obj(vec![
                ("package", s(&g.package)),
                ("caps", arr(&g.caps, |c| s(c))),
                ("pos", pos(g.pos)),
                ("own", J::Bool(g.own)),
            ])
        })),
        ("unsafe_allowed", opt(&p.unsafe_allowed, |l| arr(l, |p| s(p)))),
        ("generated", arr(&p.generated, |g| {
            J::Obj(vec![
                ("file", n(g.file)),
                ("impl_pos", pos(g.impl_pos)),
                ("label", s(&g.label)),
                ("generator", s(&g.generator)),
            ])
        })),
    ])
}

fn doc(d: &Option<String>) -> J {
    opt(d, |t| s(t))
}

fn generic(g: &GenericParam) -> J {
    J::Obj(vec![("name", s(&g.name)), ("pos", pos(g.pos)), ("bounds", arr(&g.bounds, pair))])
}

fn owner(o: &Owner) -> J {
    J::Obj(vec![("name", s(&o.name)), ("pos", pos(o.pos)), ("params", opt(&o.params, |ps| arr(ps, generic)))])
}

fn field(f: &Field) -> J {
    J::Obj(vec![("name", s(&f.name)), ("pos", pos(f.pos)), ("doc", doc(&f.doc)), ("ty", ty(&f.ty))])
}

fn contract(c: &Contract) -> J {
    J::Obj(vec![
        ("is_ensures", J::Bool(c.ensures)),
        ("cond", expr(&c.cond)),
        ("text", s(&c.text)),
        ("pos", pos(c.pos)),
        ("olds", arr(&c.olds, |(name, e)| J::Obj(vec![("name", s(name)), ("value", expr(e))]))),
    ])
}

fn struct_decl(d: &StructDecl) -> J {
    J::Obj(vec![
        ("name", s(&d.name)),
        ("pos", pos(d.pos)),
        ("doc", doc(&d.doc)),
        ("is_error", J::Bool(d.is_error)),
        ("is_unique", J::Bool(d.unique)),
        ("generics", arr(&d.generics, generic)),
        ("fields", arr(&d.fields, field)),
        ("invariants", arr(&d.invariants, contract)),
    ])
}

fn enum_decl(d: &EnumDecl) -> J {
    J::Obj(vec![
        ("name", s(&d.name)),
        ("pos", pos(d.pos)),
        ("doc", doc(&d.doc)),
        ("is_error", J::Bool(d.is_error)),
        ("is_unique", J::Bool(d.unique)),
        ("generics", arr(&d.generics, generic)),
        ("variants", arr(&d.variants, |v| {
            J::Obj(vec![
                ("name", s(&v.name)),
                ("pos", pos(v.pos)),
                ("doc", doc(&v.doc)),
                ("fields", arr(&v.fields, field)),
            ])
        })),
    ])
}

fn trait_decl(t: &TraitDecl) -> J {
    J::Obj(vec![
        ("name", s(&t.name)),
        ("pos", pos(t.pos)),
        ("doc", doc(&t.doc)),
        ("methods", arr(&t.methods, |i| n(*i))),
        ("derive", opt(&t.derive, pair)),
        ("supers", arr(&t.supers, pair)),
    ])
}

fn impl_decl(i: &ImplDecl) -> J {
    J::Obj(vec![
        ("trait_name", s(&i.trait_name)),
        ("trait_pos", pos(i.trait_pos)),
        ("target", owner(&i.target)),
        ("pos", pos(i.pos)),
        ("methods", arr(&i.methods, |i| n(*i))),
    ])
}

fn function(f: &FnDecl) -> J {
    J::Obj(vec![
        ("attrs", arr(&f.attrs, |a| {
            J::Obj(vec![("name", s(&a.name)), ("pos", pos(a.pos)), ("arg", opt(&a.arg, |t| s(t)))])
        })),
        ("owner", opt(&f.owner, owner)),
        ("name", s(&f.name)),
        ("pos", pos(f.pos)),
        ("doc", doc(&f.doc)),
        ("generics", arr(&f.generics, generic)),
        ("receiver", opt(&f.receiver, |r| J::Obj(vec![("mutable", J::Bool(r.mutable)), ("moved", J::Bool(r.moved))]))),
        ("params", arr(&f.params, |p| {
            J::Obj(vec![
                ("name", s(&p.name)),
                ("pos", pos(p.pos)),
                ("mutable", J::Bool(p.mutable)),
                ("moved", J::Bool(p.moved)),
                ("ty", ty(&p.ty)),
            ])
        })),
        ("ret", opt(&f.ret, ty)),
        ("body", opt(&f.body, |b| block(b))),
        ("within", match f.within {
            Within::TopLevel => s("TopLevel"),
            Within::Trait(i) => variant("InTrait", vec![("index", n(i))]),
            Within::Impl(i) => variant("InImpl", vec![("index", n(i))]),
        }),
        ("contracts", arr(&f.contracts, contract)),
        ("inherits", opt(&f.inherits, |i| n(*i))),
        ("unsafe_fn", J::Bool(f.unsafe_fn)),
        ("ext", opt(&f.ext, |e| J::Obj(vec![("lib", opt(&e.lib, |l| s(l)))]))),
    ])
}

/// A type as written (api.ast.2).
fn ty(t: &TypeRef) -> J {
    let shape = match &t.kind {
        TypeKind::Named { name, args } => variant("Named", vec![("name", s(name)), ("args", arr(args, ty))]),
        TypeKind::Optional(inner) => variant("Optional", vec![("inner", ty(inner))]),
        TypeKind::Fallible { ok, errors } => variant(
            "Fallible",
            vec![("ok", opt(ok, |t| ty(t))), ("errors", opt(errors, |es| arr(es, ty)))],
        ),
        TypeKind::Any(name) => variant("AnyTrait", vec![("trait_name", s(name))]),
        TypeKind::Ref(inner) => variant("RefTo", vec![("inner", ty(inner))]),
        TypeKind::Ptr(inner) => variant("PtrTo", vec![("inner", ty(inner))]),
        TypeKind::Fn { params, ret } => {
            variant("FnType", vec![("params", arr(params, ty)), ("ret", opt(ret, |r| ty(r)))])
        }
    };
    J::Obj(vec![("shape", shape), ("pos", pos(t.pos))])
}

fn block(b: &Block) -> J {
    arr(b, stmt)
}

fn bin_op(op: BinOp) -> J {
    s(match op {
        BinOp::Add => "Add",
        BinOp::Sub => "Sub",
        BinOp::Mul => "Mul",
        BinOp::Div => "Div",
        BinOp::Rem => "Rem",
        BinOp::Eq => "Eq",
        BinOp::Ne => "Ne",
        BinOp::Lt => "Lt",
        BinOp::Le => "Le",
        BinOp::Gt => "Gt",
        BinOp::Ge => "Ge",
        BinOp::And => "And",
        BinOp::Or => "Or",
    })
}

/// A statement (api.ast.2).
fn stmt(st: &Stmt) -> J {
    match st {
        Stmt::Let { name, pos: p, mutable, value, otherwise } => variant(
            "Let",
            vec![
                ("name", s(name)),
                ("pos", pos(*p)),
                ("mutable", J::Bool(*mutable)),
                ("value", expr(value)),
                ("otherwise", opt(otherwise, otherwise_part)),
            ],
        ),
        Stmt::Assign { target, op, value, pos: p } => variant(
            "Assign",
            vec![("target", expr(target)), ("op", opt(op, |o| bin_op(*o))), ("value", expr(value)), ("pos", pos(*p))],
        ),
        Stmt::While { cond, body, pos: p } => {
            variant("While", vec![("cond", expr(cond)), ("body", block(body)), ("pos", pos(*p))])
        }
        Stmt::For { par, var, var_pos, mutable, iter, body, pos: p } => variant(
            "For",
            vec![
                ("is_par", J::Bool(*par)),
                ("var", s(var)),
                ("var_pos", pos(*var_pos)),
                ("mutable", J::Bool(*mutable)),
                ("iter", expr(iter)),
                ("body", block(body)),
                ("pos", pos(*p)),
            ],
        ),
        Stmt::Break(p) => variant("Break", vec![("pos", pos(*p))]),
        Stmt::Continue(p) => variant("Continue", vec![("pos", pos(*p))]),
        Stmt::Return(value, p) => variant("Return", vec![("value", opt(value, expr)), ("pos", pos(*p))]),
        Stmt::Expr(e) => variant("Eval", vec![("value", expr(e))]),
        Stmt::Handle { value, otherwise } => {
            variant("Handle", vec![("value", expr(value)), ("otherwise", otherwise_part(otherwise))])
        }
        Stmt::Defer(inner, p) => variant("Defer", vec![("stmt", stmt(inner)), ("pos", pos(*p))]),
        // unsafe.block.4: `ail api --ast` lists every `unsafe` block (here) and `unsafe fn` (`unsafe_fn`).
        Stmt::Unsafe { body, pos: p } => variant("Unsafe", vec![("body", block(body)), ("pos", pos(*p))]),
        Stmt::With { mutex, var, var_pos, body, pos: p } => variant(
            "With",
            vec![
                ("mutex", expr(mutex)),
                ("var", s(var)),
                ("var_pos", pos(*var_pos)),
                ("body", block(body)),
                ("pos", pos(*p)),
            ],
        ),
    }
}

fn otherwise_part(e: &Else) -> J {
    match e {
        Else::Value(v) => variant("Fallback", vec![("value", expr(v))]),
        Else::Leave(st) => variant("Leave", vec![("stmt", stmt(st))]),
        Else::Block { err, body, pos: p } => {
            variant("Handler", vec![("err", opt(err, pair)), ("body", block(body)), ("pos", pos(*p))])
        }
    }
}

fn arg(a: &Arg) -> J {
    J::Obj(vec![
        ("label", opt(&a.label, pair)),
        ("mutable", J::Bool(a.mutable)),
        ("moved", J::Bool(a.moved)),
        ("value", expr(&a.value)),
        ("param", opt(&a.param, pair)),
    ])
}

/// An expression (api.ast.2).
fn expr(e: &Expr) -> J {
    match e {
        Expr::Str(v, p) => variant("Str", vec![("value", s(v)), ("pos", pos(*p))]),
        Expr::Bytes(v, p) => variant("Bytes", vec![("hex", s(&hex_of(v))), ("pos", pos(*p))]),
        Expr::Interp(parts, p) => variant(
            "Interp",
            vec![
                ("parts", arr(parts, |part| match part {
                    InterpPart::Lit(t) => variant("Lit", vec![("text", s(t))]),
                    InterpPart::Expr(e) => variant("Hole", vec![("value", expr(e))]),
                    InterpPart::Fmt(e, spec) => variant("Fmt", vec![("value", expr(e)), ("spec", format_spec(spec))]),
                })),
                ("pos", pos(*p)),
            ],
        ),
        Expr::Int(v, p) => variant("Int", vec![("value", s(&v.to_string())), ("pos", pos(*p))]),
        Expr::Float(v, p) => variant("Float", vec![("value", J::Num(*v)), ("pos", pos(*p))]),
        Expr::Char(c, p) => variant("Char", vec![("value", s(&c.to_string())), ("pos", pos(*p))]),
        Expr::Bool(b, p) => variant("Bool", vec![("value", J::Bool(*b)), ("pos", pos(*p))]),
        Expr::None(p) => variant("NoneLit", vec![("pos", pos(*p))]),
        Expr::Hole(p) => variant("Hole", vec![("pos", pos(*p))]),
        Expr::Name(name, p) => variant("Name", vec![("name", s(name)), ("pos", pos(*p))]),
        Expr::TypeArgs { name, args, pos: p } => {
            variant("TypeArgs", vec![("name", s(name)), ("args", arr(args, ty)), ("pos", pos(*p))])
        }
        Expr::List(items, p) => variant("ListLit", vec![("items", arr(items, expr)), ("pos", pos(*p))]),
        Expr::Call { callee, args, pos: p } => {
            variant("Call", vec![("callee", expr(callee)), ("args", arr(args, arg)), ("pos", pos(*p))])
        }
        Expr::Method { recv, name, name_pos, args } => variant(
            "Method",
            vec![("recv", expr(recv)), ("name", s(name)), ("name_pos", pos(*name_pos)), ("args", arr(args, arg))],
        ),
        Expr::Field { recv, name, name_pos } => {
            variant("Member", vec![("recv", expr(recv)), ("name", s(name)), ("name_pos", pos(*name_pos))])
        }
        Expr::Index { recv, index, pos: p } => {
            variant("Index", vec![("recv", expr(recv)), ("index", expr(index)), ("pos", pos(*p))])
        }
        Expr::Unary { op, operand, pos: p } => variant(
            "Unary",
            vec![
                ("op", s(match op {
                    UnOp::Neg => "Neg",
                    UnOp::Not => "Not",
                    UnOp::Spawn => "Spawn",
                })),
                ("operand", expr(operand)),
                ("pos", pos(*p)),
            ],
        ),
        Expr::Binary { op, lhs, rhs, pos: p } => variant(
            "Binary",
            vec![("op", bin_op(*op)), ("lhs", expr(lhs)), ("rhs", expr(rhs)), ("pos", pos(*p))],
        ),
        Expr::Range { lo, hi, inclusive, pos: p } => variant(
            "Range",
            vec![
                ("lo", opt(lo, |e| expr(e))),
                ("hi", opt(hi, |e| expr(e))),
                ("inclusive", J::Bool(*inclusive)),
                ("pos", pos(*p)),
            ],
        ),
        Expr::Map(pairs, p) => variant(
            "MapLit",
            vec![
                ("entries", arr(pairs, |(k, v)| J::Obj(vec![("key", expr(k)), ("value", expr(v))]))),
                ("pos", pos(*p)),
            ],
        ),
        Expr::Set(items, p) => variant("SetLit", vec![("items", arr(items, expr)), ("pos", pos(*p))]),
        Expr::If { branches, els, else_err, pos: p } => variant(
            "If",
            vec![
                ("branches", arr(branches, |(c, b)| J::Obj(vec![("cond", cond(c)), ("body", block(b))]))),
                ("els", opt(els, |b| block(b))),
                ("else_err", opt(else_err, pair)),
                ("pos", pos(*p)),
            ],
        ),
        Expr::Match { scrutinee, arms, pos: p } => variant(
            "Match",
            vec![
                ("scrutinee", expr(scrutinee)),
                ("arms", arr(arms, |a| J::Obj(vec![("pat", pattern(&a.pat)), ("guard", opt(&a.guard, expr)), ("body", block(&a.body))]))),
                ("pos", pos(*p)),
            ],
        ),
        Expr::Try { expr: inner, context, pos: p } => variant(
            "Try",
            vec![("expr", expr(inner)), ("context", opt(context, |c| expr(c))), ("pos", pos(*p))],
        ),
        Expr::Comp { kind, elem, value, clauses, pos: p } => variant(
            "Comp",
            vec![
                ("form", s(match kind {
                    CompKind::List => "ListComp",
                    CompKind::Set => "SetComp",
                    CompKind::Map => "MapComp",
                })),
                ("elem", expr(elem)),
                ("value", opt(value, |v| expr(v))),
                ("clauses", arr(clauses, |c| {
                    J::Obj(vec![
                        ("is_par", J::Bool(c.par)),
                        ("var", s(&c.var)),
                        ("var_pos", pos(c.var_pos)),
                        ("iter", expr(&c.iter)),
                        ("cond", opt(&c.cond, expr)),
                    ])
                })),
                ("pos", pos(*p)),
            ],
        ),
        Expr::Else { value, otherwise, pos: p } => variant(
            "OrElse",
            vec![("value", expr(value)), ("otherwise", otherwise_part(otherwise)), ("pos", pos(*p))],
        ),
        Expr::Ref { value, pos: p } => variant("NewRef", vec![("value", expr(value)), ("pos", pos(*p))]),
    }
}

fn cond(c: &Cond) -> J {
    match c {
        Cond::Expr(e) => variant("Test", vec![("value", expr(e))]),
        Cond::Bind { name, pos: p, value } => {
            variant("Bind", vec![("name", s(name)), ("pos", pos(*p)), ("value", expr(value))])
        }
    }
}

/// A pattern (api.ast.2).
fn pattern(p: &Pattern) -> J {
    match p {
        Pattern::Wildcard(at) => variant("Wildcard", vec![("pos", pos(*at))]),
        Pattern::Literal(lit, at) => variant("Literal", vec![("value", literal(lit)), ("pos", pos(*at))]),
        Pattern::Range { lo, hi, inclusive, pos: at } => variant(
            "Range",
            vec![("lo", literal(lo)), ("hi", literal(hi)), ("inclusive", J::Bool(*inclusive)), ("pos", pos(*at))],
        ),
        Pattern::Or(alts, at) => variant("Alt", vec![("alts", arr(alts, pattern)), ("pos", pos(*at))]),
        Pattern::Variant { qualifier, name, bindings, pos: at } => variant(
            "Variant",
            vec![
                ("qualifier", opt(qualifier, |q| s(q))),
                ("name", s(name)),
                ("bindings", opt(bindings, |b| arr(b, pair))),
                ("pos", pos(*at)),
            ],
        ),
    }
}

fn literal(lit: &Literal) -> J {
    match lit {
        Literal::Int(v) => variant("IntLit", vec![("value", s(&v.to_string()))]),
        Literal::Char(c) => variant("CharLit", vec![("value", s(&c.to_string()))]),
        Literal::Str(t) => variant("StrLit", vec![("value", s(t))]),
        Literal::Bool(b) => variant("BoolLit", vec![("value", J::Bool(*b))]),
    }
}

fn json_text(v: &str) -> String {
    let mut out = String::new();
    quote(v, &mut out);
    out
}

/// A byte string as lowercase hex, the form the syntax API shows `bytes` literals in.
fn hex_of(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}
