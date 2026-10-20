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

use crate::fasthash::FastMap as HashMap;
use std::fmt::{self, Write as _};
use std::io::{self, BufWriter, Write};
use std::cell::RefCell;
use std::rc::Rc;

use std::cmp::Ordering;

mod absent;
mod builtins;
mod keyfn;
pub mod limits;
mod contracts;
pub mod parallel;
mod net;
mod osext;
mod random;
mod stdlib;
mod tasks;
mod trace;

pub use contracts::set_enabled as set_contracts;
pub use random::seed as set_seed;
pub use stdlib::set_program;
mod prop;
mod testing;
mod ui;
pub use prop::PropInfo;
pub use testing::run_tests;
pub use trace::{render_trap, take_options as take_trace_options, option_words as trace_option_words, take_snapshot};

use builtins::{Key, Table, map_value};
use tasks::Handle;

use crate::ast::{
    Arg, BinOp, Block, CompFor, CompKind, Cond, Else, Expr, FnDecl, InterpPart, Literal, Pattern, Program, Stmt,
    TypeKind, UnOp, place_path, place_root,
};
use crate::check::{Call, Con, Resolved};
use crate::diag::Pos;
use crate::num::Num;

#[derive(Debug)]
enum Value {
    Unit,
    Int(i64),
    Float(f64),
    /// A sized number (type.num.1) as its stored bits (`crate::num`).
    Num(Num, i64),
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
    /// `present` applied `n >= 1` times to `none`: an optional of an
    /// optional whose inner levels are absent (type.opt.7). Any other
    /// present optional is its content, so `T` converts to `T?` for free.
    Nested(u32),
    /// The failure of a `T ! E` result (err.model.1).
    Fail(Rc<Failure>),
    /// A `ref T` cell (own.ref.1): copies share it.
    Ref(Rc<RefCell<Value>>),
    /// A task, task group, channel or mutex (conc.task.1, conc.share.2-3).
    Handle(Handle),
}

impl Clone for Value {
    /// Written out so the scalar cases inline at every read of a local
    /// (the derived `clone` was an out-of-line call; ADR 0219).
    #[inline(always)]
    fn clone(&self) -> Value {
        match self {
            Value::Unit => Value::Unit,
            Value::Int(v) => Value::Int(*v),
            Value::Float(v) => Value::Float(*v),
            Value::Num(k, v) => Value::Num(*k, *v),
            Value::Bool(v) => Value::Bool(*v),
            Value::Char(v) => Value::Char(*v),
            Value::None => Value::None,
            Value::Nested(n) => Value::Nested(*n),
            Value::Str(s) => Value::Str(Rc::clone(s)),
            Value::Struct(i, f) => Value::Struct(*i, Rc::clone(f)),
            Value::Enum(e, v, f) => Value::Enum(*e, *v, Rc::clone(f)),
            Value::List(l) => Value::List(Rc::clone(l)),
            Value::Map(t) => Value::Map(Rc::clone(t)),
            Value::Set(t) => Value::Set(Rc::clone(t)),
            Value::Fail(f) => Value::Fail(Rc::clone(f)),
            Value::Ref(c) => Value::Ref(Rc::clone(c)),
            Value::Handle(h) => Value::Handle(*h),
        }
    }
}

/// An error value with its context messages and return trace (err.ctx.1-2).
#[derive(Debug)]
struct Failure {
    error: Value,
    context: Vec<String>,
    trace: Vec<Pos>,
}

impl Value {
    /// The optional that holds `v` (type.opt.7): `v` itself, unless `v` is
    /// an absent optional, which would read as the outer `none`.
    fn present(v: Value) -> Value {
        match v {
            Value::None => Value::Nested(1),
            Value::Nested(n) => Value::Nested(n + 1),
            v => v,
        }
    }

    /// The content of a present optional (type.opt.7): `present`'s inverse.
    fn unwrap_present(v: Value) -> Value {
        match v {
            Value::Nested(1) => Value::None,
            Value::Nested(n) => Value::Nested(n - 1),
            v => v,
        }
    }

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
    /// `at` writes a position as `FILE:LINE:COL` (a program can have
    /// several files).
    pub fn render_with(&self, at: impl Fn(Pos) -> String) -> String {
        let mut s = format!("error: {}", self.error);
        for c in &self.context {
            let _ = write!(s, "\n  context: {c}");
        }
        for p in &self.trace {
            let _ = write!(s, "\n  at {}", at(*p));
        }
        s
    }
}

/// Ways to leave a block other than reaching its end.
enum Exit {
    Break,
    Continue,
    /// A `return` (or `?`): the value and where it was returned wait in `Interp::returned`,
    /// which keeps `Flow<Value>` at 32 bytes instead of 48 (ADR 0219).
    Return,
    Trap(Box<Trap>),
}

impl From<Trap> for Exit {
    fn from(t: Trap) -> Self {
        Exit::Trap(Box::new(t))
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

/// What is known of a method-call site: its trait dispatch, whether the method may change its
/// receiver, and the last user method it resolved to (type kind, index, function).
#[derive(Clone, Copy)]
struct MethodSite<'a> {
    call: Option<Call>,
    std: Option<crate::stdlib::StdCall>,
    mutating: bool,
    last: Option<(bool, usize, &'a FnDecl)>,
}

/// The most `mut` operands (receiver and parameters) the local-move call path handles.
const MAX_LOCAL_MOVES: usize = 6;

/// What a call expression names.
#[derive(Clone, Copy)]
enum Target<'a> {
    Struct(usize),
    Variant(usize, usize),
    Fn(&'a FnDecl),
}

/// What a method's receiver is, when it isn't a built-in value: a struct or enum (is-enum, index)
/// or a task handle.
#[derive(Clone, Copy)]
enum MethodKind {
    Type(bool, usize),
    Handle(Handle),
}

/// The kind of a method receiver, and the cell when it is a `ref` (own.ref.3).
fn kind_or_cell(v: &Value) -> (Option<MethodKind>, Option<Rc<RefCell<Value>>>) {
    match v {
        Value::Ref(c) => (None, Some(c.clone())),
        v => (method_kind(v), None),
    }
}

fn method_kind(v: &Value) -> Option<MethodKind> {
    match v {
        Value::Struct(s, _) => Some(MethodKind::Type(false, *s)),
        Value::Enum(e, _, _) => Some(MethodKind::Type(true, *e)),
        &Value::Handle(h) => Some(MethodKind::Handle(h)),
        _ => None,
    }
}

/// A place (own.place.1) resolved to a local slot and a path of steps, or,
/// when the place goes through a `ref` (own.ref.3), to `IN_CELL` and a path
/// that starts with `Step::Cell`.
#[derive(Clone, Debug)]
struct Place {
    slot: usize,
    path: Vec<Step>,
}

const IN_CELL: usize = usize::MAX;

impl Place {
    fn local(slot: usize) -> Self {
        Place { slot, path: Vec::new() }
    }

    fn in_cell(cell: Rc<RefCell<Value>>) -> Self {
        Place { slot: IN_CELL, path: vec![Step::Cell(cell)] }
    }

    fn cell(&self) -> Option<&Rc<RefCell<Value>>> {
        match (self.slot, self.path.first()) {
            (IN_CELL, Some(Step::Cell(c))) => Some(c),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
enum Step {
    Field(usize),
    Index(i64, Pos),
    /// `m[k]` (type.map.3)
    Key(Value, Pos),
    /// The root of a place in a `ref` cell; only first in a path (own.ref.3).
    Cell(Rc<RefCell<Value>>),
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

/// An interpreter for `prog` writing standard output to `out`.
fn build<'a, W: Write>(prog: &'a Program, resolved: &'a Resolved, out: W) -> Interp<'a, W> {
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
    let mut fns = HashMap::default();
    let mut methods: HashMap<&str, HashMap<&str, &FnDecl>> = HashMap::default();
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
    let user_std =
        [(resolved.std.eq, "eq"), (resolved.std.ord, "cmp"), (resolved.std.to_str, "to_str")].map(|(tr, name)| {
            resolved.impls.keys().any(|(t, _)| *t == tr)
                || prog.traits[tr].methods.iter().any(|&m| prog.fns[m].name == name && prog.fns[m].body.is_some())
        });
    let mut mutating: crate::fasthash::FastSet<&str> =
        ["push", "swap", "pop", "sort", "insert", "remove_at", "extend", "clear", "reverse", "remove", "add"].into_iter().collect();
    mutating.extend(prog.fns.iter().filter(|f| f.receiver.is_some_and(|r| r.mutable)).map(|f| f.name.as_str()));
    Interp {
        prog,
        resolved,
        fns,
        methods,
        structs,
        enums,
        out: BufWriter::new(out),
        depth: 0,
        closed: false,
        consts: HashMap::default(),
        env_pool: Vec::new(),
        returned: None,
        ops_pool: Vec::new(),
        arg_pool: Vec::new(),
        no_fields: Rc::new(Vec::new()),
        method_sites: HashMap::default(),
        call_targets: HashMap::default(),
        user_std,
        mutating,
        field_slots: HashMap::default(),
        str_lits: HashMap::default(),
        lit_mask: resolved.float_literals.iter().chain(resolved.sized_literals.keys()).fold(0, |m, p| m | 1 << (p.line & 63)),
        has_unique: prog.structs.iter().any(|s| s.unique) || prog.enums.iter().any(|e| e.unique),
        has_defer: prog.fns.iter().filter_map(|f| f.body.as_ref()).any(|b| has_defer(b)),
        has_moves: !resolved.moves.is_empty(),
        showing: Vec::new(),
        conc: None,
        tracing: trace::mode(),
        snap: None,
        par: None,
        pure: None,
        hook: limits::active(),
        steps: 0,
        cov_last: 0,
    }
}

/// Keep the first way out: a trap or exit already on its way wins over one
/// from a cleanup (own.uniq.3, own.defer.1).
fn keep_first(result: &mut Flow<Value>, r: Flow<()>) {
    if let Err(e) = r
        && result.is_ok()
    {
        *result = Err(e);
    }
}

/// Whether a block, or a block inside it, has a `defer` (own.defer.1).
fn has_defer(b: &Block) -> bool {
    fn in_expr(e: &Expr) -> bool {
        match e {
            Expr::If { branches, els, .. } => {
                branches.iter().any(|(_, b)| has_defer(b)) || els.as_ref().is_some_and(|b| has_defer(b))
            }
            Expr::Match { arms, .. } => arms.iter().any(|a| has_defer(&a.body)),
            _ => false,
        }
    }
    b.iter().any(|s| match s {
        Stmt::Defer(..) => true,
        Stmt::While { body, .. } | Stmt::For { body, .. } => has_defer(body),
        Stmt::Let { value, otherwise, .. } => {
            in_expr(value) || matches!(otherwise, Some(Else::Block { body, .. }) if has_defer(body))
        }
        Stmt::Expr(e) | Stmt::Assign { value: e, .. } => in_expr(e),
        _ => false,
    })
}

pub fn run(prog: &Program, resolved: &Resolved) -> Result<(), Stop> {
    let stdout = io::stdout();
    let mut it = build(prog, resolved, stdout.lock());
    let _watch = limits::watch();
    // trace.snap.1: a trap that ends the program carries a snapshot.
    it.snap = Some(Box::default());
    it.par = parallel::plans(prog, resolved);
    let result = run_main(&mut it, prog);
    if let (Err(Stop::Trap(t)), Some(snap)) = (&result, it.snap.take()) {
        trace::keep(prog, *snap, t);
    }
    if it.tracing != 0 {
        trace::finish();
    }
    if it.conc.is_some() {
        // What tasks printed before a trap still reaches standard output.
        it.task_print("");
        tasks::shutdown(it.conc.take());
    }
    result
}

fn run_main<'a, W: Write>(it: &mut Interp<'a, W>, prog: &'a Program) -> Result<(), Stop> {
    // mod.const.1: constants are computed before the program runs.
    for c in &prog.consts {
        it.constant(&c.name, c.pos).map_err(|e| match e {
            Exit::Trap(t) => Stop::Trap(*t),
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
    /// Spare call environments, so a call doesn't allocate one (ADR 0219).
    env_pool: Vec<Env<'a>>,
    /// The value of the `return` being carried out, and where it was returned.
    returned: Option<(Value, Pos)>,
    ops_pool: Vec<Vec<Prepared>>,
    arg_pool: Vec<Vec<Value>>,
    /// The shared field list of every variant without fields.
    no_fields: Rc<Vec<Value>>,
    /// What each method-call site resolves to, by the address of its receiver expression.
    method_sites: HashMap<usize, MethodSite<'a>>,
    /// What each call site names, by the address of its callee expression.
    call_targets: HashMap<usize, Target<'a>>,
    /// Whether the program implements, or gives a default body to, `eq`, `cmp` and `to_str`.
    /// When not, `==`, ordering and text go straight to the built-in forms (trait.std.1).
    user_std: [bool; 3],
    /// Names of methods that may change their receiver: the built-in mutators and every
    /// user method with a `mut` receiver. A call to any other method needs only the receiver's value.
    mutating: crate::fasthash::FastSet<&'a str>,
    /// The field each argument of a struct or variant construction sets, by the address of its arguments.
    field_slots: HashMap<usize, Rc<[usize]>>,
    /// String literals as shared text, by the address of the literal in the program.
    str_lits: HashMap<usize, Rc<str>>,
    /// Bit `line % 64` is set for each line holding a sized or float-typed literal, so most literals skip the tables.
    lit_mask: u64,
    /// The program declares a unique type, so scopes release values
    /// (own.uniq.3); uses `defer` (own.defer.1); moves values (own.uniq.2).
    /// Each is off for most programs, which then pay nothing for it.
    has_unique: bool,
    has_defer: bool,
    has_moves: bool,
    /// The `ref` cells whose text form is being made, to show cycles.
    showing: Vec<*const RefCell<Value>>,
    /// The task scheduler, once the program uses tasks (conc.task.1).
    conc: Option<tasks::Ctx>,
    /// What `--trace` and `--watch` report (trace.run.1, trace.watch.1); 0 when off.
    tracing: u8,
    /// The frames of a trap on its way out, when `run` keeps a snapshot (trace.snap.1).
    snap: Option<Box<trace::Snap>>,
    /// The loops that run on the thread pool (conc.par.8).
    par: Option<Rc<parallel::Plans>>,
    /// Which functions a task may run on the pool (conc.par.10), once needed.
    pure: Option<Rc<Vec<bool>>>,
    /// A run limit or coverage is on (run.limit.1-3, test.cover.1); off costs one branch per statement.
    hook: bool,
    steps: u64,
    cov_last: usize,
}

/// Name equality without a `memcmp` call: names are short, and `lookup` is
/// the hottest function in the interpreter (ADR 0219).
#[inline(always)]
fn name_eq(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x == y)
}

#[inline(always)]
fn lookup<'e>(env: &'e Env<'_>, name: &str) -> Option<&'e Value> {
    env.iter().rev().find(|(n, _)| name_eq(n, name)).map(|(_, v)| v)
}

/// The text form of a float (type.num.7): the shortest decimal that reads
/// back as the same value, in fixed notation when the decimal exponent is
/// from -4 to 15 and in scientific notation otherwise, as Python's `repr`.
fn float_text(x: f64) -> String {
    float_shape(x.is_nan(), x.is_infinite(), x.is_sign_negative(), format!("{:e}", x.abs()))
}

/// An `f32`'s text form: the same shape, from the shortest digits that read back as an `f32`.
fn float_text32(x: f32) -> String {
    float_shape(x.is_nan(), x.is_infinite(), x.is_sign_negative(), format!("{:e}", x.abs()))
}

/// `sci` is the shortest round-trip scientific form of the magnitude, e.g. "1.2345e3" (type.num.7).
fn float_shape(nan: bool, inf: bool, negative: bool, sci: String) -> String {
    if nan {
        return "nan".into();
    }
    if inf {
        return if negative { "-inf".into() } else { "inf".into() };
    }
    let sign = if negative { "-" } else { "" };
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

/// type.num.11: numbers in numeric order, with NaN after every number.
fn float_order(x: f64, y: f64) -> Ordering {
    match (x.is_nan(), y.is_nan()) {
        (true, true) => Ordering::Equal,
        (true, false) => Ordering::Greater,
        (false, true) => Ordering::Less,
        _ => x.partial_cmp(&y).expect("not NaN"),
    }
}

/// A literal typed by context as `k` (type.num.13): `float` is its value as a
/// float, `whole` its value as an integer. The checker has range-checked it.
fn sized_literal(k: Num, float: f64, whole: i64) -> Value {
    if k.is_float() { Value::Num(k, Num::f32_bits(float as f32)) } else { Value::Num(k, whole) }
}

/// Unary `-` on a sized number (type.num.14). A negated literal was
/// range-checked by the checker as the negative number.
fn negate(k: Num, bits: i64, pos: Pos) -> Flow<Value> {
    if k.is_float() {
        return Ok(Value::Num(k, Num::f32_bits(-Num::f32_of(bits))));
    }
    let n = -k.decode(bits);
    if k.fits(n) {
        return Ok(Value::Num(k, k.encode(n)));
    }
    let message = format!("`-({})` overflows `{}`", -n, k.name());
    Err(Trap { rule: "type.num.14", pos, message }.into())
}

/// Arithmetic and comparison on two sized numbers of one type (type.num.14):
/// integers trap on overflow and division by zero as `int` does.
fn num_binop(op: BinOp, k: Num, a: i64, b: i64, pos: Pos) -> Result<Value, Trap> {
    let sym = op.symbol();
    if k.is_float() {
        let (x, y) = (Num::f32_of(a), Num::f32_of(b));
        let f = |v: f32| Value::Num(k, Num::f32_bits(v));
        return Ok(match op {
            BinOp::Add => f(x + y),
            BinOp::Sub => f(x - y),
            BinOp::Mul => f(x * y),
            BinOp::Div => f(x / y),
            BinOp::Eq => Value::Bool(x == y),
            BinOp::Ne => Value::Bool(x != y),
            BinOp::Lt => Value::Bool(x < y),
            BinOp::Le => Value::Bool(x <= y),
            BinOp::Gt => Value::Bool(x > y),
            BinOp::Ge => Value::Bool(x >= y),
            _ => unreachable!("checked: `{sym}` on f32"),
        });
    }
    let (x, y) = (k.decode(a), k.decode(b));
    let result = match op {
        BinOp::Add => x + y,
        BinOp::Sub => x - y,
        BinOp::Mul => x.checked_mul(y).unwrap_or(i128::MAX), // only u64 x u64 can exceed 128 bits
        BinOp::Div | BinOp::Rem => {
            if y == 0 {
                return trap("expr.arith.3", pos, format!("division by zero in `{x} {sym} 0`"));
            }
            if op == BinOp::Div { x / y } else { x % y }
        }
        BinOp::Eq => return Ok(Value::Bool(x == y)),
        BinOp::Ne => return Ok(Value::Bool(x != y)),
        BinOp::Lt => return Ok(Value::Bool(x < y)),
        BinOp::Le => return Ok(Value::Bool(x <= y)),
        BinOp::Gt => return Ok(Value::Bool(x > y)),
        BinOp::Ge => return Ok(Value::Bool(x >= y)),
        _ => unreachable!("checked: `{sym}` on {}", k.name()),
    };
    if !k.fits(result) {
        let rule = if op == BinOp::Div { "expr.arith.3" } else { "expr.arith.1" };
        return trap(rule, pos, format!("`{x} {sym} {y}` overflows `{}`", k.name()));
    }
    Ok(Value::Num(k, k.encode(result)))
}

/// The int arithmetic that cannot trap: everything else (overflow, a zero
/// divisor) is `binop`'s (expr.arith.1, expr.arith.3).
#[inline(always)]
fn int_arith(op: BinOp, a: i64, b: i64) -> Option<i64> {
    match op {
        BinOp::Add => a.checked_add(b),
        BinOp::Sub => a.checked_sub(b),
        BinOp::Mul => a.checked_mul(b),
        BinOp::Div if b != 0 => a.checked_div(b),
        BinOp::Rem if b != 0 => Some(a.wrapping_rem(b)), // exact: `a % -1` is 0
        _ => None,
    }
}

#[inline(always)]
fn int_cmp(op: BinOp, a: i64, b: i64) -> Option<bool> {
    match op {
        BinOp::Eq => Some(a == b),
        BinOp::Ne => Some(a != b),
        BinOp::Lt => Some(a < b),
        BinOp::Le => Some(a <= b),
        BinOp::Gt => Some(a > b),
        BinOp::Ge => Some(a >= b),
        _ => None,
    }
}

#[inline(always)]
fn int_op(op: BinOp, a: i64, b: i64) -> Option<Value> {
    match int_arith(op, a, b) {
        Some(v) => Some(Value::Int(v)),
        None => int_cmp(op, a, b).map(Value::Bool),
    }
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
#[inline]
fn slot_mut<'e>(env: &'e mut Env<'_>, p: &Place, insert: bool) -> Result<&'e mut Value, Trap> {
    debug_assert!(p.slot != IN_CELL, "a place in a `ref` cell goes through `with_slot`");
    slot_mut_at(env, p.slot, &p.path, insert)
}

/// `slot_mut` for a slot and a path of steps that need not be in a `Place`.
fn slot_mut_at<'e>(env: &'e mut Env<'_>, slot: usize, path: &[Step], insert: bool) -> Result<&'e mut Value, Trap> {
    slot_in(&mut env[slot].1, path, insert)
}

/// The storage at `path` inside `root`, as for `slot_mut`.
fn slot_in<'e>(root: &'e mut Value, path: &[Step], insert: bool) -> Result<&'e mut Value, Trap> {
    let mut v = root;
    let last = path.len();
    for (n, step) in path.iter().enumerate() {
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
#[inline]
fn peek<'e>(env: &'e Env<'_>, p: &Place) -> Result<&'e Value, Trap> {
    debug_assert!(p.slot != IN_CELL, "a place in a `ref` cell goes through `view`");
    peek_at(env, p.slot, &p.path)
}

fn peek_at<'e>(env: &'e Env<'_>, slot: usize, path: &[Step]) -> Result<&'e Value, Trap> {
    peek_in(&env[slot].1, path)
}

fn peek_in<'e>(root: &'e Value, path: &[Step]) -> Result<&'e Value, Trap> {
    let mut v = root;
    for step in path {
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
        Value::Num(k, bits) if k.is_float() => float_text32(Num::f32_of(*bits)),
        Value::Num(k, bits) => k.decode(*bits).to_string(),
        Value::Bool(b) => b.to_string(),
        Value::None => "none".into(),
        Value::Nested(n) => format!("{}none{}", "some(".repeat(*n as usize), ")".repeat(*n as usize)),
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

/// The zero a missing `m[k] op= v` starts from, of the type of `v` (type.map.7).
fn zero_like(rhs: &Value) -> Value {
    match rhs {
        Value::Float(_) => Value::Float(0.0),
        Value::Num(k, _) => Value::Num(*k, 0),
        _ => Value::Int(0),
    }
}

/// Whether `p` is `m[k]` with `k` not in `m` (type.map.7).
fn missing_key(env: &Env<'_>, p: &Place) -> Result<bool, Trap> {
    let Some((Step::Key(k, _), parent)) = p.path.split_last() else { return Ok(false) };
    let parent = Place { slot: p.slot, path: parent.to_vec() };
    Ok(matches!(peek(env, &parent)?, Value::Map(t) if t.get(k).is_none()))
}

#[inline(always)]
fn load(env: &Env<'_>, p: &Place) -> Result<Value, Trap> {
    view(env, p, Value::clone)
}

/// own.ref.7: a cell whose contents are out for a `mut` access (a `mut self`
/// method, a `mut` argument, `for mut`) can't be used through another copy.
fn busy<T>(pos: Pos) -> Result<T, Trap> {
    trap(
        "own.ref.7",
        pos,
        "this `ref` cell is used while a `mut` access to it is in progress (another copy of the reference is being changed)"
            .into(),
    )
}

/// The position for a trap at a place, when there is one in its path.
fn place_pos(p: &Place) -> Pos {
    p.path
        .iter()
        .find_map(|s| match s {
            Step::Index(_, pos) | Step::Key(_, pos) => Some(*pos),
            Step::Field(_) | Step::Cell(_) => None,
        })
        .unwrap_or(Pos { line: 0, col: 0 })
}

/// Look at the value at a place, which may be in a `ref` cell.
#[inline(always)]
fn view<R>(env: &Env<'_>, p: &Place, f: impl FnOnce(&Value) -> R) -> Result<R, Trap> {
    match p.cell() {
        None => Ok(f(peek(env, p)?)),
        Some(c) => {
            let Ok(b) = c.try_borrow() else { return busy(place_pos(p)) };
            if matches!(*b, Value::Unit) {
                return busy(place_pos(p));
            }
            Ok(f(peek_in(&b, &p.path[1..])?))
        }
    }
}

/// Change the value at a place, which may be in a `ref` cell (own.ref.4).
#[inline(always)]
fn with_slot<R>(env: &mut Env<'_>, p: &Place, insert: bool, f: impl FnOnce(&mut Value) -> R) -> Result<R, Trap> {
    match p.cell() {
        None => Ok(f(slot_mut(env, p, insert)?)),
        Some(c) => {
            let Ok(mut b) = c.try_borrow_mut() else { return busy(place_pos(p)) };
            if matches!(*b, Value::Unit) {
                return busy(place_pos(p));
            }
            Ok(f(slot_in(&mut b, &p.path[1..], insert)?))
        }
    }
}

/// Move the value out of a place for a `mut` access; `put` returns it.
#[inline(always)]
fn take(env: &mut Env<'_>, p: &Place) -> Result<Value, Trap> {
    with_slot(env, p, false, |v| std::mem::replace(v, Value::Unit))
}

#[inline(always)]
fn put(env: &mut Env<'_>, p: &Place, insert: bool, v: Value) -> Result<(), Trap> {
    with_slot(env, p, insert, |slot| *slot = v)
}

/// own.ref.7: for a call, the contents of each `ref` cell that a `mut`
/// operand reaches are moved to a hidden local, so that another copy of the
/// reference can't see them half-changed: it finds the cell empty and traps.
#[inline(never)]
fn hoist(env: &mut Env<'_>, ops: &mut [Prepared], pos: Pos) -> Result<Vec<(usize, Rc<RefCell<Value>>)>, Trap> {
    let mut hoisted: Vec<(usize, Rc<RefCell<Value>>)> = Vec::new();
    for (place, _, mutable) in ops.iter_mut() {
        if !*mutable || !p_in_cell(place) {
            continue;
        }
        let Some(p) = place.as_mut() else { continue };
        let Step::Cell(c) = p.path.remove(0) else { unreachable!("a place in a cell") };
        let slot = match hoisted.iter().find(|(_, h)| Rc::ptr_eq(h, &c)) {
            Some((slot, _)) => *slot,
            None => {
                let Ok(mut b) = c.try_borrow_mut() else { return busy(pos) };
                if matches!(*b, Value::Unit) {
                    return busy(pos);
                }
                env.push(("", std::mem::replace(&mut *b, Value::Unit)));
                drop(b);
                hoisted.push((env.len() - 1, c));
                env.len() - 1
            }
        };
        p.slot = slot;
    }
    Ok(hoisted)
}

fn p_in_cell(p: &Option<Place>) -> bool {
    p.as_ref().is_some_and(|p| p.slot == IN_CELL)
}

/// Put hoisted cell contents back, last first.
fn unhoist(env: &mut Env<'_>, hoisted: Vec<(usize, Rc<RefCell<Value>>)>) {
    for (_, c) in hoisted.into_iter().rev() {
        let (_, v) = env.pop().expect("a hoisted cell");
        *c.borrow_mut() = v;
    }
}

/// Move a binding's value out (own.uniq.2, own.move.2); the checker makes
/// sure nothing uses the binding again.
fn move_out(env: &mut Env<'_>, n: &str) -> Value {
    let slot = env.iter().rposition(|(m, _)| *m == n).expect("checked: a local");
    std::mem::replace(&mut env[slot].1, Value::Unit)
}

/// The contents of a `ref` cell (own.ref.3), or the value itself.
#[inline]
fn deref(v: Value, pos: Pos) -> Result<Value, Trap> {
    match v {
        Value::Ref(c) => {
            let Ok(b) = c.try_borrow() else { return busy(pos) };
            if matches!(*b, Value::Unit) {
                return busy(pos);
            }
            Ok(b.clone())
        }
        v => Ok(v),
    }
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
fn check_overlap(ops: &[Prepared], pos: Pos) -> Result<(), Trap> {
    for (j, (b, _, b_mut)) in ops.iter().enumerate() {
        if !*b_mut {
            continue;
        }
        for (a, _, a_mut) in &ops[..j] {
            if let (Some(a), Some(b)) = (a, b)
                && *a_mut
                && overlaps(a, b)
            {
                return trap("own.excl.3", pos, "two `mut` arguments of this call overlap".into());
            }
        }
    }
    Ok(())
}

/// Two places overlap when one is the other or contains it (own.excl.1);
/// places in the same `ref` cell have the same root (own.ref.7).
fn overlaps(a: &Place, b: &Place) -> bool {
    a.slot == b.slot
        && a.path.iter().zip(&b.path).all(|(x, y)| match (x, y) {
            (Step::Cell(i), Step::Cell(j)) => Rc::ptr_eq(i, j),
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
        let mut env = self.env_pool.pop().unwrap_or_else(|| Vec::with_capacity(16));
        if let Some(r) = recv {
            env.push(("self", r));
        }
        env.extend(f.params.iter().map(|p| p.name.as_str()).zip(args));
        let result = self.run_body(f, &mut env, pos)?;
        // Only `mut` receivers and parameters are written back (own.param.2).
        let finals = if f.receiver.is_some_and(|r| r.mutable) || f.params.iter().any(|p| p.mutable) {
            env.drain(..).map(|(_, v)| v).collect()
        } else {
            Vec::new()
        };
        env.clear();
        self.env_pool.push(env);
        Ok((result, finals))
    }

    /// Run `f`'s body in `env`, which holds its receiver and parameters; on
    /// return `env` holds just those again.
    fn run_body(&mut self, f: &'a FnDecl, env: &mut Env<'a>, pos: Pos) -> Result<Value, Trap> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return trap("err.trap.1", pos, format!("call depth exceeded {MAX_DEPTH} in `{}`", f.full_name()));
        }
        let n = env.len();
        if self.tracing & trace::CALLS != 0 {
            self.trace_call(f, &env, pos);
        }
        let checked = contracts::active(f);
        if checked {
            self.check_requires(f, env, pos)?;
        }
        let body = f.body.as_ref().expect("checked: the function has a body");
        let mut outcome = self.block(body, env);
        // own.uniq.3: a `move` parameter owns its value until the call ends.
        if self.has_unique && f.params.iter().any(|p| p.moved) {
            let first = usize::from(f.receiver.is_some());
            for (k, _) in f.params.iter().enumerate().filter(|(_, p)| p.moved) {
                let v = std::mem::replace(&mut env[first + k].1, Value::Unit);
                keep_first(&mut outcome, self.release(v, pos).map_err(Exit::from));
            }
        }
        let (mut result, at) = match outcome {
            Ok(v) => (v, None),
            Err(Exit::Return) => {
                let (v, at) = self.returned.take().expect("a return leaves its value");
                (v, Some(at))
            }
            Err(Exit::Trap(t)) => return Err(self.unwound(f, env, pos, *t)),
            Err(Exit::Break | Exit::Continue) => unreachable!("checked: `break` and `continue` are inside loops"),
        };
        // err.decl.3: an error value leaving a fallible function is its failure.
        if matches!(f.ret.as_ref().map(|t| &t.kind), Some(TypeKind::Fallible { .. })) && self.is_error(&result) {
            let at = at.unwrap_or_else(|| body.last().map_or(f.pos, Stmt::pos));
            result = Value::Fail(Rc::new(Failure { error: result, context: Vec::new(), trace: vec![at] }));
        }
        if checked {
            let at = at.unwrap_or_else(|| body.last().map_or(f.pos, Stmt::pos));
            if let Err(t) = self.check_ensures(f, env, &result, at) {
                return Err(self.unwound(f, env, pos, t));
            }
        }
        if self.tracing & trace::CALLS != 0 {
            self.trace_return(f, &result);
        }
        self.depth -= 1;
        env.truncate(n);
        Ok(result)
    }

    #[inline]
    fn block(&mut self, stmts: &'a Block, env: &mut Env<'a>) -> Flow<Value> {
        if self.has_unique || self.has_defer {
            return self.owning_block(stmts, env);
        }
        let mark = env.len();
        let mut result = Ok(Value::Unit);
        for s in stmts {
            result = self.stmt(s, env);
            if result.is_err() {
                if matches!(result, Err(Exit::Trap(_))) && self.snap.is_some() {
                    self.snap_block(env);
                }
                break;
            }
        }
        // conc.task.4: the block's tasks and groups finish before it ends.
        if self.conc.is_some()
            && !matches!(result, Err(Exit::Trap(_)))
            && let Some(last) = stmts.last()
            && let Err(t) = self.end_tasks(&env[mark..], last.pos())
        {
            result = Err(t.into());
        }
        env.truncate(mark);
        result
    }

    /// A block whose exit releases unique values (own.uniq.3-4) and runs
    /// deferred statements (own.defer.1-2): in reverse order of their
    /// declarations and `defer`s, by any exit, including a trap.
    #[inline(never)]
    fn owning_block(&mut self, stmts: &'a Block, env: &mut Env<'a>) -> Flow<Value> {
        let mark = env.len();
        let pos = stmts.first().map_or(Pos { line: 0, col: 0 }, Stmt::pos);
        let mut defers: Vec<(usize, &'a Stmt)> = Vec::new();
        let mut result = Ok(Value::Unit);
        for (i, s) in stmts.iter().enumerate() {
            if let Stmt::Defer(inner, _) = s {
                defers.push((env.len(), inner));
                continue;
            }
            result = self.stmt(s, env);
            // A new unique value nobody took is released at once.
            if i + 1 < stmts.len()
                && let Stmt::Expr(e) = s
                && !matches!(e, Expr::Name(..) | Expr::Field { .. } | Expr::Index { .. } | Expr::If { .. } | Expr::Match { .. })
                && let Ok(v) = &mut result
            {
                let v = std::mem::replace(v, Value::Unit);
                if let Err(t) = self.release(v, pos) {
                    result = Err(t.into());
                }
            }
            if result.is_err() {
                break;
            }
        }
        let mut d = defers.len();
        let mut slot = env.len();
        while slot > mark {
            slot -= 1;
            while d > 0 && defers[d - 1].0 > slot {
                d -= 1;
                keep_first(&mut result, self.stmt(defers[d].1, env).map(|_| ()));
            }
            if self.has_unique {
                let v = std::mem::replace(&mut env[slot].1, Value::Unit);
                keep_first(&mut result, self.release(v, pos).map_err(Exit::from));
            }
        }
        while d > 0 {
            d -= 1;
            keep_first(&mut result, self.stmt(defers[d].1, env).map(|_| ()));
        }
        env.truncate(mark);
        result
    }

    /// Whether a value is unique and so needs releasing (own.uniq.3).
    fn is_unique(&self, v: &Value) -> bool {
        match v {
            Value::Struct(s, _) => self.prog.structs[*s].unique,
            Value::Enum(e, _, _) => self.prog.enums[*e].unique,
            _ => false,
        }
    }

    /// Release a value whose owner's scope ends (own.uniq.3): its type's
    /// `drop`, then its unique fields, last first. Other values need nothing.
    fn release(&mut self, v: Value, pos: Pos) -> Result<(), Trap> {
        if !self.is_unique(&v) {
            return Ok(());
        }
        let mut v = v;
        let con = match &v {
            Value::Struct(s, _) => Con::Struct(*s),
            Value::Enum(e, _, _) => Con::Enum(*e),
            _ => unreachable!("a unique value"),
        };
        if let Some(drop) = self.resolved.std.drop
            && let Some(&i) = self.resolved.impls.get(&(drop, con))
            && let Some(&m) = self.prog.impls[i].methods.iter().find(|&&m| self.prog.fns[m].name == "drop")
        {
            let prog = self.prog;
            let (_, finals) = self.call(&prog.fns[m], Some(v), Vec::new(), pos)?;
            v = finals.into_iter().next().unwrap_or(Value::Unit);
        }
        if let Value::Struct(_, fields) | Value::Enum(_, _, fields) = v {
            for f in Rc::unwrap_or_clone(fields).into_iter().rev() {
                self.release(f, pos)?;
            }
        }
        Ok(())
    }

    /// Store into a place; a unique value it held is released (own.uniq.3).
    fn replace_at(&mut self, env: &mut Env<'a>, p: &Place, insert: bool, v: Value) -> Result<(), Trap> {
        if !self.has_unique {
            return put(env, p, insert, v);
        }
        let old = with_slot(env, p, insert, |slot| std::mem::replace(slot, v))?;
        self.release(old, place_pos(p))
    }

    fn stmt(&mut self, s: &'a Stmt, env: &mut Env<'a>) -> Flow<Value> {
        if self.hook {
            self.hit(s.pos())?;
        }
        match s {
            Stmt::Let { name, value, otherwise, mutable, pos } => {
                let v = if otherwise.is_some() { self.else_value(value, env)? } else { self.eval(value, env)? };
                let v = match (otherwise, v) {
                    (Some(o), v @ (Value::None | Value::Fail(_))) => self.otherwise(o, v, env)?,
                    (Some(_), v) => Value::unwrap_present(v),
                    (_, v) => v,
                };
                env.push((name.as_str(), v));
                if self.tracing & trace::WATCH != 0 && *mutable {
                    self.trace_watch(name, env, *pos);
                }
            }
            Stmt::Assign { target, op, value, pos } => self.assign(target, *op, value, *pos, env)?,
            Stmt::While { cond, body, pos } => {
                while self.cond(cond, env)? {
                    if self.hook {
                        self.hit(*pos)?;
                    }
                    match self.block(body, env) {
                        Ok(_) | Err(Exit::Continue) => {}
                        Err(Exit::Break) => break,
                        Err(e) => return Err(e),
                    }
                }
            }
            Stmt::For { var, mutable: true, iter, body, .. } => self.for_mut(var, iter, body, env)?,
            Stmt::For { var, iter, body, pos, .. } => {
                if !self.par_for(*pos, var, iter, body, env)? {
                    self.for_loop(var, iter, body, env)?;
                }
            }
            Stmt::Break(_) => return Err(Exit::Break),
            Stmt::Continue(_) => return Err(Exit::Continue),
            Stmt::Return(value, pos) => {
                let v = match value {
                    Some(e) => self.eval(e, env)?,
                    None => Value::Unit,
                };
                self.returned = Some((v, *pos));
                return Err(Exit::Return);
            }
            Stmt::Expr(Expr::If { branches, els, else_err, .. }) => return self.eval_if(branches, els, else_err, env),
            Stmt::Expr(e) => return self.eval(e, env),
            // err.else.4: the `else` runs only on a failure.
            Stmt::Handle { value, otherwise } => {
                if let v @ Value::Fail(_) = self.eval(value, env)? {
                    self.otherwise(otherwise, v, env)?;
                }
            }
            // own.defer.1: `owning_block` runs it when the block exits.
            Stmt::Defer(..) => {}
        }
        Ok(Value::Unit)
    }

    /// The value of `e else ...`: an optional `?` inside it that finds
    /// `none` makes the whole value `none` (err.prop.4, ADR 0252).
    fn else_value(&mut self, value: &'a Expr, env: &mut Env<'a>) -> Flow<Value> {
        match self.eval(value, env) {
            Err(Exit::Return) if matches!(&self.returned, Some((Value::None, pos)) if self.resolved.else_try.contains(pos)) => {
                self.returned = None;
                Ok(Value::None)
            }
            r => r,
        }
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
        // own.ref.7: a list in a `ref` cell is out of the cell during the loop.
        let mut ops = [(Some(list), None, true)];
        let hoisted = hoist(env, &mut ops, iter.pos())?;
        let [(Some(list), _, _)] = ops else { unreachable!("the list's place") };
        let r = self.for_mut_in(var, &list, iter.pos(), body, env);
        unhoist(env, hoisted);
        r
    }

    fn for_mut_in(&mut self, var: &'a str, list: &Place, pos: Pos, body: &'a Block, env: &mut Env<'a>) -> Flow<()> {
        let Value::List(items) = peek(env, list)? else { unreachable!("checked: list") };
        for i in 0..items.len() {
            let mut at = list.clone();
            at.path.push(Step::Index(i as i64, pos));
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

    /// Run a comprehension's clauses from the first, collecting each element
    /// into the list or table (expr.comp.3). `continue` skips an element and
    /// `break` ends the comprehension (expr.comp.4); false if it ended.
    fn comp(
        &mut self,
        clauses: &'a [CompFor],
        elem: &'a Expr,
        value: Option<&'a Expr>,
        set: bool,
        env: &mut Env<'a>,
        acc: &mut (Vec<Value>, Table),
    ) -> Flow<bool> {
        let Some((c, rest)) = clauses.split_first() else {
            let k = self.eval(elem, env)?;
            match value {
                Some(v) => {
                    let v = self.eval(v, env)?;
                    acc.1.insert(k, Some(v));
                }
                None if set => {
                    acc.1.insert(k, None);
                }
                None => acc.0.push(k),
            }
            return Ok(true);
        };
        let items: Rc<Vec<Value>> = match &c.iter {
            Expr::Range { lo: Some(lo), hi: Some(hi), inclusive, .. } => {
                let (lo, hi) = (self.eval(lo, env)?.int(), self.eval(hi, env)?.int());
                let hi = if *inclusive { hi } else { hi.saturating_sub(1) };
                Rc::new(if lo > hi { Vec::new() } else { (lo..=hi).map(Value::Int).collect() })
            }
            iter => match self.eval(iter, env)? {
                Value::List(items) => items,
                Value::Set(t) => Rc::new(t.key_list()),
                other => unreachable!("checked: iterable, found {other:?}"),
            },
        };
        for item in items.iter() {
            env.push((&c.var, item.clone()));
            let r = match &c.cond {
                Some(cond) => match self.eval(cond, env) {
                    Ok(Value::Bool(true)) => self.comp(rest, elem, value, set, env, acc),
                    Ok(_) => Ok(true),
                    Err(e) => Err(e),
                },
                None => self.comp(rest, elem, value, set, env, acc),
            };
            env.pop();
            match r {
                Ok(true) | Err(Exit::Continue) => {}
                Ok(false) | Err(Exit::Break) => return Ok(false),
                Err(e) => return Err(e),
            }
        }
        Ok(true)
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
        // Releasing a unique value and `--watch` need the general steps; most programs need neither.
        let plain = !self.has_unique && self.tracing & trace::WATCH == 0;
        // A plain local needs no `Place`: same steps as below with an empty path.
        if let Expr::Name(n, _) = target {
            let slot = env.iter().rposition(|(m, _)| name_eq(m, n)).expect("checked: a local");
            // An int local updated with int operands is overwritten in place: no `Value` built or dropped.
            if plain && matches!(env[slot].1, Value::Int(_)) {
                let new = match op {
                    None => self.int_val(value, env),
                    Some(op) => match (&env[slot].1, self.int_leaf(value, env)) {
                        (Value::Int(a), Some(b)) => int_arith(op, *a, b),
                        _ => None,
                    },
                };
                if let (Some(r), Value::Int(a)) = (new, &mut env[slot].1) {
                    *a = r;
                    return Ok(());
                }
            }
            let v = match op {
                None => self.eval(value, env)?,
                Some(op) => {
                    let old = env[slot].1.clone();
                    let rhs = self.eval(value, env)?;
                    self.binop(op, old, rhs, pos)?
                }
            };
            let old = std::mem::replace(&mut env[slot].1, v);
            if self.has_unique {
                self.release(old, pos)?;
            }
            if self.tracing & trace::WATCH != 0 {
                self.trace_watch(n, env, target.pos());
            }
            return Ok(());
        }
        // `local.field = e` and `local.field op= e`, the commonest place: same steps as below, no `Place`.
        if plain
            && let Expr::Field { recv, name, .. } = target
            && let Expr::Name(n, _) = &**recv
            && let Some(slot) = env.iter().rposition(|(m, _)| name_eq(m, n))
            && let Value::Struct(s, _) = &env[slot].1
        {
            let index = self.field_index(*s, name);
            let v = match op {
                None => self.eval(value, env)?,
                Some(op) => {
                    let Value::Struct(_, fields) = &env[slot].1 else { unreachable!("checked: struct") };
                    let old = fields[index].clone();
                    let rhs = self.eval(value, env)?;
                    self.binop(op, old, rhs, pos)?
                }
            };
            let Value::Struct(_, fields) = &mut env[slot].1 else { unreachable!("checked: struct") };
            Rc::make_mut(fields)[index] = v;
            return Ok(());
        }
        // `local[i] = e` and `local[i] op= e`: same steps as below, with the one step on the stack.
        if plain
            && let Expr::Index { recv, index, pos: at } = target
            && let Expr::Name(n, _) = &**recv
            && let Some(slot) = env.iter().rposition(|(m, _)| name_eq(m, n))
            && matches!(env[slot].1, Value::List(_) | Value::Map(_))
        {
            let is_map = matches!(env[slot].1, Value::Map(_));
            let i = self.eval(index, env)?;
            // type.map.7: `m[k] op= v` on a missing key starts from zero.
            let missing = op.is_some() && matches!(&env[slot].1, Value::Map(t) if t.get(&i).is_none());
            let steps = [if is_map { Step::Key(i, *at) } else { Step::Index(i.int(), *at) }];
            let v = match op {
                None => self.eval(value, env)?,
                Some(op) if missing => {
                    let rhs = self.eval(value, env)?;
                    self.binop(op, zero_like(&rhs), rhs, pos)?
                }
                Some(op) => {
                    let old = peek_at(env, slot, &steps)?.clone();
                    let rhs = self.eval(value, env)?;
                    self.binop(op, old, rhs, pos)?
                }
            };
            *slot_mut_at(env, slot, &steps, op.is_none() || missing)? = v;
            return Ok(());
        }
        let p = self.place(target, env)?;
        let missing = op.is_some() && missing_key(env, &p)?;
        let v = match op {
            None => self.eval(value, env)?,
            // type.map.7: `m[k] op= v` on a missing key starts from zero.
            Some(op) if missing => {
                let rhs = self.eval(value, env)?;
                self.binop(op, zero_like(&rhs), rhs, pos)?
            }
            Some(op) => {
                let old = load(env, &p)?;
                let rhs = self.eval(value, env)?;
                self.binop(op, old, rhs, pos)?
            }
        };
        self.replace_at(env, &p, op.is_none() || missing, v)?;
        if self.tracing & trace::WATCH != 0
            && let Some((root, _)) = place_path(target)
        {
            self.trace_watch(root, env, target.pos());
        }
        Ok(())
    }

    fn place(&mut self, e: &'a Expr, env: &mut Env<'a>) -> Flow<Place> {
        match e {
            Expr::Name(n, _) => {
                let slot = env.iter().rposition(|(m, _)| name_eq(m, n)).expect("checked: a local");
                Ok(Place::local(slot))
            }
            Expr::Field { recv, name, .. } => {
                let mut p = self.place(recv, env)?;
                let s = match view(env, &p, |v| match v {
                    Value::Struct(s, _) => Ok(*s),
                    Value::Ref(c) => Err(c.clone()),
                    other => unreachable!("checked: struct, found {other:?}"),
                })? {
                    Ok(s) => s,
                    // own.ref.3: the field of the struct in the cell.
                    Err(c) => {
                        p = Place::in_cell(c);
                        view(env, &p, |v| match v {
                            Value::Struct(s, _) => *s,
                            other => unreachable!("checked: struct, found {other:?}"),
                        })?
                    }
                };
                p.path.push(Step::Field(self.field_index(s, name)));
                Ok(p)
            }
            Expr::Index { recv, index, pos } => {
                let mut p = self.place(recv, env)?;
                let is_map = view(env, &p, |v| matches!(v, Value::Map(_)))?;
                let i = self.eval(index, env)?;
                p.path.push(if is_map { Step::Key(i, *pos) } else { Step::Index(i.int(), *pos) });
                Ok(p)
            }
            _ => unreachable!("checked: a place"),
        }
    }

    /// Evaluate a call operand: a place (rooted in a local) stays a place.
    fn operand(&mut self, e: &'a Expr, env: &mut Env<'a>) -> Flow<Operand> {
        if self.has_moves
            && let Expr::Name(n, pos) = e
            && self.resolved.moves.contains_key(pos)
        {
            return Ok(Operand::Value(move_out(env, n)));
        }
        if place_root(e).is_some_and(|root| lookup(env, root).is_some()) {
            Ok(Operand::Place(self.place(e, env)?))
        } else {
            Ok(Operand::Value(self.eval(e, env)?))
        }
    }

    fn field_index(&self, s: usize, name: &str) -> usize {
        self.prog.structs[s].fields.iter().position(|f| name_eq(&f.name, name)).expect("checked: field exists")
    }

    /// The hot expression forms, kept in a small frame: `eval_general`'s frame
    /// is over a kilobyte, and most evaluations are names, literals and
    /// operators (ADR 0175). Every other form, and every fallback, is
    /// `eval_general`'s, so the two stay in step by construction.
    fn eval(&mut self, e: &'a Expr, env: &mut Env<'a>) -> Flow<Value> {
        match e {
            Expr::Binary { op, lhs, rhs, pos } if !matches!(op, BinOp::And | BinOp::Or) => {
                self.eval_binary(*op, lhs, rhs, *pos, env)
            }
            Expr::Int(v, pos) if !self.lit_maybe(pos) || !self.resolved.float_literals.contains(pos) => Ok(Value::Int(*v)),
            Expr::Bool(b, _) => Ok(Value::Bool(*b)),
            Expr::Char(c, _) => Ok(Value::Char(*c)),
            Expr::Name(n, pos) if !self.has_moves || !self.resolved.moves.contains_key(pos) => match lookup(env, n) {
                Some(v) => Ok(v.clone()),
                None => self.eval_general(e, env),
            },
            Expr::If { branches, els, else_err, .. } => self.eval_if(branches, els, else_err, env),
            Expr::Binary { op: BinOp::And, lhs, rhs, .. } => {
                Ok(Value::Bool(self.eval(lhs, env)?.bool() && self.eval(rhs, env)?.bool()))
            }
            Expr::Binary { op: BinOp::Or, lhs, rhs, .. } => {
                Ok(Value::Bool(self.eval(lhs, env)?.bool() || self.eval(rhs, env)?.bool()))
            }
            Expr::Unary { op: UnOp::Not, operand, .. } => Ok(Value::Bool(!self.eval(operand, env)?.bool())),
            Expr::Call { callee, args, pos } => self.call_expr(callee, args, *pos, env),
            Expr::Method { recv, name, name_pos, args } => self.method_call(recv, name, args, *name_pos, env),
            Expr::Field { recv, name, name_pos } => self.eval_field(recv, name, *name_pos, env),
            Expr::Index { recv, index, pos } => self.eval_index(recv, index, *pos, env),
            _ => self.eval_general(e, env),
        }
    }

    /// `if` and `else if` chains (expr.if.1-5). Its own small frame, called
    /// from the hot `eval` as well as from `eval_general`.
    #[inline(never)]
    fn eval_if(
        &mut self,
        branches: &'a [(Cond, Block)],
        els: &'a Option<Block>,
        else_err: &'a Option<(String, Pos)>,
        env: &mut Env<'a>,
    ) -> Flow<Value> {
        let mut failure = None;
        for (cond, body) in branches {
            match cond {
                Cond::Expr(c) => {
                    if self.cond(c, env)? {
                        return self.block(body, env);
                    }
                }
                // expr.if.5: run the branch with the value bound, if there is one.
                Cond::Bind { name, value, pos } => match self.eval(value, env)? {
                    Value::None => {}
                    Value::Fail(f) => failure = Some(f),
                    v => {
                        let mark = env.len();
                        env.push((name.as_str(), Value::unwrap_present(v)));
                        let mut r = self.block(body, env);
                        if self.has_unique {
                            let v = std::mem::replace(&mut env[mark].1, Value::Unit);
                            keep_first(&mut r, self.release(v, *pos).map_err(Exit::from));
                        }
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

    #[inline(always)]
    fn lit_maybe(&self, pos: &Pos) -> bool {
        self.lit_mask >> (pos.line & 63) & 1 != 0
    }

    /// `recv.name`: a field, or a unit variant `Enum.Variant` (type.struct.2, type.enum.2).
    #[inline(never)]
    fn eval_field(&mut self, recv: &'a Expr, name: &'a str, name_pos: Pos, env: &mut Env<'a>) -> Flow<Value> {
        // std.ui.17: a named constant such as `ui.RED` is an int known at check time.
        if let Some(&crate::stdlib::StdCall::Const(v)) = self.resolved.std_calls.get(&name_pos) {
            return Ok(Value::Int(v));
        }
        if let Some(Value::Struct(s, fields)) = self.borrowed(recv, env) {
            return Ok(fields[self.field_index(*s, name)].clone());
        }
        if let Some(tn) = type_head(recv, env, self.prog) {
            let e = self.enums[tn];
            let v = self.variant_index(e, name);
            return Ok(Value::Enum(e, v, Rc::clone(&self.no_fields)));
        }
        let v = self.operand_value(recv, env)?;
        let Value::Struct(s, fields) = deref(v, name_pos)? else { unreachable!("checked: struct") };
        Ok(fields[self.field_index(s, name)].clone())
    }

    /// `recv[index]` on a list or map, and `recv[lo..hi]` (type.coll.4, type.coll.10, type.map.3).
    #[inline(never)]
    fn eval_index(&mut self, recv: &'a Expr, index: &'a Expr, pos: Pos, env: &mut Env<'a>) -> Flow<Value> {
        let index: &'a Expr = index;
        // A list in a local or a field, at an int that needs no evaluation: read in place, without
        // taking a reference to the list.
        if let Some(i) = self.int_val(index, env)
            && let Some(Value::List(items)) = self.borrowed(recv, env)
        {
            return Ok(element(items, i, pos)?.clone());
        }
        let items = match self.eval(recv, env)? {
            Value::List(items) => items,
            Value::Map(t) => {
                let k = self.eval(index, env)?;
                return Ok(map_value(&t, &k, || key_text(&k), pos)?.clone());
            }
            other => unreachable!("checked: list or map, found {other:?}"),
        };
        if let Expr::Range { lo, hi, inclusive, pos: range_pos } = index {
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
        Ok(element(&items, i, pos)?.clone())
    }

    /// An arithmetic or comparison operator (expr.arith.1-3). Its own frame keeps
    /// `eval`'s small for the leaves.
    #[inline(never)]
    fn eval_binary(&mut self, op: BinOp, lhs: &'a Expr, rhs: &'a Expr, pos: Pos, env: &mut Env<'a>) -> Flow<Value> {
        // Int names and literals as operands need no evaluation call or clone (ADR 0219).
        if let (Some(a), Some(b)) = (self.int_val(lhs, env), self.int_val(rhs, env))
            && let Some(v) = int_op(op, a, b)
        {
            return Ok(v);
        }
        let l = self.operand_value(lhs, env)?;
        let r = self.operand_value(rhs, env)?;
        if let (Value::Int(a), Value::Int(b)) = (&l, &r)
            && let Some(v) = int_op(op, *a, *b)
        {
            return Ok(v);
        }
        Ok(self.binop(op, l, r, pos)?)
    }

    /// `eval` with names and int literals handled in the caller's frame: most
    /// operands are leaves, and a call for each costs more than reading one
    /// (ADR 0219).
    #[inline(always)]
    fn operand_value(&mut self, e: &'a Expr, env: &mut Env<'a>) -> Flow<Value> {
        match e {
            Expr::Name(n, pos) if !self.has_moves || !self.resolved.moves.contains_key(pos) => {
                if let Some(v) = lookup(env, n) {
                    return Ok(v.clone());
                }
            }
            Expr::Int(v, pos) if !self.lit_maybe(pos) || !self.resolved.float_literals.contains(pos) => return Ok(Value::Int(*v)),
            _ => {}
        }
        self.eval(e, env)
    }

    /// The value of an int local or literal, without cloning.
    #[inline(always)]
    fn int_leaf(&self, e: &Expr, env: &Env<'a>) -> Option<i64> {
        match e {
            Expr::Name(n, _) => match lookup(env, n) {
                Some(Value::Int(v)) => Some(*v),
                _ => None,
            },
            Expr::Int(v, pos) if !self.lit_maybe(pos) || !self.resolved.float_literals.contains(pos) => Some(*v),
            _ => None,
        }
    }

    /// The value a local or a chain of fields of a local holds, without cloning it.
    fn borrowed<'e>(&self, e: &Expr, env: &'e Env<'a>) -> Option<&'e Value> {
        match e {
            Expr::Name(n, _) => lookup(env, n),
            Expr::Field { recv, name, .. } => match self.borrowed(recv, env)? {
                Value::Struct(s, fields) => Some(&fields[self.field_index(*s, name)]),
                _ => None,
            },
            _ => None,
        }
    }

    /// An int leaf, or one arithmetic operator over two int leaves, as `n % 2`
    /// or `3 * n`. Pure, and bounded, so a miss costs a constant and the caller
    /// evaluates the operand normally (ADR 0219).
    #[inline(always)]
    fn int_val(&self, e: &Expr, env: &Env<'a>) -> Option<i64> {
        match e {
            Expr::Binary { op, lhs, rhs, .. } => int_arith(*op, self.int_leaf(lhs, env)?, self.int_leaf(rhs, env)?),
            _ => self.int_leaf(e, env),
        }
    }

    /// A condition's truth value; a comparison of int operands never builds a `Value`.
    #[inline(always)]
    fn cond(&mut self, e: &'a Expr, env: &mut Env<'a>) -> Flow<bool> {
        if let Expr::Binary { op, lhs, rhs, .. } = e
            && let Some(a) = self.int_val(lhs, env)
            && let Some(b) = self.int_val(rhs, env)
            && let Some(t) = int_cmp(*op, a, b)
        {
            return Ok(t);
        }
        Ok(self.eval(e, env)?.bool())
    }

    #[inline(never)]
    fn eval_general(&mut self, e: &'a Expr, env: &mut Env<'a>) -> Flow<Value> {
        Ok(match e {
            // A literal's text is shared, not copied, on every evaluation.
            Expr::Str(s, _) => {
                Value::Str(Rc::clone(self.str_lits.entry(s.as_ptr() as usize).or_insert_with(|| Rc::from(s.as_str()))))
            }
            // type.num.13: a literal typed by context as a sized number.
            Expr::Int(v, pos) if self.lit_maybe(pos) && self.resolved.sized_literals.contains_key(pos) => {
                sized_literal(self.resolved.sized_literals[pos], *v as f64, *v)
            }
            Expr::Float(v, pos) if self.lit_maybe(pos) && self.resolved.sized_literals.contains_key(pos) => {
                sized_literal(self.resolved.sized_literals[pos], *v, 0)
            }
            // type.num.10
            Expr::Int(v, pos) if self.lit_maybe(pos) && self.resolved.float_literals.contains(pos) => Value::Float(*v as f64),
            Expr::Int(v, _) => Value::Int(*v),
            Expr::Float(v, _) => Value::Float(*v),
            Expr::Char(c, _) => Value::Char(*c),
            Expr::Bool(b, _) => Value::Bool(*b),
            Expr::None(_) => Value::None,
            Expr::Hole(pos) => {
                return Err(Exit::Trap(Box::new(Trap {
                    rule: "expr.hole.2",
                    pos: *pos,
                    message: "reached a `todo` hole".into(),
                })));
            }
            // err.else.6: the fallback runs only when the value is absent or failed.
            Expr::Else { value, otherwise, .. } => match self.else_value(value, env)? {
                v @ (Value::None | Value::Fail(_)) => self.otherwise(otherwise, v, env)?,
                v => Value::unwrap_present(v),
            },
            Expr::Try { expr, context, pos } => {
                let v = self.eval(expr, env)?;
                match v {
                    Value::None if self.resolved.absent_try.contains(pos) => {
                        return Err(self.absent(context.as_deref(), *pos, env)?);
                    }
                    Value::None if self.resolved.optional_try.contains(pos) => {
                        self.returned = Some((Value::None, *pos));
                        return Err(Exit::Return);
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
                        self.returned = Some((Value::Fail(Rc::new(failure)), *pos));
                        return Err(Exit::Return);
                    }
                    v if self.resolved.optional_try.contains(pos) => Value::unwrap_present(v),
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
            Expr::Name(n, pos) if self.has_moves && self.resolved.moves.contains_key(pos) => move_out(env, n),
            Expr::Name(n, pos) => match lookup(env, n) {
                Some(v) => v.clone(),
                None if self.prog.consts.iter().any(|c| c.name == *n) => self.constant(n, *pos)?,
                None => {
                    let Some(&(e, v)) = self.resolved.variants.get(pos) else {
                        unreachable!("checked: `{n}` at {}:{} is a variant", pos.line, pos.col)
                    };
                    Value::Enum(e, v, Rc::clone(&self.no_fields))
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
            Expr::Field { recv, name, name_pos } => return self.eval_field(recv, name, *name_pos, env),
            Expr::Index { recv, index, pos } => return self.eval_index(recv, index, *pos, env),
            Expr::Unary { op: UnOp::Neg, operand, pos } => {
                let v = match self.eval(operand, env)? {
                    Value::Float(x) => return Ok(Value::Float(-x)),
                    Value::Num(k, bits) => return negate(k, bits, *pos),
                    v => v.int(),
                };
                match v.checked_neg() {
                    Some(r) => Value::Int(r),
                    None => {
                        let message = format!("`-({v})` overflows `int`");
                        return Err(Trap { rule: "expr.arith.1", pos: *pos, message }.into());
                    }
                }
            }
            Expr::Unary { op: UnOp::Not, operand, .. } => Value::Bool(!self.eval(operand, env)?.bool()),
            Expr::Unary { op: UnOp::Spawn, operand, pos } => self.spawn(operand, *pos, env)?,
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
            Expr::Ref { value, .. } => Value::Ref(Rc::new(RefCell::new(self.eval(value, env)?))),
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
            Expr::Comp { kind, elem, value, clauses, .. } => {
                let mut acc = (Vec::new(), Table::default());
                self.comp(clauses, elem, value.as_deref(), *kind == CompKind::Set, env, &mut acc)?;
                match kind {
                    CompKind::List => Value::List(Rc::new(acc.0)),
                    CompKind::Set => Value::Set(Rc::new(acc.1)),
                    CompKind::Map => Value::Map(Rc::new(acc.1)),
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
            Expr::If { branches, els, else_err, .. } => return self.eval_if(branches, els, else_err, env),
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
            (Pattern::Variant { name, .. }, Value::Enum(e, i, _)) => {
                name_eq(&self.prog.enums[*e].variants[*i].name, name)
            }
            (Pattern::Literal(lit, _), v) => match (lit, v) {
                (Literal::Int(a), Value::Int(b)) => a == b,
                (Literal::Int(a), Value::Num(k, b)) => i128::from(*a) == k.decode(*b),
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
            (Value::Num(k, a), Value::Num(_, b)) => num_binop(op, k, a, b, pos)?,
            // Characters and booleans compare without the trait-aware generic path.
            (Value::Char(a), Value::Char(b)) if !self.user_std[0] && !self.user_std[1] => Value::Bool(match op {
                BinOp::Eq => a == b,
                BinOp::Ne => a != b,
                BinOp::Lt => a < b,
                BinOp::Le => a <= b,
                BinOp::Gt => a > b,
                _ => a >= b,
            }),
            (Value::Bool(a), Value::Bool(b)) if matches!(op, BinOp::Eq | BinOp::Ne) && !self.user_std[0] => {
                Value::Bool((a == b) == (op == BinOp::Eq))
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
        if self.user_std[0]
            && let Some(f) = self.dispatch(self.resolved.std.eq, "eq", a)
        {
            let (r, _) = self.call(f, Some(a.clone()), vec![b.clone()], pos)?;
            return Ok(r.bool());
        }
        Ok(match (a, b) {
            (Value::Int(x), Value::Int(y)) => x == y,
            (Value::Num(k, x), Value::Num(_, y)) if k.is_float() => Num::f32_of(*x) == Num::f32_of(*y),
            (Value::Num(_, x), Value::Num(_, y)) => x == y,
            (Value::Float(x), Value::Float(y)) => x == y,
            (Value::Char(x), Value::Char(y)) => x == y,
            (Value::Bool(x), Value::Bool(y)) => x == y,
            (Value::Str(x), Value::Str(y)) => x == y,
            (Value::None, Value::None) => true,
            (Value::None, _) | (_, Value::None) => false,
            (Value::Nested(m), Value::Nested(n)) => m == n,
            (Value::Nested(_), _) | (_, Value::Nested(_)) => false,
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
            // own.ref.6: references are equal when they share a cell.
            (Value::Ref(x), Value::Ref(y)) => Rc::ptr_eq(x, y),
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
        if self.user_std[1]
            && let Some(f) = self.dispatch(self.resolved.std.ord, "cmp", a)
        {
            let (r, _) = self.call(f, Some(a.clone()), vec![b.clone()], pos)?;
            let Value::Enum(_, v, _) = r else { unreachable!("checked: `cmp` returns an `Ordering`") };
            return Ok([Ordering::Less, Ordering::Equal, Ordering::Greater][v]);
        }
        Ok(match (a, b) {
            (Value::Int(x), Value::Int(y)) => x.cmp(y),
            (Value::Num(k, x), Value::Num(_, y)) if k.is_float() => {
                float_order(f64::from(Num::f32_of(*x)), f64::from(Num::f32_of(*y)))
            }
            (Value::Num(k, x), Value::Num(_, y)) => k.decode(*x).cmp(&k.decode(*y)),
            // type.num.7: a total order for sorting, with NaN after every number.
            (Value::Float(x), Value::Float(y)) => float_order(*x, *y),
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
        Value::Enum(self.resolved.std.ordering, v, Rc::clone(&self.no_fields))
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
            return Err(Trap { rule: "err.model.2", pos, message: format!("unreachable: {msg}") }.into());
        }
        if (n == "assert" || n == "assert_eq") && lookup(env, n).is_none() {
            return self.assert(n, args, pos, env);
        }
        if matches!(callee, Expr::TypeArgs { .. }) {
            match n.as_str() {
                "List" => return Ok(Value::List(Rc::new(Vec::new()))),
                "Map" => return Ok(Value::Map(Rc::new(Table::default()))),
                "Set" => return Ok(Value::Set(Rc::new(Table::default()))),
                _ => {}
            }
        }
        if matches!(n.as_str(), "TaskGroup" | "Channel" | "Mutex") {
            return self.conc_constructor(n, args, pos, env);
        }
        // What a call names doesn't change, so it is looked up once per call site.
        let key = callee as *const Expr as usize;
        let target = match self.call_targets.get(&key) {
            Some(&t) => t,
            None => {
                let t = if let Some(&s) = self.structs.get(n.as_str()) {
                    Target::Struct(s)
                } else if let Some(&(e, v)) = self.resolved.variants.get(&callee.pos()) {
                    Target::Variant(e, v)
                } else {
                    Target::Fn(self.fns[n.as_str()])
                };
                self.call_targets.insert(key, t);
                t
            }
        };
        match target {
            Target::Struct(s) => {
                let fields = self.labelled(&self.prog.structs[s].fields, args, env)?;
                Ok(Value::Struct(s, Rc::new(fields)))
            }
            Target::Variant(e, v) => {
                let fields = self.labelled(&self.prog.enums[e].variants[v].fields, args, env)?;
                let fields = if fields.is_empty() { Rc::clone(&self.no_fields) } else { Rc::new(fields) };
                Ok(Value::Enum(e, v, fields))
            }
            Target::Fn(f) => self.invoke(f, None, args, callee.pos(), env),
        }
    }

    /// Field values in declaration order from named, shorthand and
    /// positional arguments, evaluated in the order written (type.struct.3,
    /// type.struct.7, type.enum.6).
    fn labelled(&mut self, fields: &'a [crate::ast::Field], args: &'a [Arg], env: &mut Env<'a>) -> Flow<Vec<Value>> {
        // Which field each argument sets is fixed by the call site, so it is worked out once.
        let slots = match self.field_slots.get(&(args.as_ptr() as usize)) {
            Some(slots) => Rc::clone(slots),
            None => {
                let slots: Rc<[usize]> = crate::ast::field_slots(fields, args)
                    .into_iter()
                    .map(|(slot, _)| slot.expect("checked: field exists"))
                    .collect();
                self.field_slots.insert(args.as_ptr() as usize, Rc::clone(&slots));
                slots
            }
        };
        let mut values = vec![Value::Unit; fields.len()];
        for (a, &slot) in args.iter().zip(slots.iter()) {
            values[slot] = self.eval(&a.value, env)?;
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
        let numeric = matches!(v, Value::Int(_) | Value::Float(_) | Value::Num(..));
        let body = match v {
            Value::Num(k, bits) if k.is_float() && spec.precision.is_some() && Num::f32_of(*bits).is_finite() => {
                format!("{:.*}", spec.precision.unwrap_or(0), Num::f32_of(*bits))
            }
            Value::Num(k, bits) if spec.hex && !k.is_float() => {
                let n = k.decode(*bits);
                if n < 0 { format!("-{:x}", n.unsigned_abs()) } else { format!("{n:x}") }
            }
            Value::Float(x) if spec.precision.is_some() && x.is_finite() => {
                format!("{x:.*}", spec.precision.unwrap_or(0))
            }
            Value::Int(n) if spec.hex => {
                if *n < 0 {
                    format!("-{:x}", n.unsigned_abs())
                } else {
                    format!("{n:x}")
                }
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
        if self.user_std[2]
            && let Some(f) = self.dispatch(self.resolved.std.to_str, "to_str", v)
        {
            let (r, _) = self.call(f, Some(v.clone()), Vec::new(), pos)?;
            let Value::Str(s) = r else { unreachable!("checked: `to_str` returns a `str`") };
            return Ok(s.to_string());
        }
        let prog = self.prog;
        Ok(match v {
            Value::Unit => "()".into(),
            Value::Int(n) => n.to_string(),
            Value::Float(x) => float_text(*x),
            Value::Num(k, bits) if k.is_float() => float_text32(Num::f32_of(*bits)),
            Value::Num(k, bits) => k.decode(*bits).to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Str(s) => quoted(s, '"'),
            Value::Char(c) => quoted(&c.to_string(), '\''),
            Value::None => "none".into(),
            Value::Nested(n) => format!("{}none{}", "some(".repeat(*n as usize), ")".repeat(*n as usize)),
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
            Value::Handle(h) => h.text(),
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
            // own.ref.6: `ref` and the contents; a cell already being shown
            // (a cycle) is `ref ...`.
            Value::Ref(c) => {
                let ptr = Rc::as_ptr(c);
                if self.showing.contains(&ptr) {
                    return Ok("ref ...".into());
                }
                let inner = deref(v.clone(), pos)?;
                self.showing.push(ptr);
                let text = self.describe(&inner, pos);
                self.showing.pop();
                format!("ref {}", text?)
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
        // What a call site says about itself doesn't change, so it is worked out once.
        let key = recv as *const Expr as usize;
        let site = match self.method_sites.get(&key) {
            Some(&site) => site,
            None => {
                let site = MethodSite {
                    call: self.resolved.calls.get(&pos).copied(),
                    std: match self.resolved.std_calls.get(&pos) {
                        Some(&c @ crate::stdlib::StdCall::Method(_)) => Some(c),
                        _ => None,
                    },
                    mutating: self.mutating.contains(name),
                    last: None,
                };
                self.method_sites.insert(key, site);
                site
            }
        };
        let call = site.call;
        // `Trait.name(x, ...)` (trait.res.1).
        if let Some(Call::Qualified(tr)) = call {
            let (first, rest) = args.split_first().expect("checked: a receiver argument");
            let op = self.operand(&first.value, env)?;
            return self.trait_method(tr, name, op, rest, pos, env);
        }
        // `Type.Variant(field: value)` (type.enum.6), or an associated
        // function `Type.name(args)` (trait.method.4).
        if let Some(tn) = type_head(recv, env, self.prog) {
            // `m.f(args)` on a standard module (std.use.2).
            if let Some(&c) = self.resolved.std_calls.get(&pos) {
                return self.std_call(c, args, pos, env);
            }
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
        if let Some(crate::stdlib::StdCall::Method(m)) = site.std {
            return self.ui_method(m, recv, args, pos, env);
        }
        let mut recv = if site.mutating { self.operand(recv, env)? } else { Operand::Value(self.eval(recv, env)?) };
        let (mut kind, cell) = match &recv {
            Operand::Place(p) => view(env, p, |v| kind_or_cell(v))?,
            Operand::Value(v) => kind_or_cell(v),
        };
        // own.ref.3: a method of a `ref` runs on the cell's contents.
        if let Some(c) = cell {
            let p = Place::in_cell(c);
            kind = view(env, &p, method_kind)?;
            recv = Operand::Place(p);
        }
        if let Some(Call::Trait(tr)) = call {
            return self.trait_method(tr, name, recv, args, pos, env);
        }
        // The method for the receiver's type; a site usually sees one type, so the last one is kept.
        let (is_enum, index) = match kind {
            Some(MethodKind::Type(is_enum, index)) => (is_enum, index),
            Some(MethodKind::Handle(h)) => return self.handle_method(h, name, args, pos, env),
            None => return self.builtin(recv, name, args, pos, env),
        };
        let f = match site.last {
            Some((e, i, f)) if e == is_enum && i == index => f,
            _ => {
                let type_name = if is_enum { &self.prog.enums[index].name } else { &self.prog.structs[index].name };
                let f = self.methods[type_name.as_str()][name];
                if let Some(site) = self.method_sites.get_mut(&key) {
                    site.last = Some((is_enum, index, f));
                }
                f
            }
        };
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
            Operand::Place(p) => load(env, p)?,
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
        // No `mut` operand: no overlap check and nothing to write back, so the
        // operands go straight into the callee's environment.
        if f.receiver.is_none_or(|r| !r.mutable) && f.params.iter().all(|p| !p.mutable) {
            let mut callee = self.env_pool.pop().unwrap_or_else(|| Vec::with_capacity(16));
            match recv {
                Some(Operand::Place(p)) => callee.push(("self", load(env, &p)?)),
                Some(Operand::Value(v)) => callee.push(("self", v)),
                None => {}
            }
            for (a, param) in args.iter().zip(&f.params) {
                // A read-only operand is its value: reading a place and evaluating it agree.
                let v = self.operand_value(&a.value, env)?;
                callee.push((param.name.as_str(), v));
            }
            let result = self.run_body(f, &mut callee, pos)?;
            callee.clear();
            self.env_pool.push(callee);
            return Ok(result);
        }
        // `mut` operands that are plain locals (the usual `mut p` and `mut self`): each is moved out of
        // its slot and back, with no places or overlap tables; same steps and order as below.
        let recv_slot = match &recv {
            Some(Operand::Place(p)) if f.receiver.is_some_and(|r| r.mutable) && p.path.is_empty() => Some(Some(p.slot)),
            _ if f.receiver.is_some_and(|r| r.mutable) => None,
            _ => Some(None),
        };
        if let Some(recv_slot) = recv_slot
            && f.params.len() <= MAX_LOCAL_MOVES
            && args
                .iter()
                .zip(&f.params)
                .all(|(a, p)| !p.mutable || matches!(&a.value, Expr::Name(n, _) if lookup(env, n).is_some()))
        {
            let mut callee = self.env_pool.pop().unwrap_or_else(|| Vec::with_capacity(16));
            let mut moves = [(0usize, 0usize); MAX_LOCAL_MOVES + 1]; // (index in `callee`, slot in `env`)
            let mut count = 0;
            match recv {
                Some(Operand::Place(p)) => match recv_slot {
                    Some(slot) => {
                        moves[count] = (callee.len(), slot);
                        count += 1;
                        callee.push(("self", Value::Unit));
                    }
                    None => callee.push(("self", load(env, &p)?)),
                },
                Some(Operand::Value(v)) => callee.push(("self", v)),
                None => {}
            }
            for (a, param) in args.iter().zip(&f.params) {
                if param.mutable {
                    let Expr::Name(n, _) = &a.value else { unreachable!("checked above") };
                    let slot = env.iter().rposition(|(m, _)| name_eq(m, n)).expect("checked above");
                    moves[count] = (callee.len(), slot);
                    count += 1;
                    callee.push((param.name.as_str(), Value::Unit));
                } else {
                    let v = self.operand_value(&a.value, env)?;
                    callee.push((param.name.as_str(), v));
                }
            }
            // own.excl.3: `mut` operands that overlap trap before the call.
            for j in 1..count {
                if moves[..j].iter().any(|m| m.1 == moves[j].1) {
                    return Ok(trap("own.excl.3", pos, "two `mut` arguments of this call overlap".into())?);
                }
            }
            for &(at, slot) in &moves[..count] {
                callee[at].1 = std::mem::replace(&mut env[slot].1, Value::Unit);
            }
            let result = self.run_body(f, &mut callee, pos)?;
            // Only `mut` receivers and parameters are written back (own.param.2).
            for &(at, slot) in &moves[..count] {
                env[slot].1 = std::mem::replace(&mut callee[at].1, Value::Unit);
            }
            callee.clear();
            self.env_pool.push(callee);
            return Ok(result);
        }
        let has_recv = recv.is_some();
        let mut ops = self.ops_pool.pop().unwrap_or_default();
        if let Some(r) = recv {
            ops.push(prepare(r, f.receiver.is_some_and(|r| r.mutable), env)?);
        }
        for (a, param) in args.iter().zip(&f.params) {
            let op = self.operand(&a.value, env)?;
            ops.push(prepare(op, param.mutable, env)?);
        }
        check_overlap(&ops, pos)?;
        // The operands move straight into the callee's environment (receiver first).
        let in_cell = ops.iter().any(|(p, _, m)| *m && p_in_cell(p));
        let hoisted = if in_cell { hoist(env, &mut ops, pos)? } else { Vec::new() };
        let mut callee = self.env_pool.pop().unwrap_or_else(|| Vec::with_capacity(16));
        for (i, (place, value, _)) in ops.iter_mut().enumerate() {
            let name = match (has_recv, i) {
                (true, 0) => "self",
                (true, i) => f.params[i - 1].name.as_str(),
                (false, i) => f.params[i].name.as_str(),
            };
            let v = match value.take() {
                Some(v) => v,
                None => take(env, place.as_ref().expect("checked: `mut` operands are places"))?,
            };
            callee.push((name, v));
        }
        let result = match self.run_body(f, &mut callee, pos) {
            Ok(r) => r,
            Err(t) => {
                if !hoisted.is_empty() {
                    unhoist(env, hoisted);
                }
                return Err(t.into());
            }
        };
        // Only `mut` receivers and parameters are written back (own.param.2).
        for ((place, _, mutable), (_, v)) in ops.drain(..).zip(callee.drain(..)) {
            if mutable {
                put(env, &place.expect("checked: `mut` operands are places"), false, v)?;
            }
        }
        if !hoisted.is_empty() {
            unhoist(env, hoisted);
        }
        self.ops_pool.push(ops);
        self.env_pool.push(callee);
        Ok(result)
    }

    fn print(&mut self, v: &Value, pos: Pos) -> Result<(), Trap> {
        let text = self.text(v, pos)?;
        if self.conc.is_some() && self.task_print(&text) {
            return Ok(());
        }
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
        if self.conc.is_some() {
            self.task_print("");
        }
        match self.out.flush() {
            Err(e) if e.kind() != io::ErrorKind::BrokenPipe => {
                trap("mod.main.3", pos, format!("writing to standard output failed: {e}"))
            }
            _ => Ok(()),
        }
    }
}
