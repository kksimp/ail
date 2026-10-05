//! Reference interpreter (ADR 0032) for checked programs.
//!
//! Values have value semantics (own.val.1): structs and lists share storage
//! until one copy is mutated (own.val.2). Traps: expr.arith.1, expr.arith.3,
//! type.coll.4, own.excl.3, err.trap.1.

use std::collections::HashMap;
use std::fmt::{self, Write as _};
use std::io::{self, BufWriter, Write};
use std::rc::Rc;

use crate::ast::{Arg, BinOp, Block, Expr, FnDecl, InterpPart, Pattern, Program, Stmt, UnOp, place_path};
use crate::check::Resolved;
use crate::diag::Pos;

#[derive(Clone, Debug)]
enum Value {
    Unit,
    Int(i64),
    Bool(bool),
    Str(Rc<str>),
    /// A struct (by index in the program) and its fields.
    Struct(usize, Rc<Vec<Value>>),
    /// An enum and variant, by index.
    Enum(usize, usize),
    List(Rc<Vec<Value>>),
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

impl fmt::Display for Value {
    /// Text form (type.str.3), as used by `print` and interpolation.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Unit => Ok(()),
            Value::Int(v) => write!(f, "{v}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Str(s) => f.write_str(s),
            other => unreachable!("checked: no text form for {other:?}"),
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

/// Ways to leave a block other than reaching its end.
enum Exit {
    Break,
    Continue,
    Return(Value),
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

pub fn run(prog: &Program, resolved: &Resolved) -> Result<(), Trap> {
    let mut fns = HashMap::new();
    let mut methods: HashMap<&str, HashMap<&str, &FnDecl>> = HashMap::new();
    for f in &prog.fns {
        match &f.owner {
            Some((owner, _)) => {
                methods.entry(owner.as_str()).or_default().insert(f.name.as_str(), f);
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
    it.call(main, None, Vec::new(), main.pos)?;
    it.flush(main.pos)
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

/// own.excl.3: overlaps that depend on run-time indexes trap before the call.
fn check_overlap(places: &[(Option<Place>, bool)], pos: Pos) -> Result<(), Trap> {
    for (j, (b, b_mut)) in places.iter().enumerate() {
        for (a, a_mut) in &places[..j] {
            if let (Some(a), Some(b)) = (a, b)
                && (*a_mut || *b_mut)
                && overlaps(a, b)
            {
                return trap("own.excl.3", pos, "two arguments of this call overlap, and one is passed `mut`".into());
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
        let result = match self.block(&f.body, &mut env) {
            Ok(v) | Err(Exit::Return(v)) => v,
            Err(Exit::Trap(t)) => return Err(t),
            Err(Exit::Break | Exit::Continue) => unreachable!("checked: `break` and `continue` are inside loops"),
        };
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
            Stmt::Let { name, value, .. } => {
                let v = self.eval(value, env)?;
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
            Stmt::For { var, iter, body, .. } => self.for_loop(var, iter, body, env)?,
            Stmt::Break(_) => return Err(Exit::Break),
            Stmt::Continue(_) => return Err(Exit::Continue),
            Stmt::Return(value, _) => {
                let v = match value {
                    Some(e) => self.eval(e, env)?,
                    None => Value::Unit,
                };
                return Err(Exit::Return(v));
            }
            Stmt::Expr(e) => return self.eval(e, env),
        }
        Ok(Value::Unit)
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
            Expr::Interp(parts, _) => {
                let mut s = String::new();
                for part in parts {
                    match part {
                        InterpPart::Lit(text) => s.push_str(text),
                        InterpPart::Expr(x) => {
                            let v = self.eval(x, env)?;
                            let _ = write!(s, "{v}");
                        }
                    }
                }
                Value::Str(Rc::from(s))
            }
            Expr::Name(n, pos) => match lookup(env, n) {
                Some(v) => v.clone(),
                None => {
                    let (e, v) = self.resolved.variants[pos];
                    Value::Enum(e, v)
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
            Expr::Field { recv, name, .. } => {
                if let Expr::Name(tn, _) = &**recv
                    && lookup(env, tn).is_none()
                {
                    let e = self.enums[tn.as_str()];
                    let v = self.prog.enums[e].variants.iter().position(|(v, _)| v == name).expect("checked: variant");
                    return Ok(Value::Enum(e, v));
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
            Expr::If { branches, els, .. } => {
                for (cond, body) in branches {
                    if self.eval(cond, env)?.bool() {
                        return self.block(body, env);
                    }
                }
                match els {
                    Some(body) => return self.block(body, env),
                    None => Value::Unit,
                }
            }
            Expr::Match { scrutinee, arms, .. } => {
                let Value::Enum(e, v) = self.eval(scrutinee, env)? else { unreachable!("checked: enum") };
                let variant = &self.prog.enums[e].variants[v].0;
                for arm in arms {
                    let hit = match &arm.pat {
                        Pattern::Wildcard(_) => true,
                        Pattern::Variant { name, .. } => name == variant,
                    };
                    if hit {
                        return self.block(&arm.body, env);
                    }
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
            (Value::Str(a), Value::Str(b)) => Value::Bool((a == b) == (op == BinOp::Eq)),
            (Value::Bool(a), Value::Bool(b)) => Value::Bool((a == b) == (op == BinOp::Eq)),
            (l, r) => unreachable!("checked: `{l:?} {sym} {r:?}`"),
        })
    }

    fn call_expr(&mut self, callee: &'a Expr, args: &'a [Arg], pos: Pos, env: &mut Env<'a>) -> Flow<Value> {
        let Expr::Name(n, _) = callee else { unreachable!("checked: named callee") };
        if n == "print" {
            let v = self.eval(&args[0].value, env)?;
            self.print(&v, pos)?;
            return Ok(Value::Unit);
        }
        if let Some(&s) = self.structs.get(n.as_str()) {
            let mut fields = vec![Value::Unit; self.prog.structs[s].fields.len()];
            for a in args {
                let (label, _) = a.label.as_ref().expect("checked: labelled");
                let i = self.field_index(s, label);
                fields[i] = self.eval(&a.value, env)?;
            }
            return Ok(Value::Struct(s, Rc::new(fields)));
        }
        let f = self.fns[n.as_str()];
        self.invoke(f, None, args, callee.pos(), env)
    }

    fn method_call(
        &mut self,
        recv: &'a Expr,
        name: &'a str,
        args: &'a [Arg],
        pos: Pos,
        env: &mut Env<'a>,
    ) -> Flow<Value> {
        // `Type.name(args)`: an associated function (trait.method.4).
        if let Expr::Name(tn, _) = recv
            && lookup(env, tn).is_none()
        {
            let f = self.methods[tn.as_str()][name];
            return self.invoke(f, None, args, pos, env);
        }
        let recv = self.operand(recv, env)?;
        let value = match &recv {
            Operand::Place(p) => peek(env, p)?,
            Operand::Value(v) => v,
        };
        let type_name = match value {
            Value::List(items) => {
                let len = items.len();
                return self.list_method(len, recv, name, args, pos, env);
            }
            Value::Struct(s, _) => self.prog.structs[*s].name.as_str(),
            Value::Enum(e, _) => self.prog.enums[*e].name.as_str(),
            other => unreachable!("checked: no methods on {other:?}"),
        };
        let f = self.methods[type_name][name];
        self.invoke(f, Some(recv), args, pos, env)
    }

    /// `xs.len()` and `xs.push(v)` (type.coll.5-6).
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
        let Operand::Place(list) = recv else { unreachable!("checked: `push` needs a mutable place") };
        let (v, arg_place) = match self.operand(&args[0].value, env)? {
            Operand::Place(p) => (load(env, &p)?, Some(p)),
            Operand::Value(v) => (v, None),
        };
        check_overlap(&[(Some(list.clone()), true), (arg_place, false)], pos)?;
        let Value::List(items) = slot_mut(env, &list)? else { unreachable!("checked: list") };
        Rc::make_mut(items).push(v);
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
        if self.closed {
            return Ok(());
        }
        match writeln!(self.out, "{v}") {
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
