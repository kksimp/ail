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

mod builtins;

use builtins::{Key, Table, map_value};

use crate::ast::{
    Arg, BinOp, Block, Cond, Else, Expr, FnDecl, InterpPart, Literal, Pattern, Program, Stmt, TypeKind, UnOp,
    place_path,
};
use crate::check::{Call, Con, Resolved};
use crate::diag::Pos;

#[derive(Clone, Debug)]
enum Value {
    Unit,
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(Rc<str>),
    Char(char),
    /// A struct (by index in the program) and its fields.
    Struct(usize, Rc<Vec<Value>>),
    /// An enum and variant, by index, and the variant's fields.
    Enum(usize, usize, Rc<Vec<Value>>),
    List(Rc<Vec<Value>>),
    /// `Map[K, V]` and `Set[T]`, in insertion order (type.map.1).
    Map(Rc<Table>),
    Set(Rc<Table>),
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

    fn float(&self) -> f64 {
        match self {
            Value::Float(v) => *v,
            other => unreachable!("checked: float, found {other:?}"),
        }
    }

    fn char(&self) -> char {
        match self {
            Value::Char(c) => *c,
            other => unreachable!("checked: char, found {other:?}"),
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
    /// `m[k]` (type.map.3)
    Key(Value, Pos),
}

/// Type and field names, for showing a map key in a trap message without
/// running user code (type.map.3).
struct TypeNames {
    /// Each struct's name and field names.
    structs: Vec<Named>,
    /// Each enum's name, and each variant's name and field names.
    enums: Vec<(String, Vec<Named>)>,
}

/// A name and its field names.
type Named = (String, Vec<String>);

thread_local! {
    static NAMES: std::cell::RefCell<TypeNames> =
        const { std::cell::RefCell::new(TypeNames { structs: Vec::new(), enums: Vec::new() }) };
}

pub fn run(prog: &Program, resolved: &Resolved) -> Result<(), Stop> {
    NAMES.with(|n| {
        let fields = |fs: &[crate::ast::Field]| fs.iter().map(|f| f.name.clone()).collect::<Vec<_>>();
        *n.borrow_mut() = TypeNames {
            structs: prog.structs.iter().map(|s| (s.name.clone(), fields(&s.fields))).collect(),
            enums: prog
                .enums
                .iter()
                .map(|e| (e.name.clone(), e.variants.iter().map(|v| (v.name.clone(), fields(&v.fields))).collect()))
                .collect(),
        };
    });
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
        consts: HashMap::new(),
    };
    // mod.const.1: constants are computed before the program runs.
    for c in &prog.consts {
        it.constant(&c.name, c.pos).map_err(|e| match e {
            Exit::Trap(t) => Stop::Trap(t),
            _ => unreachable!("checked: constants don't leave"),
        })?;
    }
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
    /// The values of the constants computed so far (mod.const.1).
    consts: HashMap<&'a str, Value>,
}

fn lookup<'e>(env: &'e Env<'_>, name: &str) -> Option<&'e Value> {
    env.iter().rev().find(|(n, _)| *n == name).map(|(_, v)| v)
}

/// The text form of a float (type.num.7): the shortest decimal that reads
/// back as the same value, in fixed notation when the decimal exponent is
/// from -4 to 15 and in scientific notation otherwise, as Python's `repr`.
fn float_text(x: f64) -> String {
    if x.is_nan() {
        return "nan".into();
    }
    if x.is_infinite() {
        return if x > 0.0 { "inf".into() } else { "-inf".into() };
    }
    let sign = if x.is_sign_negative() { "-" } else { "" };
    // `{:e}` gives the shortest round-trip digits: "1.2345e3".
    let sci = format!("{:e}", x.abs());
    let (mantissa, exp) = sci.split_once('e').expect("scientific form");
    let digits: String = mantissa.chars().filter(|c| *c != '.').collect();
    let exp: i32 = exp.parse().expect("an exponent");
    let point = exp + 1; // digits before the decimal point
    let n = digits.len() as i32;
    let body = if (-3..=16).contains(&point) {
        if point <= 0 {
            format!("0.{}{digits}", "0".repeat((-point) as usize))
        } else if point >= n {
            format!("{digits}{}.0", "0".repeat((point - n) as usize))
        } else {
            format!("{}.{}", &digits[..point as usize], &digits[point as usize..])
        }
    } else {
        let rest = if n > 1 { format!(".{}", &digits[1..]) } else { String::new() };
        let e = point - 1;
        format!("{}{rest}e{}{:02}", &digits[..1], if e < 0 { '-' } else { '+' }, e.abs())
    };
    format!("{sign}{body}")
}

/// The type or trait an expression names, when it isn't a value: `Shape`
/// in `Shape.Circle(r: 1)`, or `Stack` in `Stack[int].new()`.
fn type_head<'e>(e: &'e Expr, env: &Env<'_>, prog: &Program) -> Option<&'e str> {
    match e {
        Expr::Name(n, _) if lookup(env, n).is_none() && !prog.consts.iter().any(|c| c.name == *n) => Some(n),
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
/// With `insert`, a missing map key at the end of the path is added, for
/// `m[k] = v` (type.map.3).
fn slot_mut<'e>(env: &'e mut Env<'_>, p: &Place, insert: bool) -> Result<&'e mut Value, Trap> {
    let mut v = &mut env[p.slot].1;
    let last = p.path.len();
    for (n, step) in p.path.iter().enumerate() {
        v = match (step, v) {
            (Step::Key(k, pos), Value::Map(t)) => {
                let t = Rc::make_mut(t);
                match t.get(k) {
                    Some(i) => t.val_mut(i),
                    None if insert && n + 1 == last => {
                        let i = t.insert(k.clone(), Some(Value::Unit));
                        t.val_mut(i)
                    }
                    None => return trap("type.map.3", *pos, format!("the map has no key {}", key_text(k))),
                }
            }
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
            (Step::Key(k, pos), Value::Map(t)) => map_value(t, k, || key_text(k), *pos)?,
            (_, other) => unreachable!("checked: place step {step:?} on {other:?}"),
        };
    }
    Ok(v)
}

/// A string or character as it appears inside another value (trait.auto.2):
/// in its quotes, with `\\`, the quote and control characters escaped.
fn quoted(text: &str, quote: char) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push(quote);
    for c in text.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            '\0' => out.push_str("\\0"),
            c if c == quote => {
                out.push('\\');
                out.push(c);
            }
            c if c.is_control() => out.push_str(&format!("\\u{{{:x}}}", c as u32)),
            c => out.push(c),
        }
    }
    out.push(quote);
    out
}

/// A key for a trap message: strings and characters quoted, as inside a value.
fn key_text(k: &Value) -> String {
    let fields = |names: &[String], values: &[Value]| {
        if values.is_empty() {
            return String::new();
        }
        let parts: Vec<String> = names.iter().zip(values).map(|(n, v)| format!("{n}: {}", key_text(v))).collect();
        format!("({})", parts.join(", "))
    };
    match k {
        Value::Str(s) => quoted(s, '"'),
        Value::Char(c) => quoted(&c.to_string(), '\''),
        Value::Int(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::None => "none".into(),
        Value::List(items) => format!("[{}]", items.iter().map(key_text).collect::<Vec<_>>().join(", ")),
        Value::Struct(s, values) => NAMES.with(|n| {
            let (name, names) = &n.borrow().structs[*s];
            format!("{name}{}", fields(names, values))
        }),
        Value::Enum(e, i, values) => NAMES.with(|n| {
            let n = n.borrow();
            let (enum_name, variants) = &n.enums[*e];
            let (name, names) = &variants[*i];
            format!("{enum_name}.{name}{}", fields(names, values))
        }),
        _ => "(a value)".into(),
    }
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
            (Step::Key(i, _), Step::Key(j, _)) => Key::of(i) == Key::of(j),
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
            // err.else.4: the `else` runs only on a failure.
            Stmt::Handle { value, otherwise } => {
                if let v @ Value::Fail(_) = self.eval(value, env)? {
                    self.otherwise(otherwise, v, env)?;
                }
            }
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
            let v = std::mem::replace(slot_mut(env, &at, false)?, Value::Unit);
            env.push((var, v));
            let r = self.block(body, env);
            let (_, v) = env.pop().expect("the loop variable");
            *slot_mut(env, &at, false)? = v;
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
        if let Expr::Range { lo: Some(lo), hi: Some(hi), inclusive, .. } = iter {
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
        let items = match self.eval(iter, env)? {
            Value::List(items) => items,
            Value::Set(t) => Rc::new(t.key_list()),
            other => unreachable!("checked: iterable, found {other:?}"),
        };
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
        *slot_mut(env, &p, op.is_none())? = v;
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
                let is_map = matches!(peek(env, &p)?, Value::Map(_));
                let i = self.eval(index, env)?;
                p.path.push(if is_map { Step::Key(i, *pos) } else { Step::Index(i.int(), *pos) });
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
            // type.num.10
            Expr::Int(v, pos) if self.resolved.float_literals.contains(pos) => Value::Float(*v as f64),
            Expr::Int(v, _) => Value::Int(*v),
            Expr::Float(v, _) => Value::Float(*v),
            Expr::Char(c, _) => Value::Char(*c),
            Expr::Bool(b, _) => Value::Bool(*b),
            Expr::None(_) => Value::None,
            // err.else.6: the fallback runs only when the value is absent or failed.
            Expr::Else { value, otherwise, .. } => match self.eval(value, env)? {
                v @ (Value::None | Value::Fail(_)) => self.otherwise(otherwise, v, env)?,
                v => v,
            },
            Expr::Try { expr, context, pos } => {
                let v = self.eval(expr, env)?;
                match v {
                    Value::None if self.resolved.optional_try.contains(pos) => {
                        return Err(Exit::Return(Value::None, *pos));
                    }
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
                        InterpPart::Fmt(x, spec) => {
                            let v = self.eval(x, env)?;
                            s.push_str(&self.formatted(&v, spec, x.pos())?);
                        }
                    }
                }
                Value::Str(Rc::from(s))
            }
            Expr::Name(n, pos) => match lookup(env, n) {
                Some(v) => v.clone(),
                None if self.prog.consts.iter().any(|c| c.name == *n) => self.constant(n, *pos)?,
                None => {
                    let Some(&(e, v)) = self.resolved.variants.get(pos) else {
                        unreachable!("checked: `{n}` at {}:{} is a variant", pos.line, pos.col)
                    };
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
                if let Some(tn) = type_head(recv, env, self.prog) {
                    let e = self.enums[tn];
                    let v = self.variant_index(e, name);
                    return Ok(Value::Enum(e, v, Rc::new(Vec::new())));
                }
                let Value::Struct(s, fields) = self.eval(recv, env)? else { unreachable!("checked: struct") };
                fields[self.field_index(s, name)].clone()
            }
            Expr::Index { recv, index, pos } => {
                let items = match self.eval(recv, env)? {
                    Value::List(items) => items,
                    Value::Map(t) => {
                        let k = self.eval(index, env)?;
                        return Ok(map_value(&t, &k, || key_text(&k), *pos)?.clone());
                    }
                    other => unreachable!("checked: list or map, found {other:?}"),
                };
                if let Expr::Range { lo, hi, inclusive, pos: range_pos } = &**index {
                    // type.coll.10: a missing bound is the start or the end.
                    let lo = match lo {
                        Some(e) => self.eval(e, env)?.int(),
                        None => 0,
                    };
                    let (hi, inclusive) = match hi {
                        Some(e) => (self.eval(e, env)?.int(), *inclusive),
                        None => (items.len() as i64, false),
                    };
                    return Ok(Value::List(Rc::new(self.slice(&items, lo, hi, inclusive, *range_pos)?)));
                }
                let i = self.eval(index, env)?.int();
                element(&items, i, *pos)?.clone()
            }
            Expr::Unary { op: UnOp::Neg, operand, pos } => {
                let v = match self.eval(operand, env)? {
                    Value::Float(x) => return Ok(Value::Float(-x)),
                    v => v.int(),
                };
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
            Expr::Map(pairs, pos) => {
                let mut t = Table::default();
                for (k, v) in pairs {
                    let k = self.eval(k, env)?;
                    let v = self.eval(v, env)?;
                    t.insert(k, Some(v));
                }
                if pairs.is_empty() && self.resolved.empty_sets.contains(pos) {
                    Value::Set(Rc::new(t))
                } else {
                    Value::Map(Rc::new(t))
                }
            }
            Expr::Set(items, _) => {
                let mut t = Table::default();
                for x in items {
                    let x = self.eval(x, env)?;
                    t.insert(x, None);
                }
                Value::Set(Rc::new(t))
            }
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
                let v = self.eval(scrutinee, env)?;
                for arm in arms {
                    if !self.matches(&arm.pat, &v) {
                        continue;
                    }
                    let mark = env.len();
                    // expr.match.6: bind fields by position; `_` skips one.
                    if let (Pattern::Variant { bindings: Some(names), .. }, Value::Enum(_, _, fields)) = (&arm.pat, &v)
                    {
                        for ((n, _), value) in names.iter().zip(fields.iter()) {
                            if n != "_" {
                                env.push((n.as_str(), value.clone()));
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

    /// The value of the constant `name`, computing it on first use from the
    /// constants it refers to (mod.const.1).
    fn constant(&mut self, name: &'a str, pos: Pos) -> Flow<Value> {
        if let Some(v) = self.consts.get(name) {
            return Ok(v.clone());
        }
        let prog = self.prog;
        let c = prog.consts.iter().find(|c| c.name == name).expect("checked: a constant");
        let mut env = Vec::new();
        let v = self.eval(&c.value, &mut env)?;
        let _ = pos;
        self.consts.insert(&c.name, v.clone());
        Ok(v)
    }

    /// Whether `v` matches `pat` (expr.match.4, expr.match.7-8).
    fn matches(&self, pat: &Pattern, v: &Value) -> bool {
        match (pat, v) {
            (Pattern::Wildcard(_), _) => true,
            (Pattern::Or(alts, _), v) => alts.iter().any(|a| self.matches(a, v)),
            (Pattern::Variant { name, .. }, Value::Enum(e, i, _)) => self.prog.enums[*e].variants[*i].name == *name,
            (Pattern::Literal(lit, _), v) => match (lit, v) {
                (Literal::Int(a), Value::Int(b)) => a == b,
                (Literal::Char(a), Value::Char(b)) => a == b,
                (Literal::Str(a), Value::Str(b)) => a.as_str() == &**b,
                (Literal::Bool(a), Value::Bool(b)) => a == b,
                _ => false,
            },
            _ => false,
        }
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
            // type.num.6: IEEE 754 arithmetic and comparisons, with no traps.
            (Value::Float(a), Value::Float(b)) => match op {
                BinOp::Add => Value::Float(a + b),
                BinOp::Sub => Value::Float(a - b),
                BinOp::Mul => Value::Float(a * b),
                BinOp::Div => Value::Float(a / b),
                BinOp::Eq => Value::Bool(a == b),
                BinOp::Ne => Value::Bool(a != b),
                BinOp::Lt => Value::Bool(a < b),
                BinOp::Le => Value::Bool(a <= b),
                BinOp::Gt => Value::Bool(a > b),
                BinOp::Ge => Value::Bool(a >= b),
                _ => unreachable!("checked: `{sym}` on floats"),
            },
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
            Value::Float(_) => Some(Con::Float),
            Value::Str(_) => Some(Con::Str),
            Value::Char(_) => Some(Con::Char),
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
            (Value::Float(x), Value::Float(y)) => x == y,
            (Value::Char(x), Value::Char(y)) => x == y,
            (Value::Bool(x), Value::Bool(y)) => x == y,
            (Value::Str(x), Value::Str(y)) => x == y,
            (Value::None, Value::None) => true,
            (Value::None, _) | (_, Value::None) => false,
            (Value::List(x), Value::List(y)) | (Value::Struct(_, x), Value::Struct(_, y)) => {
                self.all_equal(x, y, pos)?
            }
            (Value::Enum(e, v, x), Value::Enum(f, w, y)) => e == f && v == w && self.all_equal(x, y, pos)?,
            // type.map.4: the same keys with equal values, in any order.
            (Value::Map(x), Value::Map(y)) => {
                if x.len() != y.len() {
                    return Ok(false);
                }
                for (k, v) in x.entries() {
                    match y.get(k) {
                        Some(i) if self.equal(v, y.val(i), pos)? => {}
                        _ => return Ok(false),
                    }
                }
                true
            }
            (Value::Set(x), Value::Set(y)) => x.len() == y.len() && x.keys().all(|k| y.get(k).is_some()),
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
            // type.num.7: a total order for sorting, with NaN after every number.
            (Value::Float(x), Value::Float(y)) => match (x.is_nan(), y.is_nan()) {
                (true, true) => Ordering::Equal,
                (true, false) => Ordering::Greater,
                (false, true) => Ordering::Less,
                _ => x.partial_cmp(y).expect("not NaN"),
            },
            (Value::Char(x), Value::Char(y)) => x.cmp(y),
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
        if n == "unreachable" && lookup(env, n).is_none() {
            let Value::Str(msg) = self.eval(&args[0].value, env)? else { unreachable!("checked: str") };
            return Err(Exit::Trap(Trap { rule: "err.model.2", pos, message: format!("unreachable: {msg}") }));
        }
        if matches!(callee, Expr::TypeArgs { .. }) {
            match n.as_str() {
                "List" => return Ok(Value::List(Rc::new(Vec::new()))),
                "Map" => return Ok(Value::Map(Rc::new(Table::default()))),
                "Set" => return Ok(Value::Set(Rc::new(Table::default()))),
                _ => {}
            }
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

    /// Field values in declaration order from named, shorthand and
    /// positional arguments, evaluated in the order written (type.struct.3,
    /// type.struct.7, type.enum.6).
    fn labelled(&mut self, fields: &'a [crate::ast::Field], args: &'a [Arg], env: &mut Env<'a>) -> Flow<Vec<Value>> {
        let mut values = vec![Value::Unit; fields.len()];
        for (a, (slot, _)) in args.iter().zip(crate::ast::field_slots(fields, args)) {
            values[slot.expect("checked: field exists")] = self.eval(&a.value, env)?;
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

    /// `{x:spec}` (type.str.11): the text form of `v` shaped by `spec`.
    /// Numbers align right by default, everything else left.
    fn formatted(&mut self, v: &Value, spec: &crate::lexer::FormatSpec, pos: Pos) -> Result<String, Trap> {
        let numeric = matches!(v, Value::Int(_) | Value::Float(_));
        let body = match v {
            Value::Float(x) if spec.precision.is_some() && x.is_finite() => {
                format!("{x:.*}", spec.precision.unwrap_or(0))
            }
            Value::Int(n) if spec.hex => {
                if *n < 0 { format!("-{:x}", n.unsigned_abs()) } else { format!("{n:x}") }
            }
            other => self.text(other, pos)?,
        };
        let pad = spec.width.unwrap_or(0).saturating_sub(body.chars().count());
        let fill = spec.fill.unwrap_or(' ');
        let (left, right) = match spec.align.unwrap_or(if numeric { '>' } else { '<' }) {
            '<' => (0, pad),
            '>' => (pad, 0),
            _ => (pad / 2, pad - pad / 2),
        };
        let mut out: String = std::iter::repeat(fill).take(left).collect();
        out.push_str(&body);
        out.extend(std::iter::repeat(fill).take(right));
        Ok(out)
    }

    /// Text form, as used by `print`, interpolation and error reports
    /// (type.str.3, trait.std.3, err.text.1).
    fn text(&mut self, v: &Value, pos: Pos) -> Result<String, Trap> {
        match v {
            Value::Unit => Ok(String::new()),
            Value::Str(s) => Ok(s.to_string()),
            Value::Char(c) => Ok(c.to_string()),
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
            Value::Float(x) => float_text(*x),
            Value::Bool(b) => b.to_string(),
            Value::Str(s) => quoted(s, '"'),
            Value::Char(c) => quoted(&c.to_string(), '\''),
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
            Value::Map(t) => {
                let mut parts = Vec::with_capacity(t.len());
                for (k, v) in t.entries() {
                    parts.push(format!("{}: {}", self.describe(k, pos)?, self.describe(v, pos)?));
                }
                format!("{{{}}}", parts.join(", "))
            }
            Value::Set(t) => {
                let mut parts = Vec::with_capacity(t.len());
                for k in t.keys() {
                    parts.push(self.describe(k, pos)?);
                }
                format!("{{{}}}", parts.join(", "))
            }
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
        if let Some(tn) = type_head(recv, env, self.prog) {
            if tn == "char" {
                let n = self.eval(&args[0].value, env)?.int();
                let c = u32::try_from(n).ok().and_then(char::from_u32);
                return Ok(c.map_or(Value::None, Value::Char));
            }
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
            Value::Struct(s, _) => self.prog.structs[*s].name.as_str(),
            Value::Enum(e, _, _) => self.prog.enums[*e].name.as_str(),
            _ => return self.builtin(recv, name, args, pos, env),
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
                    slot_mut(env, place.as_ref().expect("checked: `mut` operands are places"), false)?,
                    Value::Unit,
                ),
            });
        }
        let recv_value = if has_recv { Some(values.remove(0)) } else { None };
        let (result, finals) = self.call(f, recv_value, values, pos)?;
        for ((place, _, mutable), v) in ops.into_iter().zip(finals) {
            if mutable {
                *slot_mut(env, &place.expect("checked: `mut` operands are places"), false)? = v;
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
