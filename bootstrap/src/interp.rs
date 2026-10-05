//! Reference interpreter (ADR 0032) for checked programs.
//!
//! Values have value semantics (own.val.1): structs and lists share storage
//! until one copy is mutated (own.val.2). Traps: expr.arith.1, expr.arith.3,
//! type.coll.4, type.str.4, own.excl.3, err.trap.1. Failures carry context
//! and a return trace (err.ctx.1-3), and end the program from `main`
//! (mod.main.2). Generic code runs on the values it is given (gen.impl.1);
//! trait methods dispatch on the value's type (trait.any.1), and `==`, `<`
//! and text forms use a type's `impl` or the built-in structural forms
//! (trait.std.1-4, trait.auto.1-2, trait.derive.1).

use std::collections::HashMap;
use std::fmt::{self, Write as _};
use std::io::{self, BufWriter, Write};
use std::rc::Rc;

use std::cmp::Ordering;

use crate::ast::{
    Arg, BinOp, Block, Cond, Else, Expr, FnDecl, InterpPart, Pattern, Program, Stmt, TypeKind, UnOp, place_path,
};
use crate::check::{Call, Con, Resolved};
use crate::diag::Pos;

#[derive(Clone, Debug)]
enum Value {
    Unit,
    Int(i64),
    Bool(bool),
    Str(Rc<str>),
    /// A struct (by index in the program) and its fields.
    Struct(usize, Rc<Vec<Value>>),
    /// An enum and variant, by index, and the variant's fields.
    Enum(usize, usize, Rc<Vec<Value>>),
    List(Rc<Vec<Value>>),
    /// The absent value of an optional (type.opt.4).
    None,
    /// The failure of a `T ! E` result (err.model.1).
    Fail(Rc<Failure>),
}

/// An error value with its context messages and return trace (err.ctx.1-2).
#[derive(Debug)]
struct Failure {
    error: Value,
    context: Vec<String>,
    trace: Vec<Pos>,
}

impl Value {
    fn int(&self) -> i64 {
        match self {
            Value::Int(v) => *v,
            other => unreachable!("checked: int, found {other:?}"),
        }
    }

    fn bool(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            other => unreachable!("checked: bool, found {other:?}"),
        }
    }
}

/// A trap ends the program when it reaches `main` (err.trap.3).
#[derive(Debug)]
pub struct Trap {
    pub rule: &'static str,
    pub pos: Pos,
    pub message: String,
}

impl fmt::Display for Trap {
    /// `LINE:COL: trap[RULE]: MESSAGE`, to follow a file path.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}: trap[{}]: {}", self.pos.line, self.pos.col, self.rule, self.message)
    }
}

/// How a program stopped other than by finishing.
#[derive(Debug)]
pub enum Stop {
    Trap(Trap),
    /// `main` returned an error (mod.main.2).
    Failed(Report),
}

impl From<Trap> for Stop {
    fn from(t: Trap) -> Self {
        Stop::Trap(t)
    }
}

/// An error report (err.ctx.3): the error, its context messages, and the
/// return trace, innermost first.
#[derive(Debug)]
pub struct Report {
    pub error: String,
    pub context: Vec<String>,
    pub trace: Vec<Pos>,
}

impl Report {
    pub fn render(&self, path: &str) -> String {
        let mut s = format!("error: {}", self.error);
        for c in &self.context {
            let _ = write!(s, "\n  context: {c}");
        }
        for p in &self.trace {
            let _ = write!(s, "\n  at {path}:{}:{}", p.line, p.col);
        }
        s
    }
}

/// Ways to leave a block other than reaching its end.
enum Exit {
    Break,
    Continue,
    /// A returned value, and where it was returned.
    Return(Value, Pos),
    Trap(Trap),
}

impl From<Trap> for Exit {
    fn from(t: Trap) -> Self {
        Exit::Trap(t)
    }
}

type Flow<T> = Result<T, Exit>;

fn trap<T>(rule: &'static str, pos: Pos, message: String) -> Result<T, Trap> {
    Err(Trap { rule, pos, message })
}

/// Deep recursion traps instead of crashing the interpreter.
const MAX_DEPTH: usize = 10_000;

/// Locals of one call, innermost last.
type Env<'a> = Vec<(&'a str, Value)>;

/// A place (own.place.1) resolved to a local slot and a path of steps.
#[derive(Clone, Debug)]
struct Place {
    slot: usize,
    path: Vec<Step>,
}

#[derive(Clone, Debug)]
enum Step {
    Field(usize),
    Index(i64, Pos),
}

pub fn run(prog: &Program, resolved: &Resolved) -> Result<(), Stop> {
    let mut fns = HashMap::new();
    let mut methods: HashMap<&str, HashMap<&str, &FnDecl>> = HashMap::new();
    for f in prog.fns.iter().filter(|f| f.top_level()) {
        match &f.owner {
            Some(owner) => {
                methods.entry(owner.name.as_str()).or_default().insert(f.name.as_str(), f);
            }
            None => {
                fns.insert(f.name.as_str(), f);
            }
        }
    }
    let structs = prog.structs.iter().enumerate().map(|(i, s)| (s.name.as_str(), i)).collect();
    let enums = prog.enums.iter().enumerate().map(|(i, e)| (e.name.as_str(), i)).collect();
    let stdout = io::stdout();
    let mut it = Interp {
        prog,
        resolved,
        fns,
        methods,
        structs,
        enums,
        out: BufWriter::new(stdout.lock()),
        depth: 0,
        closed: false,
    };
    let main = it.fns["main"];
    let (result, _) = it.call(main, None, Vec::new(), main.pos)?;
    let report = match result {
        Value::Fail(f) => {
            Some(Report { error: it.text(&f.error, main.pos)?, context: f.context.clone(), trace: f.trace.clone() })
        }
        _ => None,
    };
    it.flush(main.pos)?;
    match report {
        Some(r) => Err(Stop::Failed(r)),
        None => Ok(()),
    }
}

struct Interp<'a, W: Write> {
    prog: &'a Program,
    resolved: &'a Resolved,
    fns: HashMap<&'a str, &'a FnDecl>,
    methods: HashMap<&'a str, HashMap<&'a str, &'a FnDecl>>,
    structs: HashMap<&'a str, usize>,
    enums: HashMap<&'a str, usize>,
    out: BufWriter<W>,
    depth: usize,
    /// Standard output was closed by the reader (e.g. `| head`); stop writing quietly.
    closed: bool,
}

fn lookup<'e>(env: &'e Env<'_>, name: &str) -> Option<&'e Value> {
    env.iter().rev().find(|(n, _)| *n == name).map(|(_, v)| v)
}

/// The type or trait an expression names, when it isn't a value: `Shape`
/// in `Shape.Circle(r: 1)`, or `Stack` in `Stack[int].new()`.
fn type_head<'e>(e: &'e Expr, env: &Env<'_>) -> Option<&'e str> {
    match e {
        Expr::Name(n, _) if lookup(env, n).is_none() => Some(n),
        Expr::TypeArgs { name, .. } => Some(name),
        _ => None,
    }
}

fn element(items: &[Value], i: i64, pos: Pos) -> Result<&Value, Trap> {
    match usize::try_from(i).ok().and_then(|u| items.get(u)) {
        Some(v) => Ok(v),
        None => trap("type.coll.4", pos, format!("index {i} is out of bounds for a list of length {}", items.len())),
    }
}

/// The storage a place refers to, copying shared storage first (own.val.2).
fn slot_mut<'e>(env: &'e mut Env<'_>, p: &Place) -> Result<&'e mut Value, Trap> {
    let mut v = &mut env[p.slot].1;
    for step in &p.path {
        v = match (step, v) {
            (Step::Field(i), Value::Struct(_, fields)) => &mut Rc::make_mut(fields)[*i],
            (Step::Index(i, pos), Value::List(items)) => {
                let items = Rc::make_mut(items);
                let len = items.len();
                match usize::try_from(*i).ok().filter(|u| *u < len) {
                    Some(u) => &mut items[u],
                    None => {
                        return trap(
                            "type.coll.4",
                            *pos,
                            format!("index {i} is out of bounds for a list of length {len}"),
                        );
                    }
                }
            }
            (_, other) => unreachable!("checked: place step {step:?} on {other:?}"),
        };
    }
    Ok(v)
}

/// A call operand: a place, or the value of any other expression.
enum Operand {
    Place(Place),
    Value(Value),
}

/// The value at a place, without copying it.
fn peek<'e>(env: &'e Env<'_>, p: &Place) -> Result<&'e Value, Trap> {
    let mut v = &env[p.slot].1;
    for step in &p.path {
        v = match (step, v) {
            (Step::Field(i), Value::Struct(_, fields)) => &fields[*i],
            (Step::Index(i, pos), Value::List(items)) => element(items, *i, *pos)?,
            (_, other) => unreachable!("checked: place step {step:?} on {other:?}"),
        };
    }
    Ok(v)
}

fn load(env: &Env<'_>, p: &Place) -> Result<Value, Trap> {
    peek(env, p).cloned()
}

/// An operand's place (if it is one) and, unless it is passed `mut`, its value.
/// Reading read-only operands before any `mut` one is moved out is what makes
/// a read-only argument that overlaps a `mut` one a copy (own.excl.4).
type Prepared = (Option<Place>, Option<Value>, bool);

fn prepare(op: Operand, mutable: bool, env: &Env<'_>) -> Result<Prepared, Trap> {
    Ok(match op {
        Operand::Place(p) => {
            let value = if mutable { None } else { Some(load(env, &p)?) };
            (Some(p), value, mutable)
        }
        Operand::Value(v) => (None, Some(v), mutable),
    })
}

/// own.excl.3: `mut` operands that overlap at run time trap before the call.
fn check_overlap(places: &[(Option<Place>, bool)], pos: Pos) -> Result<(), Trap> {
    for (j, (b, b_mut)) in places.iter().enumerate() {
        for (a, a_mut) in &places[..j] {
            if let (Some(a), Some(b)) = (a, b)
                && *a_mut
                && *b_mut
                && overlaps(a, b)
            {
                return trap("own.excl.3", pos, "two `mut` arguments of this call overlap".into());
            }
        }
    }
    Ok(())
}

/// Two places overlap when one is the other or contains it (own.excl.1).
fn overlaps(a: &Place, b: &Place) -> bool {
    a.slot == b.slot
        && a.path.iter().zip(&b.path).all(|(x, y)| match (x, y) {
            (Step::Field(i), Step::Field(j)) => i == j,
            (Step::Index(i, _), Step::Index(j, _)) => i == j,
            _ => false,
        })
}

impl<'a, W: Write> Interp<'a, W> {
    /// Call `f`. Returns its result and the final values of its receiver and
    /// parameters, for writing `mut` ones back to the caller.
    fn call(
        &mut self,
        f: &'a FnDecl,
        recv: Option<Value>,
        args: Vec<Value>,
        pos: Pos,
    ) -> Result<(Value, Vec<Value>), Trap> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return trap("err.trap.1", pos, format!("call depth exceeded {MAX_DEPTH} in `{}`", f.full_name()));
        }
        let mut env: Env<'a> = Vec::new();
        if let Some(r) = recv {
            env.push(("self", r));
        }
        env.extend(f.params.iter().map(|p| p.name.as_str()).zip(args));
        let n = env.len();
        let body = f.body.as_ref().expect("checked: the function has a body");
        let end = body.last().map_or(f.pos, Stmt::pos);
        let (mut result, at) = match self.block(body, &mut env) {
            Ok(v) => (v, end),
            Err(Exit::Return(v, at)) => (v, at),
            Err(Exit::Trap(t)) => return Err(t),
            Err(Exit::Break | Exit::Continue) => unreachable!("checked: `break` and `continue` are inside loops"),
        };
        // err.decl.3: an error value leaving a fallible function is its failure.
        if matches!(f.ret.as_ref().map(|t| &t.kind), Some(TypeKind::Fallible { .. })) && self.is_error(&result) {
            result = Value::Fail(Rc::new(Failure { error: result, context: Vec::new(), trace: vec![at] }));
        }
        self.depth -= 1;
        env.truncate(n);
        Ok((result, env.into_iter().map(|(_, v)| v).collect()))
    }

    fn block(&mut self, stmts: &'a Block, env: &mut Env<'a>) -> Flow<Value> {
        let mark = env.len();
        let mut result = Ok(Value::Unit);
        for s in stmts {
            result = self.stmt(s, env);
            if result.is_err() {
                break;
            }
        }
        env.truncate(mark);
        result
    }

    fn stmt(&mut self, s: &'a Stmt, env: &mut Env<'a>) -> Flow<Value> {
        match s {
            Stmt::Let { name, value, otherwise, .. } => {
                let v = self.eval(value, env)?;
                let v = match (otherwise, v) {
                    (Some(o), v @ (Value::None | Value::Fail(_))) => self.otherwise(o, v, env)?,
                    (_, v) => v,
                };
                env.push((name.as_str(), v));
            }
            Stmt::Assign { target, op, value, pos } => self.assign(target, *op, value, *pos, env)?,
            Stmt::While { cond, body, .. } => {
                while self.eval(cond, env)?.bool() {
                    match self.block(body, env) {
                        Ok(_) | Err(Exit::Continue) => {}
                        Err(Exit::Break) => break,
                        Err(e) => return Err(e),
                    }
                }
            }
            Stmt::For { var, mutable: true, iter, body, .. } => self.for_mut(var, iter, body, env)?,
            Stmt::For { var, iter, body, .. } => self.for_loop(var, iter, body, env)?,
            Stmt::Break(_) => return Err(Exit::Break),
            Stmt::Continue(_) => return Err(Exit::Continue),
            Stmt::Return(value, pos) => {
                let v = match value {
                    Some(e) => self.eval(e, env)?,
                    None => Value::Unit,
                };
                return Err(Exit::Return(v, *pos));
            }
            Stmt::Expr(e) => return self.eval(e, env),
        }
        Ok(Value::Unit)
    }

    /// The `else` part of a binding whose value is absent or failed (err.else.1-3).
    fn otherwise(&mut self, o: &'a Else, failed: Value, env: &mut Env<'a>) -> Flow<Value> {
        match o {
            Else::Value(d) => self.eval(d, env),
            Else::Leave(stmt) => {
                self.stmt(stmt, env)?;
                unreachable!("checked: the statement leaves")
            }
            Else::Block { err, body, .. } => {
                let mark = env.len();
                if let (Some((name, _)), Value::Fail(f)) = (err, &failed) {
                    env.push((name.as_str(), f.error.clone()));
                }
                let r = self.block(body, env);
                env.truncate(mark);
                r
            }
        }
    }

    /// `for mut x in e`: each element is moved into `x` for one iteration and
    /// written back afterwards, however the iteration ends (expr.loop.6).
    fn for_mut(&mut self, var: &'a str, iter: &'a Expr, body: &'a Block, env: &mut Env<'a>) -> Flow<()> {
        let list = self.place(iter, env)?;
        let Value::List(items) = peek(env, &list)? else { unreachable!("checked: list") };
        for i in 0..items.len() {
            let mut at = list.clone();
            at.path.push(Step::Index(i as i64, iter.pos()));
            let v = std::mem::replace(slot_mut(env, &at)?, Value::Unit);
            env.push((var, v));
            let r = self.block(body, env);
            let (_, v) = env.pop().expect("the loop variable");
            *slot_mut(env, &at)? = v;
            match r {
                Ok(_) | Err(Exit::Continue) => {}
                Err(Exit::Break) => break,
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    /// `for x in e` iterates the value `e` had when the loop started (expr.loop.4).
    fn for_loop(&mut self, var: &'a str, iter: &'a Expr, body: &'a Block, env: &mut Env<'a>) -> Flow<()> {
        if let Expr::Range { lo, hi, inclusive, .. } = iter {
            let lo = self.eval(lo, env)?.int();
            let hi = self.eval(hi, env)?.int();
            let mut i = lo;
            while if *inclusive { i <= hi } else { i < hi } {
                if !self.loop_body(var, Value::Int(i), body, env)? || i == i64::MAX {
                    break;
                }
                i += 1;
            }
            return Ok(());
        }
        let Value::List(items) = self.eval(iter, env)? else { unreachable!("checked: list") };
        for item in items.iter() {
            if !self.loop_body(var, item.clone(), body, env)? {
                break;
            }
        }
        Ok(())
    }

    /// Run one iteration; false if the loop should stop.
    fn loop_body(&mut self, var: &'a str, v: Value, body: &'a Block, env: &mut Env<'a>) -> Flow<bool> {
        env.push((var, v));
        let r = self.block(body, env);
        env.pop();
        match r {
            Ok(_) | Err(Exit::Continue) => Ok(true),
            Err(Exit::Break) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// `p = e` and `p op= e`, which means `p = p op e` (expr.op.2).
    fn assign(
        &mut self,
        target: &'a Expr,
        op: Option<BinOp>,
        value: &'a Expr,
        pos: Pos,
        env: &mut Env<'a>,
    ) -> Flow<()> {
        let p = self.place(target, env)?;
        let v = match op {
            None => self.eval(value, env)?,
            Some(op) => {
                let old = load(env, &p)?;
                let rhs = self.eval(value, env)?;
                self.binop(op, old, rhs, pos)?
            }
        };
        *slot_mut(env, &p)? = v;
        Ok(())
    }

    fn place(&mut self, e: &'a Expr, env: &mut Env<'a>) -> Flow<Place> {
        match e {
            Expr::Name(n, _) => {
                let slot = env.iter().rposition(|(m, _)| m == n).expect("checked: a local");
                Ok(Place { slot, path: Vec::new() })
            }
            Expr::Field { recv, name, .. } => {
                let mut p = self.place(recv, env)?;
                let Value::Struct(s, _) = peek(env, &p)? else { unreachable!("checked: struct") };
                p.path.push(Step::Field(self.field_index(*s, name)));
                Ok(p)
            }
            Expr::Index { recv, index, pos } => {
                let mut p = self.place(recv, env)?;
                let i = self.eval(index, env)?.int();
                p.path.push(Step::Index(i, *pos));
                Ok(p)
            }
            _ => unreachable!("checked: a place"),
        }
    }

    /// Evaluate a call operand: a place (rooted in a local) stays a place.
    fn operand(&mut self, e: &'a Expr, env: &mut Env<'a>) -> Flow<Operand> {
        if place_path(e).is_some_and(|(root, _)| lookup(env, root).is_some()) {
            Ok(Operand::Place(self.place(e, env)?))
        } else {
            Ok(Operand::Value(self.eval(e, env)?))
        }
    }

    fn field_index(&self, s: usize, name: &str) -> usize {
        self.prog.structs[s].fields.iter().position(|f| f.name == name).expect("checked: field exists")
    }

    fn eval(&mut self, e: &'a Expr, env: &mut Env<'a>) -> Flow<Value> {
        Ok(match e {
            Expr::Str(s, _) => Value::Str(Rc::from(s.as_str())),
            Expr::Int(v, _) => Value::Int(*v),
            Expr::Bool(b, _) => Value::Bool(*b),
            Expr::None(_) => Value::None,
            Expr::Try { expr, context, pos } => {
                let v = self.eval(expr, env)?;
                match v {
                    Value::None => return Err(Exit::Return(Value::None, *pos)),
                    Value::Fail(f) => {
                        let mut context_list = f.context.clone();
                        if let Some(c) = context {
                            let Value::Str(msg) = self.eval(c, env)? else { unreachable!("checked: str") };
                            context_list.push(msg.to_string());
                        }
                        let mut trace = f.trace.clone();
                        trace.push(*pos);
                        let failure = Failure { error: f.error.clone(), context: context_list, trace };
                        return Err(Exit::Return(Value::Fail(Rc::new(failure)), *pos));
                    }
                    v => v,
                }
            }
            Expr::Interp(parts, _) => {
                let mut s = String::new();
                for part in parts {
                    match part {
                        InterpPart::Lit(text) => s.push_str(text),
                        InterpPart::Expr(x) => {
                            let v = self.eval(x, env)?;
                            s.push_str(&self.text(&v, x.pos())?);
                        }
                    }
                }
                Value::Str(Rc::from(s))
            }
            Expr::Name(n, pos) => match lookup(env, n) {
                Some(v) => v.clone(),
                None => {
                    let (e, v) = self.resolved.variants[pos];
                    Value::Enum(e, v, Rc::new(Vec::new()))
                }
            },
            Expr::List(elems, _) => {
                let mut items = Vec::with_capacity(elems.len());
                for x in elems {
                    items.push(self.eval(x, env)?);
                }
                Value::List(Rc::new(items))
            }
            Expr::Call { callee, args, pos } => self.call_expr(callee, args, *pos, env)?,
            Expr::Method { recv, name, name_pos, args } => self.method_call(recv, name, args, *name_pos, env)?,
            Expr::TypeArgs { .. } => unreachable!("checked: type arguments only on a callee or receiver"),
            Expr::Field { recv, name, .. } => {
                if let Some(tn) = type_head(recv, env) {
                    let e = self.enums[tn];
                    let v = self.variant_index(e, name);
                    return Ok(Value::Enum(e, v, Rc::new(Vec::new())));
                }
                let Value::Struct(s, fields) = self.eval(recv, env)? else { unreachable!("checked: struct") };
                fields[self.field_index(s, name)].clone()
            }
            Expr::Index { recv, index, pos } => {
                let Value::List(items) = self.eval(recv, env)? else { unreachable!("checked: list") };
                if let Expr::Range { lo, hi, inclusive, pos: range_pos } = &**index {
                    let lo = self.eval(lo, env)?.int();
                    let hi = self.eval(hi, env)?.int();
                    return Ok(Value::List(Rc::new(self.slice(&items, lo, hi, *inclusive, *range_pos)?)));
                }
                let i = self.eval(index, env)?.int();
                element(&items, i, *pos)?.clone()
            }
            Expr::Unary { op: UnOp::Neg, operand, pos } => {
                let v = self.eval(operand, env)?.int();
                match v.checked_neg() {
                    Some(r) => Value::Int(r),
                    None => {
                        let message = format!("`-({v})` overflows `int`");
                        return Err(Exit::Trap(Trap { rule: "expr.arith.1", pos: *pos, message }));
                    }
                }
            }
            Expr::Unary { op: UnOp::Not, operand, .. } => Value::Bool(!self.eval(operand, env)?.bool()),
            Expr::Binary { op: BinOp::And, lhs, rhs, .. } => {
                Value::Bool(self.eval(lhs, env)?.bool() && self.eval(rhs, env)?.bool())
            }
            Expr::Binary { op: BinOp::Or, lhs, rhs, .. } => {
                Value::Bool(self.eval(lhs, env)?.bool() || self.eval(rhs, env)?.bool())
            }
            Expr::Binary { op, lhs, rhs, pos } => {
                let l = self.eval(lhs, env)?;
                let r = self.eval(rhs, env)?;
                self.binop(*op, l, r, *pos)?
            }
            Expr::Range { .. } => unreachable!("checked: ranges only in `for` and slices"),
            Expr::If { branches, els, else_err, .. } => {
                let mut failure = None;
                for (cond, body) in branches {
                    match cond {
                        Cond::Expr(c) => {
                            if self.eval(c, env)?.bool() {
                                return self.block(body, env);
                            }
                        }
                        // expr.if.5: run the branch with the value bound, if there is one.
                        Cond::Bind { name, value, .. } => match self.eval(value, env)? {
                            Value::None => {}
                            Value::Fail(f) => failure = Some(f),
                            v => {
                                let mark = env.len();
                                env.push((name.as_str(), v));
                                let r = self.block(body, env);
                                env.truncate(mark);
                                return r;
                            }
                        },
                    }
                }
                let Some(body) = els else { return Ok(Value::Unit) };
                let mark = env.len();
                if let (Some((name, _)), Some(f)) = (else_err, failure) {
                    env.push((name.as_str(), f.error.clone()));
                }
                let r = self.block(body, env);
                env.truncate(mark);
                return r;
            }
            Expr::Match { scrutinee, arms, .. } => {
                let Value::Enum(e, v, fields) = self.eval(scrutinee, env)? else { unreachable!("checked: enum") };
                let variant = &self.prog.enums[e].variants[v].name;
                for arm in arms {
                    let mark = env.len();
                    match &arm.pat {
                        Pattern::Wildcard(_) => {}
                        Pattern::Variant { name, .. } if name != variant => continue,
                        Pattern::Variant { bindings, .. } => {
                            // expr.match.6: bind fields by position; `_` skips one.
                            for ((n, _), value) in bindings.iter().flatten().zip(fields.iter()) {
                                if n != "_" {
                                    env.push((n.as_str(), value.clone()));
                                }
                            }
                        }
                    }
                    let r = self.block(&arm.body, env);
                    env.truncate(mark);
                    return r;
                }
                unreachable!("checked: exhaustive match")
            }
        })
    }

    fn slice(&self, items: &[Value], lo: i64, hi: i64, inclusive: bool, pos: Pos) -> Result<Vec<Value>, Trap> {
        let end = if inclusive { hi.checked_add(1) } else { Some(hi) };
        let dots = if inclusive { "..=" } else { ".." };
        match (usize::try_from(lo), end.and_then(|e| usize::try_from(e).ok())) {
            (Ok(a), Some(b)) if a <= b && b <= items.len() => Ok(items[a..b].to_vec()),
            _ => trap(
                "type.coll.4",
                pos,
                format!("slice {lo}{dots}{hi} is out of bounds for a list of length {}", items.len()),
            ),
        }
    }

    /// Integer arithmetic traps instead of overflowing (expr.arith.1, expr.arith.3, expr.arith.5).
    fn binop(&mut self, op: BinOp, l: Value, r: Value, pos: Pos) -> Result<Value, Trap> {
        let sym = op.symbol();
        Ok(match (l, r) {
            (Value::Int(a), Value::Int(b)) if op.is_arithmetic() => {
                let result = match op {
                    BinOp::Add => a.checked_add(b),
                    BinOp::Sub => a.checked_sub(b),
                    BinOp::Mul => a.checked_mul(b),
                    _ if b == 0 => return trap("expr.arith.3", pos, format!("division by zero in `{a} {sym} 0`")),
                    BinOp::Div => a.checked_div(b),
                    _ => Some(a.wrapping_rem(b)), // exact: `a % -1` is 0 for every `a`
                };
                let rule = if op == BinOp::Div { "expr.arith.3" } else { "expr.arith.1" };
                match result {
                    Some(v) => Value::Int(v),
                    None => return trap(rule, pos, format!("`{a} {sym} {b}` overflows `int`")),
                }
            }
            (Value::Int(a), Value::Int(b)) => Value::Bool(match op {
                BinOp::Eq => a == b,
                BinOp::Ne => a != b,
                BinOp::Lt => a < b,
                BinOp::Le => a <= b,
                BinOp::Gt => a > b,
                BinOp::Ge => a >= b,
                _ => unreachable!("arithmetic handled above"),
            }),
            (l, r) => match op {
                BinOp::Eq => Value::Bool(self.equal(&l, &r, pos)?),
                BinOp::Ne => Value::Bool(!self.equal(&l, &r, pos)?),
                _ => {
                    let o = self.compare(&l, &r, pos)?;
                    Value::Bool(match op {
                        BinOp::Lt => o.is_lt(),
                        BinOp::Le => o.is_le(),
                        BinOp::Gt => o.is_gt(),
                        _ => o.is_ge(),
                    })
                }
            },
        })
    }

    /// The function implementing trait `tr`'s method `name` for `v`: from
    /// its type's `impl` block, or else the trait's default. `None` means the
    /// built-in implementation of a standard trait (trait.auto.1-2,
    /// trait.derive.1).
    fn dispatch(&self, tr: usize, name: &str, v: &Value) -> Option<&'a FnDecl> {
        let prog = self.prog;
        let con = match v {
            Value::Int(_) => Some(Con::Int),
            Value::Str(_) => Some(Con::Str),
            Value::Bool(_) => Some(Con::Bool),
            Value::List(_) => Some(Con::List),
            Value::Struct(s, _) => Some(Con::Struct(*s)),
            Value::Enum(e, _, _) => Some(Con::Enum(*e)),
            _ => None,
        };
        if let Some(con) = con
            && let Some(&i) = self.resolved.impls.get(&(tr, con))
            && let Some(&m) = prog.impls[i].methods.iter().find(|&&m| prog.fns[m].name == name)
        {
            return Some(&prog.fns[m]);
        }
        let m = prog.traits[tr].methods.iter().copied().find(|&m| prog.fns[m].name == name)?;
        prog.fns[m].body.is_some().then(|| &prog.fns[m])
    }

    /// `a == b` (trait.std.1): the type's `eq`, or structural equality (trait.auto.1).
    fn equal(&mut self, a: &Value, b: &Value, pos: Pos) -> Result<bool, Trap> {
        if let Some(f) = self.dispatch(self.resolved.std.eq, "eq", a) {
            let (r, _) = self.call(f, Some(a.clone()), vec![b.clone()], pos)?;
            return Ok(r.bool());
        }
        Ok(match (a, b) {
            (Value::Int(x), Value::Int(y)) => x == y,
            (Value::Bool(x), Value::Bool(y)) => x == y,
            (Value::Str(x), Value::Str(y)) => x == y,
            (Value::None, Value::None) => true,
            (Value::None, _) | (_, Value::None) => false,
            (Value::List(x), Value::List(y)) | (Value::Struct(_, x), Value::Struct(_, y)) => {
                self.all_equal(x, y, pos)?
            }
            (Value::Enum(e, v, x), Value::Enum(f, w, y)) => e == f && v == w && self.all_equal(x, y, pos)?,
            (x, y) => unreachable!("checked: comparing {x:?} and {y:?}"),
        })
    }

    fn all_equal(&mut self, x: &[Value], y: &[Value], pos: Pos) -> Result<bool, Trap> {
        if x.len() != y.len() {
            return Ok(false);
        }
        for (p, q) in x.iter().zip(y) {
            if !self.equal(p, q, pos)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// `a.cmp(b)` (trait.std.2): the type's `cmp`, or the built-in order
    /// (trait.std.4, trait.derive.1).
    fn compare(&mut self, a: &Value, b: &Value, pos: Pos) -> Result<Ordering, Trap> {
        if let Some(f) = self.dispatch(self.resolved.std.ord, "cmp", a) {
            let (r, _) = self.call(f, Some(a.clone()), vec![b.clone()], pos)?;
            let Value::Enum(_, v, _) = r else { unreachable!("checked: `cmp` returns an `Ordering`") };
            return Ok([Ordering::Less, Ordering::Equal, Ordering::Greater][v]);
        }
        Ok(match (a, b) {
            (Value::Int(x), Value::Int(y)) => x.cmp(y),
            (Value::Str(x), Value::Str(y)) => x.as_bytes().cmp(y.as_bytes()),
            (Value::List(x), Value::List(y)) | (Value::Struct(_, x), Value::Struct(_, y)) => {
                self.lexicographic(x, y, pos)?
            }
            (Value::Enum(_, v, x), Value::Enum(_, w, y)) => match v.cmp(w) {
                Ordering::Equal => self.lexicographic(x, y, pos)?,
                o => o,
            },
            (x, y) => unreachable!("checked: ordering {x:?} and {y:?}"),
        })
    }

    fn lexicographic(&mut self, x: &[Value], y: &[Value], pos: Pos) -> Result<Ordering, Trap> {
        for (p, q) in x.iter().zip(y) {
            match self.compare(p, q, pos)? {
                Ordering::Equal => {}
                o => return Ok(o),
            }
        }
        Ok(x.len().cmp(&y.len()))
    }

    fn ordering_value(&self, o: Ordering) -> Value {
        let v = match o {
            Ordering::Less => 0,
            Ordering::Equal => 1,
            Ordering::Greater => 2,
        };
        Value::Enum(self.resolved.std.ordering, v, Rc::new(Vec::new()))
    }

    fn call_expr(&mut self, callee: &'a Expr, args: &'a [Arg], pos: Pos, env: &mut Env<'a>) -> Flow<Value> {
        let n = match callee {
            Expr::Name(n, _) | Expr::TypeArgs { name: n, .. } => n,
            _ => unreachable!("checked: named callee"),
        };
        if n == "print" {
            let v = self.eval(&args[0].value, env)?;
            self.print(&v, pos)?;
            return Ok(Value::Unit);
        }
        if let Some(&s) = self.structs.get(n.as_str()) {
            let fields = self.labelled(&self.prog.structs[s].fields, args, env)?;
            return Ok(Value::Struct(s, Rc::new(fields)));
        }
        if let Some(&(e, v)) = self.resolved.variants.get(&callee.pos()) {
            let fields = self.labelled(&self.prog.enums[e].variants[v].fields, args, env)?;
            return Ok(Value::Enum(e, v, Rc::new(fields)));
        }
        let f = self.fns[n.as_str()];
        self.invoke(f, None, args, callee.pos(), env)
    }

    /// Field values in declaration order from `field: value` arguments,
    /// evaluated in the order written (type.struct.3, type.enum.6).
    fn labelled(&mut self, fields: &'a [crate::ast::Field], args: &'a [Arg], env: &mut Env<'a>) -> Flow<Vec<Value>> {
        let mut values = vec![Value::Unit; fields.len()];
        for a in args {
            let (label, _) = a.label.as_ref().expect("checked: labelled");
            let i = fields.iter().position(|f| f.name == *label).expect("checked: field exists");
            values[i] = self.eval(&a.value, env)?;
        }
        Ok(values)
    }

    fn variant_index(&self, e: usize, name: &str) -> usize {
        self.prog.enums[e].variants.iter().position(|v| v.name == name).expect("checked: variant exists")
    }

    fn is_error(&self, v: &Value) -> bool {
        match v {
            Value::Struct(s, _) => self.prog.structs[*s].is_error,
            Value::Enum(e, _, _) => self.prog.enums[*e].is_error,
            _ => false,
        }
    }

    /// Text form, as used by `print`, interpolation and error reports
    /// (type.str.3, trait.std.3, err.text.1).
    fn text(&mut self, v: &Value, pos: Pos) -> Result<String, Trap> {
        match v {
            Value::Unit => Ok(String::new()),
            Value::Str(s) => Ok(s.to_string()),
            other => self.describe(other, pos),
        }
    }

    /// A value's text form inside another (trait.auto.2): its `to_str`, or
    /// `Type.Variant(field: value, ...)`, with strings quoted.
    fn describe(&mut self, v: &Value, pos: Pos) -> Result<String, Trap> {
        if let Some(f) = self.dispatch(self.resolved.std.to_str, "to_str", v) {
            let (r, _) = self.call(f, Some(v.clone()), Vec::new(), pos)?;
            let Value::Str(s) = r else { unreachable!("checked: `to_str` returns a `str`") };
            return Ok(s.to_string());
        }
        let prog = self.prog;
        Ok(match v {
            Value::Unit => "()".into(),
            Value::Int(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Str(s) => format!("{:?}", &**s),
            Value::None => "none".into(),
            Value::List(items) => {
                let mut parts = Vec::with_capacity(items.len());
                for x in items.iter() {
                    parts.push(self.describe(x, pos)?);
                }
                format!("[{}]", parts.join(", "))
            }
            Value::Struct(s, values) => {
                let decl = &prog.structs[*s];
                let names = decl.fields.iter().map(|f| f.name.as_str());
                format!("{}{}", decl.name, self.fields_text(names, values, pos)?)
            }
            Value::Enum(e, i, values) => {
                let decl = &prog.enums[*e];
                let variant = &decl.variants[*i];
                let names = variant.fields.iter().map(|f| f.name.as_str());
                format!("{}.{}{}", decl.name, variant.name, self.fields_text(names, values, pos)?)
            }
            Value::Fail(f) => self.describe(&f.error, pos)?,
        })
    }

    /// `(name: value, ...)`, or nothing when there are no fields.
    fn fields_text<'n>(
        &mut self,
        names: impl Iterator<Item = &'n str>,
        values: &[Value],
        pos: Pos,
    ) -> Result<String, Trap> {
        if values.is_empty() {
            return Ok(String::new());
        }
        let mut parts = Vec::with_capacity(values.len());
        for (n, x) in names.zip(values) {
            parts.push(format!("{n}: {}", self.describe(x, pos)?));
        }
        Ok(format!("({})", parts.join(", ")))
    }

    fn method_call(
        &mut self,
        recv: &'a Expr,
        name: &'a str,
        args: &'a [Arg],
        pos: Pos,
        env: &mut Env<'a>,
    ) -> Flow<Value> {
        let call = self.resolved.calls.get(&pos).copied();
        // `Trait.name(x, ...)` (trait.res.1).
        if let Some(Call::Qualified(tr)) = call {
            let (first, rest) = args.split_first().expect("checked: a receiver argument");
            let op = self.operand(&first.value, env)?;
            return self.trait_method(tr, name, op, rest, pos, env);
        }
        // `Type.Variant(field: value)` (type.enum.6), or an associated
        // function `Type.name(args)` (trait.method.4).
        if let Some(tn) = type_head(recv, env) {
            if let Some(&e) = self.enums.get(tn)
                && let Some(v) = self.prog.enums[e].variants.iter().position(|v| v.name == name)
            {
                let fields = self.labelled(&self.prog.enums[e].variants[v].fields, args, env)?;
                return Ok(Value::Enum(e, v, Rc::new(fields)));
            }
            let f = self.methods[tn][name];
            return self.invoke(f, None, args, pos, env);
        }
        let recv = self.operand(recv, env)?;
        if let Some(Call::Trait(tr)) = call {
            return self.trait_method(tr, name, recv, args, pos, env);
        }
        let value = match &recv {
            Operand::Place(p) => peek(env, p)?,
            Operand::Value(v) => v,
        };
        let type_name = match value {
            Value::List(items) => {
                let len = items.len();
                return self.list_method(len, recv, name, args, pos, env);
            }
            Value::Str(s) => {
                let s = s.clone();
                return self.str_method(s, name, args, pos, env);
            }
            Value::Struct(s, _) => self.prog.structs[*s].name.as_str(),
            Value::Enum(e, _, _) => self.prog.enums[*e].name.as_str(),
            other => unreachable!("checked: no methods on {other:?}"),
        };
        let f = self.methods[type_name][name];
        self.invoke(f, Some(recv), args, pos, env)
    }

    /// A trait's method called on `recv`: the implementation for its type,
    /// the trait's default, or the built-in `eq`, `cmp` or `to_str`.
    fn trait_method(
        &mut self,
        tr: usize,
        name: &str,
        recv: Operand,
        args: &'a [Arg],
        pos: Pos,
        env: &mut Env<'a>,
    ) -> Flow<Value> {
        let value = match &recv {
            Operand::Place(p) => peek(env, p)?.clone(),
            Operand::Value(v) => v.clone(),
        };
        if let Some(f) = self.dispatch(tr, name, &value) {
            return self.invoke(f, Some(recv), args, pos, env);
        }
        let std = self.resolved.std;
        Ok(if tr == std.to_str {
            Value::Str(Rc::from(self.text(&value, pos)?))
        } else {
            let other = self.eval(&args[0].value, env)?;
            if tr == std.eq {
                Value::Bool(self.equal(&value, &other, pos)?)
            } else {
                let o = self.compare(&value, &other, pos)?;
                self.ordering_value(o)
            }
        })
    }

    /// `s.split(sep)` and `s.to_int()` (type.str.4-5).
    fn str_method(&mut self, s: Rc<str>, name: &str, args: &'a [Arg], pos: Pos, env: &mut Env<'a>) -> Flow<Value> {
        if name == "split" {
            let Value::Str(sep) = self.eval(&args[0].value, env)? else { unreachable!("checked: str") };
            if sep.is_empty() {
                let message = "`split` needs a non-empty separator".into();
                return Err(Exit::Trap(Trap { rule: "type.str.4", pos, message }));
            }
            let pieces = s.split(&*sep).map(|p| Value::Str(Rc::from(p))).collect();
            return Ok(Value::List(Rc::new(pieces)));
        }
        let digits = s.strip_prefix('-').unwrap_or(&s);
        let valid = !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit());
        Ok(match s.parse::<i64>() {
            Ok(n) if valid => Value::Int(n),
            _ => Value::None,
        })
    }

    /// `xs.len()`, `xs.push(v)` and `xs.swap(i, j)` (type.coll.5-7).
    fn list_method(
        &mut self,
        len: usize,
        recv: Operand,
        name: &str,
        args: &'a [Arg],
        pos: Pos,
        env: &mut Env<'a>,
    ) -> Flow<Value> {
        if name == "len" {
            return Ok(Value::Int(len as i64));
        }
        let Operand::Place(list) = recv else { unreachable!("checked: `{name}` needs a mutable place") };
        // Arguments are read before the list changes, so they are copies (own.excl.4).
        let mut values = Vec::with_capacity(args.len());
        for a in args {
            values.push(self.eval(&a.value, env)?);
        }
        let Value::List(items) = slot_mut(env, &list)? else { unreachable!("checked: list") };
        match name {
            "push" => Rc::make_mut(items).push(values.pop().expect("checked: one argument")),
            _ => {
                let (i, j) = (values[0].int(), values[1].int());
                element(items, i, pos)?;
                element(items, j, pos)?;
                Rc::make_mut(items).swap(i as usize, j as usize);
            }
        }
        Ok(Value::Unit)
    }

    /// Call `f`. Operands are evaluated left to right, and read-only ones are
    /// read immediately. After the overlap check (own.excl.3), `mut` receivers
    /// and arguments are moved out of their places for the call and written
    /// back afterwards (trait.method.3, own.param.2), so the callee mutates
    /// them without copying.
    fn invoke(
        &mut self,
        f: &'a FnDecl,
        recv: Option<Operand>,
        args: &'a [Arg],
        pos: Pos,
        env: &mut Env<'a>,
    ) -> Flow<Value> {
        let has_recv = recv.is_some();
        let mut ops = Vec::with_capacity(args.len() + 1);
        if let Some(r) = recv {
            ops.push(prepare(r, f.receiver.is_some_and(|r| r.mutable), env)?);
        }
        for (a, param) in args.iter().zip(&f.params) {
            let op = self.operand(&a.value, env)?;
            ops.push(prepare(op, param.mutable, env)?);
        }
        let places: Vec<(Option<Place>, bool)> = ops.iter().map(|(p, _, m)| (p.clone(), *m)).collect();
        check_overlap(&places, pos)?;
        let mut values = Vec::with_capacity(ops.len());
        for (place, value, _) in &mut ops {
            values.push(match value.take() {
                Some(v) => v,
                None => std::mem::replace(
                    slot_mut(env, place.as_ref().expect("checked: `mut` operands are places"))?,
                    Value::Unit,
                ),
            });
        }
        let recv_value = if has_recv { Some(values.remove(0)) } else { None };
        let (result, finals) = self.call(f, recv_value, values, pos)?;
        for ((place, _, mutable), v) in ops.into_iter().zip(finals) {
            if mutable {
                *slot_mut(env, &place.expect("checked: `mut` operands are places"))? = v;
            }
        }
        Ok(result)
    }

    fn print(&mut self, v: &Value, pos: Pos) -> Result<(), Trap> {
        let text = self.text(v, pos)?;
        if self.closed {
            return Ok(());
        }
        match writeln!(self.out, "{text}") {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == io::ErrorKind::BrokenPipe => {
                self.closed = true;
                Ok(())
            }
            Err(e) => trap("mod.main.3", pos, format!("writing to standard output failed: {e}")),
        }
    }

    fn flush(&mut self, pos: Pos) -> Result<(), Trap> {
        match self.out.flush() {
            Err(e) if e.kind() != io::ErrorKind::BrokenPipe => {
                trap("mod.main.3", pos, format!("writing to standard output failed: {e}"))
            }
            _ => Ok(()),
        }
    }
}
