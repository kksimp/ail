//! Tuples (type.tuple.1-9, ADR 0791): `(a, b)` values typed `(A, B)`,
//! element access `t.0`, destructuring `a, b := t` and `for a, b in xs`, and
//! tuple patterns in `match`.

use super::{Checker, Kind, Local, Ty};
use crate::ast::{Arg, Binder, Expr, Pattern};
use crate::diag::{Diagnostic, Pos};

/// Whether `name` is the hidden variable of `for a, b in xs` (type.tuple.5).
pub(super) fn is_hidden(name: &str) -> bool {
    name.starts_with('(')
}

impl<'p> Checker<'p> {
    /// `(a, b, ...)` (type.tuple.2): each element against the expected
    /// element type, so `(1, none)` can be a `(int, str?)`.
    pub(super) fn tuple_lit(&mut self, items: &'p [Expr], expect: Option<&Ty>) -> Ty {
        let want: Option<Vec<Ty>> = match expect {
            Some(Ty::Tuple(xs)) if xs.len() == items.len() => Some(xs.clone()),
            Some(Ty::Opt(t)) => match &**t {
                Ty::Tuple(xs) if xs.len() == items.len() => Some(xs.clone()),
                _ => None,
            },
            _ => None,
        };
        let mut elems = Vec::with_capacity(items.len());
        for (i, e) in items.iter().enumerate() {
            let w = want.as_ref().map(|w| w[i].clone());
            let t = self.expr(e, w.as_ref(), true);
            self.consume(e, &t);
            if self.reject_unhandled(&t, e.pos()) {
                elems.push(Ty::Err);
                continue;
            }
            if matches!(t, Ty::Unit) {
                self.error("type.tuple.2", e.pos(), "a tuple's elements must be values");
                elems.push(Ty::Err);
                continue;
            }
            self.no_unique(&t, e.pos(), "an element of a tuple");
            match w {
                Some(w) if self.accepts(&w, &t) => elems.push(w),
                _ => elems.push(if t == Ty::Never { Ty::Err } else { t }),
            }
        }
        Ty::Tuple(elems)
    }

    /// `zip(a, b)` (type.tuple.9): the pairs of two lists, as long as the
    /// shorter.
    pub(super) fn zip_call(&mut self, args: &'p [Arg], pos: Pos) -> Ty {
        self.resolved.zips.insert(pos);
        let mut elems = Vec::new();
        for a in args {
            if a.label.is_some() || a.mutable || a.moved || a.param.is_some() {
                self.error_help("type.tuple.9", a.value.pos(), "`zip` takes two plain lists", "write `zip(a, b)`");
            }
            let t = self.expr(&a.value, None, true);
            self.consume(&a.value, &t);
            match t {
                Ty::List(e) => elems.push(*e),
                Ty::Err => elems.push(Ty::Err),
                t if self.reject_unhandled(&t, a.value.pos()) => elems.push(Ty::Err),
                t => {
                    let msg = format!("`zip` takes lists, but this is {}", self.show(&t));
                    let help = match t {
                        Ty::Str => "zip the characters with `s.chars()`",
                        Ty::Map(..) | Ty::Set(_) => "zip a list of its keys, e.g. `m.keys()`",
                        _ => "pass two lists",
                    };
                    self.error_help("type.tuple.9", a.value.pos(), msg, help);
                    elems.push(Ty::Err);
                }
            }
        }
        if args.len() != 2 {
            let msg = format!("`zip` takes two lists, but {} given", super::was_were(args.len()));
            self.error_help("type.tuple.9", pos, msg, "write `zip(a, b)`; zip a third with `zip(zip(a, b), c)`");
            return Ty::Err;
        }
        Ty::List(Box::new(Ty::Tuple(elems)))
    }

    /// `t.0` (type.tuple.3), or `None` when `name` isn't an element index
    /// and `rt` isn't a tuple.
    pub(super) fn tuple_field(&mut self, rt: &Ty, name: &str, pos: Pos) -> Option<Ty> {
        let index = name.parse::<usize>().ok().filter(|_| name.bytes().all(|b| b.is_ascii_digit()));
        match (rt, index) {
            (Ty::Tuple(xs), Some(i)) if i < xs.len() => Some(xs[i].clone()),
            (Ty::Tuple(xs), Some(i)) => {
                let msg = format!("{} has no element {i}", self.show(rt));
                let help = format!("its elements are `.0` to `.{}`", xs.len() - 1);
                self.error_help("type.tuple.3", pos, msg, help);
                Some(Ty::Err)
            }
            (Ty::Tuple(xs), None) => {
                let msg = format!("{} is a tuple, so it has no field `{name}`", self.show(rt));
                let help = format!("read its elements by position, `.0` to `.{}`", xs.len() - 1);
                self.error_help("type.tuple.3", pos, msg, help);
                Some(Ty::Err)
            }
            (_, Some(i)) => {
                let msg = format!("`.{i}` reads an element of a tuple, but this is {}", self.show(rt));
                let help = match rt {
                    Ty::List(_) => format!("index a list with `[{i}]`"),
                    _ => "only tuples have numbered elements".to_string(),
                };
                self.error_help("type.tuple.3", pos, msg, help);
                Some(Ty::Err)
            }
            _ => None,
        }
    }

    /// `a, b := e` (type.tuple.4), and the first statement of the body of
    /// `for a, b in xs` (type.tuple.5).
    pub(super) fn let_tuple(&mut self, names: &'p [Binder], value: &'p Expr, pos: Pos) {
        let hidden = matches!(value, Expr::Name(n, _) if is_hidden(n));
        let vt = self.expr(value, None, true);
        self.consume(value, &vt);
        let vt = if self.reject_unhandled(&vt, value.pos()) { Self::ok_type(&vt) } else { vt };
        let elems = match &vt {
            Ty::Tuple(xs) if xs.len() == names.len() => xs.clone(),
            Ty::Err => vec![Ty::Err; names.len()],
            Ty::Tuple(xs) => {
                let (rule, what) = if hidden { ("type.tuple.5", "each element is") } else { ("type.tuple.4", "the value is") };
                let msg = format!("{} names, but {what} a tuple of {}, {}", names.len(), xs.len(), self.show(&vt));
                let help = "write one name per element, `_` for the ones you don't need";
                self.error_help(rule, pos, msg, help);
                vec![Ty::Err; names.len()]
            }
            t => {
                let shown = self.show(t);
                if hidden {
                    let help = match t {
                        Ty::Int => "for the index and the element, loop over `xs.enumerate()`".to_string(),
                        _ => "loop with one name, or over `xs.enumerate()` or `zip(a, b)` for pairs".to_string(),
                    };
                    self.error_help("type.tuple.5", pos, format!("`for` takes each element apart, but the elements are {shown}"), help);
                } else {
                    self.error_help(
                        "type.tuple.4",
                        pos,
                        format!("several names take a tuple apart, but the value is {shown}"),
                        "bind it to one name",
                    );
                }
                vec![Ty::Err; names.len()]
            }
        };
        for (b, ty) in names.iter().zip(elems) {
            if b.name == "_" {
                continue;
            }
            if self.declare(&b.name, b.pos) {
                let kind = if hidden { Kind::LoopVar } else { Kind::Binding };
                self.locals.push(Local { name: b.name.clone(), ty, mutable: b.mutable, kind, decl: Some(b.pos) });
            }
        }
    }

    /// The names of `for a, b in xs` in a comprehension (type.tuple.5):
    /// the clause's own variable has the element type `elem`.
    pub(super) fn unpack_clause(&mut self, unpack: &'p [Binder], elem: &Ty, pos: Pos) {
        let elems = match elem {
            Ty::Tuple(xs) if xs.len() == unpack.len() => xs.clone(),
            Ty::Err => vec![Ty::Err; unpack.len()],
            t => {
                let msg = match t {
                    Ty::Tuple(xs) => format!("{} names, but each element is a tuple of {}", unpack.len(), xs.len()),
                    t => format!("`for` takes each element apart, but the elements are {}", self.show(t)),
                };
                self.error_help("type.tuple.5", pos, msg, "loop with one name, or over `xs.enumerate()` or `zip(a, b)` for pairs");
                vec![Ty::Err; unpack.len()]
            }
        };
        for (b, ty) in unpack.iter().zip(elems) {
            if b.name != "_" && self.declare(&b.name, b.pos) {
                self.locals.push(Local { name: b.name.clone(), ty, mutable: false, kind: Kind::LoopVar, decl: Some(b.pos) });
            }
        }
    }

    /// One arm's pattern on a tuple `xs` (type.tuple.6): the names it binds.
    /// `wildcard` is set once an arm matches every tuple.
    pub(super) fn tuple_arm(&mut self, xs: &[Ty], pat: &Pattern, wildcard: &mut bool) -> Vec<(String, Pos, Ty)> {
        let tuple = Ty::Tuple(xs.to_vec());
        match pat {
            Pattern::Wildcard(pos) => {
                if *wildcard {
                    self.error("expr.match.5", *pos, "this arm can never match; earlier arms cover every case");
                }
                *wildcard = true;
                Vec::new()
            }
            Pattern::Or(alts, _) => {
                for a in alts {
                    let mut covers = false;
                    let names = self.elem_pattern(&tuple, a, &mut covers);
                    if let Some((_, p, _)) = names.iter().find(|(n, _, _)| n != "_") {
                        self.error_help(
                            "expr.match.8",
                            *p,
                            "a pattern with alternatives can't bind names",
                            "give this alternative an arm of its own, or use `_`",
                        );
                    }
                }
                Vec::new()
            }
            Pattern::Tuple(_, pos) => {
                let mut covers = true;
                let names = self.elem_pattern(&tuple, pat, &mut covers);
                if covers {
                    if *wildcard {
                        self.error("expr.match.5", *pos, "this arm can never match; earlier arms cover every case");
                    }
                    *wildcard = true;
                }
                names
            }
            other => {
                let msg = format!("the value matched is {}, a tuple", self.show(&tuple));
                self.error_help("type.tuple.6", other.pos(), msg, "match it with a tuple pattern such as `(0, x)`, or `_`");
                Vec::new()
            }
        }
    }

    /// A pattern for a value of type `t` inside a tuple pattern: its names,
    /// and `covers` stays true only if it matches every value.
    fn elem_pattern(&mut self, t: &Ty, pat: &Pattern, covers: &mut bool) -> Vec<(String, Pos, Ty)> {
        match (pat, t) {
            (Pattern::Wildcard(_), _) => Vec::new(),
            (Pattern::Bind(name, pos), t) => {
                if let Ty::Enum(e, _) = t
                    && self.prog.enums[*e].variants.iter().any(|v| v.name == *name)
                {
                    let en = &self.prog.enums[*e].name;
                    self.diags.push(
                        Diagnostic::new("type.tuple.6", *pos, format!("`{name}` here binds a new name; it doesn't match the variant"))
                            .help(format!("write `{en}.{name}` to match the variant, or another name to bind the element")),
                    );
                }
                vec![(name.clone(), *pos, t.clone())]
            }
            (Pattern::Tuple(ps, pos), Ty::Tuple(xs)) => {
                if ps.len() != xs.len() {
                    let msg = format!("this pattern has {} elements, but the value is {}", ps.len(), self.show(t));
                    self.error("type.tuple.6", *pos, msg);
                    *covers = false;
                    return Vec::new();
                }
                let mut names = Vec::new();
                for (p, x) in ps.iter().zip(xs) {
                    names.extend(self.elem_pattern(x, p, covers));
                }
                names
            }
            (_, Ty::Err) => {
                *covers = false;
                Vec::new()
            }
            (_, Ty::Enum(e, args)) => {
                *covers = false;
                let (e, args) = (*e, args.clone());
                let mut covered = vec![false; self.prog.enums[e].variants.len()];
                let mut wc = false;
                self.pattern(e, &args, pat, &mut covered, &mut wc)
            }
            (Pattern::Tuple(_, pos), t) => {
                *covers = false;
                let msg = format!("a tuple pattern, but this element is {}", self.show(t));
                self.error("type.tuple.6", *pos, msg);
                Vec::new()
            }
            (_, t) => {
                *covers = false;
                let lit = matches!(t, Ty::Int | Ty::Char | Ty::Str | Ty::Bool) || matches!(t, Ty::Num(k) if !k.is_float());
                if lit {
                    let (mut seen, mut wc) = (Vec::new(), false);
                    self.literal_pattern(t, pat, &mut seen, &mut wc);
                } else {
                    let msg = format!("this element is {}, which only `_` or a name can match", self.show(t));
                    self.error("type.tuple.6", pat.pos(), msg);
                }
                Vec::new()
            }
        }
    }
}

/// The names a pattern binds, typed unknown: the scrutinee was already
/// reported.
pub(super) fn pattern_names(p: &Pattern) -> Vec<(String, Pos, Ty)> {
    match p {
        Pattern::Bind(n, at) => vec![(n.clone(), *at, Ty::Err)],
        Pattern::Tuple(ps, _) => ps.iter().flat_map(pattern_names).collect(),
        Pattern::Variant { bindings: Some(names), .. } => names.iter().map(|(n, at)| (n.clone(), *at, Ty::Err)).collect(),
        _ => Vec::new(),
    }
}
