//! Tasks and their communication types (spec 11, ADRs 0095-0096, 0100,
//! 0186-0189): `spawn f(args)` (conc.task.1), `Task[T]` and its unique-handle
//! rule (conc.task.5-6), `TaskGroup` (conc.task.4, conc.task.8),
//! `Channel[T]` (conc.share.3-4) and `Mutex[T]` (conc.share.2, conc.share.5).

use super::{Checker, Errs, Sig, Ty, ty_key};
use crate::ast::{Arg, Block, Cond, Else, Expr, InterpPart, Program, Stmt, UnOp, is_prelude};
use crate::diag::{Diagnostic, Pos};

/// The four handle types. `Group` has no type argument (its `Ty` is `Unit`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Conc {
    Task,
    Group,
    Channel,
    Mutex,
    /// `Atomic[int]` or `Atomic[bool]` (conc.atomic.3, ADR 0320).
    Atomic,
}

impl Conc {
    pub fn name(self) -> &'static str {
        match self {
            Conc::Task => "Task",
            Conc::Group => "TaskGroup",
            Conc::Channel => "Channel",
            Conc::Mutex => "Mutex",
            Conc::Atomic => "Atomic",
        }
    }
}

/// The names of the handle types, for `BUILTIN_TYPES`.
/// The prelude's `enum AtomicOrder` (conc.atomic.7, ADR 0352).
const ATOMIC_ORDERS: &[&str] = &["SeqCst", "Acquire", "Release", "AcqRel", "Relaxed"];

pub(super) const TYPE_NAMES: &[&str] = &["Task", "TaskGroup", "Channel", "Mutex", "Atomic"];

const TASK_HELP: &str = "wait it with `t.wait()?`, or add it to a group with `g.add(t)`";

impl<'p> Checker<'p> {
    /// A handle type written in a signature or field: `Channel[T]`,
    /// `Mutex[T]` or `TaskGroup`. `Task[T]` can't be written: a task handle
    /// never leaves the block that spawned it (conc.task.5).
    pub(super) fn conc_type(&mut self, name: &str, args: &[crate::ast::TypeRef], pos: Pos) -> Ty {
        match name {
            "Task" => {
                self.error_help(
                    "conc.task.5",
                    pos,
                    "a task handle can't be stored, passed or returned; it stays in the block that spawned it",
                    TASK_HELP,
                );
                Ty::Err
            }
            "TaskGroup" if args.is_empty() => Ty::Conc(Conc::Group, Box::new(Ty::Unit)),
            "TaskGroup" => {
                self.error("gram.generic.1", pos, "`TaskGroup` takes no type arguments");
                Ty::Err
            }
            _ if args.len() == 1 => {
                let k = match name {
                    "Channel" => Conc::Channel,
                    "Atomic" => Conc::Atomic,
                    _ => Conc::Mutex,
                };
                let inner = self.resolve_type(&args[0]);
                if k == Conc::Atomic && !self.atomic_inner(&inner, pos) {
                    return Ty::Err;
                }
                Ty::Conc(k, Box::new(inner))
            }
            _ => {
                self.error_help(
                    "gram.generic.1",
                    pos,
                    format!("`{name}` takes 1 type argument, but {} given", super::was_were(args.len())),
                    format!("write e.g. `{name}[int]`"),
                );
                Ty::Err
            }
        }
    }

    /// `TaskGroup()`, `Channel[T](capacity)`, `Mutex(value)` or
    /// `Mutex[T](value)`; `None` for any other call.
    pub(super) fn conc_constructor(
        &mut self,
        name: &str,
        targs: Option<&[crate::ast::TypeRef]>,
        args: &'p [Arg],
        pos: Pos,
        expect: Option<&Ty>,
    ) -> Option<Ty> {
        let kind = match name {
            "TaskGroup" => Conc::Group,
            "Channel" => Conc::Channel,
            "Mutex" => Conc::Mutex,
            "Atomic" => Conc::Atomic,
            "Task" => {
                self.args_only(args);
                self.error_help("conc.task.1", pos, "a task is started with `spawn`, not built", "write `spawn f(args)`");
                return Some(Ty::Err);
            }
            _ => return None,
        };
        self.resolved.symbols.insert(pos, format!("type:{name}"));
        let inner = match (kind, targs) {
            (Conc::Group, None) => Ty::Unit,
            (Conc::Group, Some(_)) => {
                self.args_only(args);
                self.error("gram.generic.1", pos, "`TaskGroup` takes no type arguments");
                return Some(Ty::Err);
            }
            (_, Some(t)) => match self.conc_type(name, t, pos) {
                Ty::Conc(_, inner) => *inner,
                _ => {
                    self.args_only(args);
                    return Some(Ty::Err);
                }
            },
            (_, None) => match expect {
                Some(Ty::Conc(k, inner)) if *k == kind => (**inner).clone(),
                // `Mutex(v)`: the type of the first value.
                _ if matches!(kind, Conc::Mutex | Conc::Atomic) && args.len() == 1 => {
                    let t = self.expr(&args[0].value, None, true);
                    if self.reject_unhandled(&t, args[0].value.pos()) {
                        return Some(Ty::Err);
                    }
                    if kind == Conc::Atomic && !self.atomic_inner(&t, args[0].value.pos()) {
                        return Some(Ty::Err);
                    }
                    return Some(Ty::Conc(kind, Box::new(t)));
                }
                _ => {
                    self.args_only(args);
                    self.error_help(
                        "conc.share.4",
                        pos,
                        "`Channel` needs its element type",
                        "write e.g. `Channel[int](8)`",
                    );
                    return Some(Ty::Err);
                }
            },
        };
        let param = match kind {
            Conc::Group => None,
            Conc::Channel => Some(Ty::Int),
            _ => Some(inner.clone()),
        };
        let sig = Sig::plain(None, param.into_iter().map(|t| (false, t)).collect(), Ty::Unit);
        self.check_call(name, &sig, Vec::new(), args, None, pos, None);
        Some(Ty::Conc(kind, Box::new(inner)))
    }

    /// A method of a handle: `t.wait()`, `g.add(t)`, `g.wait_all()`,
    /// `ch.send(v)`, `ch.recv()`, `ch.close()`, `m.get()`, `m.set(v)`,
    /// `m.swap(v)`, `m.lock()`, `m.unlock(v)`.
    pub(super) fn conc_method(&mut self, kind: Conc, inner: &Ty, name: &str, pos: Pos, args: &'p [Arg]) -> Ty {
        // conc.atomic.7: a trailing `order:` names a weaker ordering.
        let args = match args.split_last() {
            Some((o, rest)) if kind == Conc::Atomic && o.label.as_ref().is_some_and(|(l, _)| l == "order") => {
                self.atomic_order(name, o);
                rest
            }
            _ => args,
        };
        let trapped = self.trapped();
        let t = inner.clone();
        let (rule, params, ret): (&str, Vec<Ty>, Ty) = match (kind, name) {
            // conc.task.6: the task's result, or `Trapped`; a fallible task's
            // errors pass through.
            (Conc::Task, "wait") => {
                let (ok, mut errs) = match &t {
                    Ty::Res(ok, e) => ((**ok).clone(), self.errs_of(e)),
                    _ => (t.clone(), Vec::new()),
                };
                if !errs.contains(&trapped) {
                    errs.push(trapped);
                    errs.sort_by_key(ty_key);
                }
                ("conc.task.6", vec![], Ty::Res(Box::new(ok), Errs::Set(errs)))
            }
            (Conc::Group, "add") if args.len() == 1 => {
                let t = self.expr(&args[0].value, None, true);
                match &t {
                    Ty::Conc(Conc::Task, r) if **r != Ty::Unit && **r != Ty::Err => self.error_help(
                        "conc.task.8",
                        args[0].value.pos(),
                        format!("a `TaskGroup` holds tasks that return nothing; this one returns {}", self.show(r)),
                        "wait it with `t.wait()?` to use its result, or spawn a function that returns nothing",
                    ),
                    Ty::Conc(Conc::Task, _) | Ty::Err => {}
                    _ => self.mismatch("expr.call.2", args[0].value.pos(), &Ty::Conc(Conc::Task, Box::new(Ty::Unit)), &t),
                }
                return Ty::Unit;
            }
            (Conc::Group, "wait_all") => ("conc.task.8", vec![], Ty::Res(Box::new(Ty::Unit), Errs::Set(vec![trapped]))),
            (Conc::Channel, "send") => ("conc.share.4", vec![t], Ty::Unit),
            (Conc::Channel, "recv") => ("conc.share.4", vec![], Ty::Opt(Box::new(t))),
            (Conc::Channel, "close") => ("conc.share.4", vec![], Ty::Unit),
            (Conc::Mutex, "get" | "lock") => ("conc.share.5", vec![], t),
            (Conc::Mutex, "set" | "unlock") => ("conc.share.5", vec![t], Ty::Unit),
            (Conc::Mutex, "swap") => ("conc.share.5", vec![t.clone()], t),
            // conc.cancel.1: ask a task, or every task of a group, to stop.
            (Conc::Task | Conc::Group, "cancel") => ("conc.cancel.1", vec![], Ty::Unit),
            // conc.atomic.4-5
            (Conc::Atomic, "load") => ("conc.atomic.4", vec![], t),
            (Conc::Atomic, "store") => ("conc.atomic.4", vec![t], Ty::Unit),
            (Conc::Atomic, "add") if t == Ty::Int => ("conc.atomic.5", vec![Ty::Int], Ty::Int),
            (Conc::Atomic, "cas") => ("conc.atomic.4", vec![t.clone(), t], Ty::Bool),
            _ => {
                self.args_only(args);
                let names = match kind {
                    Conc::Task => "`wait` and `cancel`",
                    Conc::Group => "`add`, `wait_all` and `cancel`",
                    Conc::Channel => "`send`, `recv` and `close`",
                    Conc::Mutex => "`get`, `set`, `swap`, `lock` and `unlock`",
                    Conc::Atomic if t == Ty::Int => "`load`, `store`, `add` and `cas`",
                    Conc::Atomic => "`load`, `store` and `cas` (`add` needs `Atomic[int]`)",
                };
                self.error_help(
                    "expr.name.1",
                    pos,
                    format!("`{}` has no method `{name}`", kind.name()),
                    format!("its methods are {names}"),
                );
                return Ty::Err;
            }
        };
        let _ = rule;
        let sig = Sig::plain(Some(false), params.into_iter().map(|t| (false, t)).collect(), ret);
        self.check_call(name, &sig, Vec::new(), args, None, pos, None)
    }

    /// `with m as mut x` (conc.share.6): `m` is a `Mutex[T]` and `x` a `mut`
    /// local of type `T` for the body, which can't name `m` again.
    pub(super) fn with_stmt(&mut self, mutex: &'p Expr, var: &str, var_pos: Pos, body: &'p Block, pos: Pos) {
        let inner = match self.expr(mutex, None, true) {
            Ty::Conc(Conc::Mutex, inner) => *inner,
            Ty::Err => Ty::Err,
            t => {
                let msg = format!("`with` holds a `Mutex`, but this is {}", self.show(&t));
                self.error_help("conc.share.6", mutex.pos(), msg, "make it with `Mutex(value)`");
                Ty::Err
            }
        };
        // Holding a mutex synchronises with other tasks (conc.par.2).
        self.resolved.handle_calls.insert(pos);
        if let Some(at) = self.overlapping_use(mutex, body) {
            self.error_help(
                "conc.share.6",
                at,
                "the body of `with` can't use the mutex it holds",
                format!("use `{var}`, which is stored back into the mutex when the block ends"),
            );
        }
        let mark = self.locals.len();
        if self.declare(var, var_pos) {
            self.locals.push(super::Local { name: var.to_string(), ty: inner, mutable: true, kind: super::Kind::Binding, decl: None });
        }
        self.block(body, None, false);
        self.end_scope(mark);
    }

    /// `spawn f(args)` (conc.task.1): a `Task[T]` for a call of a named
    /// function that returns `T`.
    pub(super) fn spawn_expr(&mut self, operand: &'p Expr, pos: Pos) -> Ty {
        let (args, at) = match operand {
            Expr::Call { callee, args, .. } if matches!(**callee, Expr::Name(..) | Expr::TypeArgs { .. }) => {
                (args, callee.pos())
            }
            Expr::Method { recv, args, name_pos, .. } if matches!(**recv, Expr::Name(..) | Expr::TypeArgs { .. }) => {
                (args, *name_pos)
            }
            _ => {
                self.quiet_type(operand);
                return self.not_spawnable(pos);
            }
        };
        let t = self.expr(operand, None, false);
        // Only a declared function or associated function runs as a task.
        // A method call on a value (`p.sum()`) isn't one: the receiver must name a type.
        let assoc = matches!(operand, Expr::Method { recv, .. }
            if matches!(&**recv, Expr::Name(n, _) | Expr::TypeArgs { name: n, .. } if self.types.contains_key(n.as_str())));
        let declared = self.resolved.symbols.get(&at).is_some_and(|s| s.starts_with("fn:"))
            || (assoc && self.resolved.targets.contains_key(&at));
        if t == Ty::Err {
            return Ty::Err;
        }
        if !declared {
            return self.not_spawnable(pos);
        }
        // conc.share.1: the task gets its own copies, so nothing is passed `mut`.
        for a in args.iter().filter(|a| a.mutable) {
            self.error_help(
                "conc.share.1",
                a.value.pos(),
                "a spawned call can't take a `mut` argument; the task works on its own copy",
                "remove the `mut`, and send results back through the task's result or a `Channel`",
            );
        }
        let t = if t == Ty::Never { Ty::Unit } else { t };
        Ty::Conc(Conc::Task, Box::new(t))
    }

    fn not_spawnable(&mut self, pos: Pos) -> Ty {
        self.error_help(
            "conc.task.1",
            pos,
            "`spawn` takes a call of a declared function, e.g. `spawn work(x)`",
            "wrap the work in a function and spawn a call of it; to wait at once, bind it: `t := spawn work(x)`",
        );
        Ty::Err
    }

    /// conc.atomic.7 (ADR 0352): `order: O` on an atomic method, where `O`
    /// is a prelude ordering the method can use.
    fn atomic_order(&mut self, method: &str, arg: &'p Arg) {
        let expect = match self.types.get("AtomicOrder") {
            Some(super::Def::Enum(i)) => Some(Ty::Enum(*i, Vec::new())),
            _ => None,
        };
        if self.expr(&arg.value, expect.as_ref(), true) == Ty::Err {
            return;
        }
        let o = match &arg.value {
            Expr::Name(o, _) if ATOMIC_ORDERS.contains(&o.as_str()) => o.as_str(),
            e => {
                return self.error_help(
                    "conc.atomic.7",
                    e.pos(),
                    "an atomic's ordering is written as its name, so the checker can see it",
                    "write `order: Relaxed`, `Acquire`, `Release`, `AcqRel` or `SeqCst`",
                );
            }
        };
        let (allowed, why) = match method {
            "load" => ("`SeqCst`, `Acquire` or `Relaxed`", "a load only reads"),
            "store" => ("`SeqCst`, `Release` or `Relaxed`", "a store only writes"),
            _ => return,
        };
        if !allowed.contains(&format!("`{o}`")) {
            self.error_help(
                "conc.atomic.7",
                arg.value.pos(),
                format!("`{method}` can't use `{o}`: {why}"),
                format!("use {allowed}"),
            );
        }
    }

    /// conc.atomic.3: an atomic holds an `int` or a `bool`.
    fn atomic_inner(&mut self, t: &Ty, pos: Pos) -> bool {
        if matches!(t, Ty::Int | Ty::Bool | Ty::Err) {
            return true;
        }
        self.error_help(
            "conc.atomic.3",
            pos,
            format!("an `Atomic` holds an `int` or a `bool`, not {}", self.show(t)),
            "share other values through a `Mutex`, e.g. `Mutex(value)`",
        );
        false
    }

    /// The prelude's `error struct Trapped` (conc.task.6).
    fn trapped(&self) -> Ty {
        match self.types.get("Trapped") {
            Some(super::Def::Struct(s)) => Ty::Struct(*s, Vec::new()),
            _ => Ty::Err,
        }
    }
}

/// conc.task.5: every task handle is consumed exactly once, on every path,
/// by `.wait()` or by `g.add(t)`, in the block that spawned it; a `spawn`
/// appears only where its handle is bound, added or waited.
pub fn check_handles(prog: &Program) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    for f in prog.fns.iter().filter(|f| !is_prelude(f.pos)) {
        if let Some(body) = &f.body {
            block(body, &mut diags);
        }
    }
    diags
}

fn block(stmts: &Block, diags: &mut Vec<Diagnostic>) {
    for (i, s) in stmts.iter().enumerate() {
        if let Stmt::Let { name, pos, value: Expr::Unary { op: UnOp::Spawn, operand, .. }, otherwise, .. } = s {
            spawns(operand, diags);
            if let Some(o) = otherwise {
                else_spawns(o, diags);
            }
            consumed(name, *pos, &stmts[i + 1..], diags);
            continue;
        }
        stmt_spawns(s, diags);
    }
}

/// Check every `spawn` inside `s` is in an allowed place.
fn stmt_spawns(s: &Stmt, diags: &mut Vec<Diagnostic>) {
    match s {
        Stmt::Let { value, otherwise, .. } => {
            spawns(value, diags);
            if let Some(o) = otherwise {
                else_spawns(o, diags);
            }
        }
        Stmt::Assign { target, value, .. } => {
            spawns(target, diags);
            spawns(value, diags);
        }
        Stmt::While { cond, body, .. } => {
            spawns(cond, diags);
            block(body, diags);
        }
        Stmt::For { iter, body, .. } | Stmt::With { mutex: iter, body, .. } => {
            spawns(iter, diags);
            block(body, diags);
        }
        Stmt::Return(Some(e), _) | Stmt::Expr(e) => spawns(e, diags),
        Stmt::Handle { value, otherwise } => {
            spawns(value, diags);
            else_spawns(otherwise, diags);
        }
        Stmt::Defer(inner, _) => stmt_spawns(inner, diags),
        Stmt::Break(_) | Stmt::Continue(_) | Stmt::Return(None, _) => {}
    }
}

fn else_spawns(o: &Else, diags: &mut Vec<Diagnostic>) {
    match o {
        Else::Value(e) => spawns(e, diags),
        Else::Leave(s) => stmt_spawns(s, diags),
        Else::Block { body, .. } => block(body, diags),
    }
}

/// A subexpression, or a nested block.
enum Node<'a> {
    E(&'a Expr),
    B(&'a Block),
}

/// Visit the subexpressions and nested blocks of `e`.
fn children<'a>(e: &'a Expr, visit: &mut dyn FnMut(Node<'a>)) {
    let ex = |x: &'a Expr, visit: &mut dyn FnMut(Node<'a>)| visit(Node::E(x));
    match e {
        Expr::Interp(parts, _) => {
            for p in parts {
                if let InterpPart::Expr(x) | InterpPart::Fmt(x, _) = p {
                    ex(x, visit);
                }
            }
        }
        Expr::List(xs, _) | Expr::Set(xs, _) => xs.iter().for_each(|x| ex(x, visit)),
        Expr::Map(pairs, _) => {
            for (k, v) in pairs {
                ex(k, visit);
                ex(v, visit);
            }
        }
        Expr::Call { callee: x, args, .. } | Expr::Method { recv: x, args, .. } => {
            ex(x, visit);
            args.iter().for_each(|a| ex(&a.value, visit));
        }
        Expr::Field { recv, .. } => ex(recv, visit),
        Expr::Index { recv: a, index: b, .. } | Expr::Binary { lhs: a, rhs: b, .. } => {
            ex(a, visit);
            ex(b, visit);
        }
        Expr::Unary { operand, .. } => ex(operand, visit),
        Expr::Range { lo, hi, .. } => {
            for x in [lo, hi].into_iter().flatten() {
                ex(x, visit);
            }
        }
        Expr::Try { expr, context, .. } => {
            ex(expr, visit);
            if let Some(c) = context {
                ex(c, visit);
            }
        }
        Expr::Else { value, otherwise, .. } => {
            ex(value, visit);
            match otherwise.as_ref() {
                Else::Value(x) => ex(x, visit),
                Else::Leave(s) => {
                    if let Stmt::Return(Some(x), _) = &**s {
                        ex(x, visit);
                    }
                }
                Else::Block { body, .. } => visit(Node::B(body)),
            }
        }
        Expr::If { branches, els, .. } => {
            for (c, b) in branches {
                match c {
                    Cond::Expr(x) | Cond::Bind { value: x, .. } => ex(x, visit),
                }
                visit(Node::B(b));
            }
            if let Some(b) = els {
                visit(Node::B(b));
            }
        }
        Expr::Match { scrutinee, arms, .. } => {
            ex(scrutinee, visit);
            arms.iter().for_each(|a| visit(Node::B(&a.body)));
        }
        _ => {}
    }
}

/// `spawn` is allowed only as `g.add(spawn f())` and `(spawn f()).wait()`
/// here; a bound one is handled by `block`.
fn spawns(e: &Expr, diags: &mut Vec<Diagnostic>) {
    match e {
        Expr::Unary { op: UnOp::Spawn, operand, pos } => {
            diags.push(
                Diagnostic::new("conc.task.5", *pos, "a task handle must be bound to a name, added to a group, or waited")
                    .help("write `t := spawn f(x)` and later `t.wait()?`, or `g.add(spawn f(x))`"),
            );
            spawns(operand, diags);
        }
        Expr::Method { recv, name, args, .. } if name == "add" && args.len() == 1 => {
            spawns(recv, diags);
            spawn_operand(&args[0].value, diags);
        }
        Expr::Method { recv, name, args, .. } if name == "wait" && args.is_empty() => spawn_operand(recv, diags),
        _ => children(e, &mut |n| match n {
            Node::E(x) => spawns(x, diags),
            Node::B(b) => block(b, diags),
        }),
    }
}

fn spawn_operand(e: &Expr, diags: &mut Vec<Diagnostic>) {
    match e {
        Expr::Unary { op: UnOp::Spawn, operand, .. } => spawns(operand, diags),
        _ => spawns(e, diags),
    }
}

/// Uses of task `name` in the statements after its `spawn`.
#[derive(Default)]
struct Uses {
    /// Where it is waited or added, and whether that is on only some paths.
    consumed: Vec<(Pos, bool)>,
    /// Any other use.
    other: Vec<Pos>,
}

fn consumed(name: &str, decl: Pos, rest: &[Stmt], diags: &mut Vec<Diagnostic>) {
    let mut u = Uses::default();
    for s in rest {
        stmt_uses(name, s, false, &mut u);
    }
    for p in &u.other {
        diags.push(
            Diagnostic::new(
                "conc.task.5",
                *p,
                format!("task `{name}` can only be waited or added to a group; a task handle can't be copied or passed"),
            )
            .help(TASK_HELP),
        );
    }
    match u.consumed.as_slice() {
        [] => diags.push(
            Diagnostic::new("conc.task.5", decl, format!("task `{name}` is never waited or added to a group"))
                .help(format!("add `{name}.wait()?` before the end of the block, or `g.add({name})`")),
        ),
        [(p, true), ..] => diags.push(
            Diagnostic::new("conc.task.5", *p, format!("task `{name}` is waited on only some paths"))
                .help("wait it, or add it to a group, outside the `if`, `match`, loop or `else`"),
        ),
        [_, rest @ ..] => {
            for (p, _) in rest {
                diags.push(
                    Diagnostic::new("conc.task.5", *p, format!("task `{name}` was already waited or added to a group"))
                        .help("a task handle is consumed once; remove this use"),
                );
            }
        }
    }
}

fn stmt_uses(name: &str, s: &Stmt, cond: bool, u: &mut Uses) {
    match s {
        Stmt::Let { value, otherwise, .. } => {
            expr_uses(name, value, cond, u);
            if let Some(o) = otherwise {
                else_uses(name, o, u);
            }
        }
        Stmt::Assign { target, value, .. } => {
            expr_uses(name, target, cond, u);
            expr_uses(name, value, cond, u);
        }
        Stmt::While { cond: c, body, .. } => {
            expr_uses(name, c, true, u);
            body.iter().for_each(|s| stmt_uses(name, s, true, u));
        }
        Stmt::For { iter, body, .. } => {
            expr_uses(name, iter, cond, u);
            body.iter().for_each(|s| stmt_uses(name, s, true, u));
        }
        // conc.share.6: the body runs once, like a plain block.
        Stmt::With { mutex, body, .. } => {
            expr_uses(name, mutex, cond, u);
            body.iter().for_each(|s| stmt_uses(name, s, cond, u));
        }
        Stmt::Return(Some(e), _) | Stmt::Expr(e) => expr_uses(name, e, cond, u),
        Stmt::Handle { value, otherwise } => {
            expr_uses(name, value, cond, u);
            else_uses(name, otherwise, u);
        }
        Stmt::Defer(inner, _) => stmt_uses(name, inner, true, u),
        Stmt::Break(_) | Stmt::Continue(_) | Stmt::Return(None, _) => {}
    }
}

fn else_uses(name: &str, o: &Else, u: &mut Uses) {
    match o {
        Else::Value(e) => expr_uses(name, e, true, u),
        Else::Leave(s) => stmt_uses(name, s, true, u),
        Else::Block { body, .. } => body.iter().for_each(|s| stmt_uses(name, s, true, u)),
    }
}

fn expr_uses(name: &str, e: &Expr, cond: bool, u: &mut Uses) {
    let is_it = |x: &Expr| matches!(x, Expr::Name(n, _) if n == name);
    match e {
        Expr::Name(n, p) if n == name => u.other.push(*p),
        Expr::Method { recv, name: m, name_pos, args } if m == "wait" && args.is_empty() && is_it(recv) => {
            u.consumed.push((*name_pos, cond));
        }
        // conc.cancel.1: cancelling a task doesn't consume its handle.
        Expr::Method { recv, name: m, args, .. } if m == "cancel" && args.is_empty() && is_it(recv) => {}
        Expr::Method { recv, name: m, args, .. } if m == "add" && args.len() == 1 && is_it(&args[0].value) => {
            expr_uses(name, recv, cond, u);
            u.consumed.push((args[0].value.pos(), cond));
        }
        // The right of `and` / `or`, and every branch, runs on only some paths.
        Expr::Binary { op: crate::ast::BinOp::And | crate::ast::BinOp::Or, lhs, rhs, .. } => {
            expr_uses(name, lhs, cond, u);
            expr_uses(name, rhs, true, u);
        }
        Expr::Else { value, otherwise, .. } => {
            expr_uses(name, value, cond, u);
            else_uses(name, otherwise, u);
        }
        Expr::If { branches, els, .. } => {
            for (i, (c, b)) in branches.iter().enumerate() {
                match c {
                    Cond::Expr(x) | Cond::Bind { value: x, .. } => expr_uses(name, x, cond || i > 0, u),
                }
                b.iter().for_each(|s| stmt_uses(name, s, true, u));
            }
            if let Some(b) = els {
                b.iter().for_each(|s| stmt_uses(name, s, true, u));
            }
        }
        Expr::Match { scrutinee, arms, .. } => {
            expr_uses(name, scrutinee, cond, u);
            for a in arms {
                a.body.iter().for_each(|s| stmt_uses(name, s, true, u));
            }
        }
        _ => children(e, &mut |n| match n {
            Node::E(x) => expr_uses(name, x, cond, u),
            Node::B(b) => b.iter().for_each(|s| stmt_uses(name, s, true, u)),
        }),
    }
}
