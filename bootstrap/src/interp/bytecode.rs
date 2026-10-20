//! The bytecode tier (ADR 0359, ADR 0361): a function's body compiled on its
//! first call to register code, run in place of the tree walker's `block`.
//!
//! The code works on the tree walker's own environment. Every binding is
//! pushed with its name at the slot the walker would give it, and every
//! block truncates the environment where the walker's does, so a slot is
//! known when the body is compiled. Any expression or statement outside the
//! compiled subset is handed to the walker (`self.eval`, `self.stmt`) and
//! sees exactly the state it would have seen. Temporaries live in a separate
//! register file. The tree walker stays the oracle (ADR 0032): the tier must
//! not change output, exit codes or trap text, and `--no-bytecode` turns it
//! off (ADR 0361).

use std::borrow::Cow;
use std::cell::Cell;
use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};

use super::*;

static ENABLED: AtomicBool = AtomicBool::new(true);

/// `--no-bytecode`, `--deterministic` and `--schedule` turn the tier off (ADR 0361).
pub fn set_enabled(on: bool) {
    ENABLED.store(on, AtomicOrdering::Relaxed);
}

/// Whether the tier runs for this program: only where nothing it doesn't
/// carry yet is on (unique values, `defer`, contracts). Call and effect
/// traces come from code the walker and the tier share; run limits and
/// coverage take a `Hit` per statement (ADR 0364). `--watch` runs here too:
/// the statements and calls that show a watched value hand off to the walker
/// (ADR 0471).
pub(super) fn usable(prog: &Program, unique: bool, defer: bool) -> bool {
    let contracts = prog.fns.iter().any(|f| !f.contracts.is_empty() || f.inherits.is_some())
        || prog.structs.iter().any(|s| !s.invariants.is_empty());
    ENABLED.load(AtomicOrdering::Relaxed)
        && !unique
        && !defer
        && !(contracts && contracts::enabled())
}

/// Counts of what ran where, printed to standard error at the end of `ail
/// run` when `AIL_BYTECODE_STATS` is set (ADR 0361).
#[derive(Default)]
pub(super) struct Stats {
    /// `AIL_BYTECODE_STATS` is set: instructions are counted too.
    pub on: bool,
    /// Calls run on the walker and on the tier.
    pub calls: [u64; 2],
    /// Instructions the tier ran, hand-offs included.
    pub ins: u64,
    /// Expressions and statements a compiled body handed to the walker.
    pub handoffs: u64,
    /// With `AIL_BYTECODE_STATS=2`: hand-offs by kind of node.
    pub kinds: Option<HashMap<Cow<'static, str>, u64>>,
}

/// The kind of a handed-off node, for `AIL_BYTECODE_STATS=2`.
fn kind_of_expr(e: &Expr) -> Cow<'static, str> {
    Cow::Borrowed(match e {
        Expr::Method { name, .. } => return Cow::Owned(format!("method call .{name}")),
        Expr::Call { .. } => "call",
        Expr::Interp(..) => "interpolation",
        Expr::Match { .. } => "match",
        Expr::Try { .. } => "?",
        Expr::Else { .. } => "else",
        Expr::List(..) => "list literal",
        Expr::Map(..) | Expr::Set(..) => "map or set literal",
        Expr::Comp { .. } => "comprehension",
        Expr::Field { .. } => "field (variant or constant)",
        Expr::Index { .. } => "slice",
        Expr::Name(..) => "name (constant or variant)",
        Expr::If { .. } => "if with else err",
        _ => "other expression",
    })
}

fn kind_of_stmt(s: &Stmt) -> &'static str {
    match s {
        Stmt::Let { .. } => "let with else",
        Stmt::Assign { .. } => "assignment to a field or element",
        Stmt::For { .. } => "for mut or parallel for",
        Stmt::Handle { .. } => "call else",
        _ => "other statement",
    }
}

/// An operand: a local slot (relative to the frame's first parameter), a
/// register, a constant, or nowhere (a result nobody reads).
type Opnd = u32;
const KIND: u32 = 3 << 30;
const LOCAL: u32 = 0;
const REG: u32 = 1 << 30;
const CONST: u32 = 2 << 30;
const NONE: u32 = u32::MAX;

/// What a compiled function's body has tried to become.
pub(super) enum Slot<'a> {
    Untried,
    /// Runs on the tree walker.
    Walker,
    Code(Rc<Code<'a>>),
}

pub(super) struct Code<'a> {
    ins: Vec<Ins<'a>>,
    consts: Vec<Value>,
    nregs: usize,
    /// The receiver and parameters: the slots the frame starts with.
    params: usize,
    exits: Vec<ExitInfo>,
    loops: Vec<LoopInfo>,
    /// `==` on strings is plain: no `Eq` implementation for `str` (trait.res.1).
    str_eq: bool,
}

/// The blocks around a point in the code, outermost first, each as its
/// mark, the position of its last statement and whether it has one; and the
/// innermost loop. A way out of a delegated node (`break`, `continue`,
/// `return`) ends them as the walker's `block` would (conc.task.4).
struct ExitInfo {
    blocks: Vec<(u32, Pos, bool)>,
    lp: Option<u32>,
}

/// A loop: how many of the blocks around a point are outside its body, the
/// depth to return to, and where `break` and `continue` go.
#[derive(Clone, Copy)]
struct LoopInfo {
    outer: usize,
    depth: u32,
    brk: u32,
    cont: u32,
}

enum Ins<'a> {
    /// `name := src`: a new binding at the end of the environment.
    Push {
        name: &'a str,
        src: Opnd,
    },
    Move {
        dst: Opnd,
        src: Opnd,
    },
    /// own.move.2: the value of a local that is moved out.
    MoveOut {
        dst: Opnd,
        slot: u32,
    },
    /// An arithmetic or comparison operator (expr.arith.1-3).
    Bin {
        op: BinOp,
        dst: Opnd,
        a: Opnd,
        b: Opnd,
        pos: Pos,
    },
    /// `Bin` with an int literal on the right.
    BinK {
        op: BinOp,
        dst: Opnd,
        a: Opnd,
        k: i64,
        pos: Pos,
    },
    Not {
        dst: Opnd,
        src: Opnd,
    },
    Neg {
        dst: Opnd,
        src: Opnd,
        pos: Pos,
    },
    /// std.tensor.20: a fused chain over the values of `srcs`, the chain's names.
    Fused {
        dst: Opnd,
        srcs: Box<[Opnd]>,
        prog: std::rc::Rc<[crate::fuse::FOp]>,
    },
    Jump(u32),
    /// Jump to `t` when the `bool` in `src` is `when`.
    JumpIf {
        src: Opnd,
        when: bool,
        t: u32,
    },
    /// Compare and jump to `t` when the result is `when`.
    BrCmp {
        op: BinOp,
        a: Opnd,
        b: Opnd,
        when: bool,
        t: u32,
        pos: Pos,
    },
    BrCmpK {
        op: BinOp,
        a: Opnd,
        k: i64,
        when: bool,
        t: u32,
        pos: Pos,
    },
    /// A call of a function with no `mut` parameters; its arguments are in
    /// the registers from `args`.
    Call {
        f: &'a FnDecl,
        dst: Opnd,
        args: u32,
        pos: Pos,
    },
    Print {
        src: Opnd,
        pos: Pos,
    },
    /// `e?` (err.prop.1-5, err.ctx.2): leave with the failure or `none`, or keep the value.
    Try {
        dst: Opnd,
        src: Opnd,
        context: Option<&'a Expr>,
        absent: bool,
        optional: bool,
        pos: Pos,
        exit: u32,
    },
    /// `e else ...` (err.else.1-6): jump to `t` when `src` is absent or failed,
    /// else keep its content.
    ElseTest {
        src: Opnd,
        t: u32,
    },
    /// Jump to `t` unless the local holds a struct.
    IfNotStruct {
        slot: u32,
        t: u32,
    },
    /// `local.name = val`, or `local.name op= val` (type.struct.6, expr.op.2).
    SetField {
        slot: u32,
        name: &'a str,
        val: Opnd,
        op: Option<BinOp>,
        pos: Pos,
    },
    /// One `match` arm (expr.match.4-8): jump to `t` unless `src` matches,
    /// else bind the variant's fields.
    Arm {
        src: Opnd,
        pat: &'a Pattern,
        t: u32,
    },
    /// A part of an interpolated string: the value in `r` becomes its text
    /// (type.str.3, type.str.11).
    Text {
        r: Opnd,
        spec: Option<&'a crate::lexer::FormatSpec>,
        pos: Pos,
    },
    /// An interpolated string (lex.str.2, lex.str.7) whose parts' texts are
    /// in the registers from `args`, in order.
    Interp {
        dst: Opnd,
        parts: &'a [InterpPart],
        args: u32,
    },
    /// A struct or variant built from the registers from `args`, each into
    /// its field slot (type.struct.3, type.enum.6).
    Build {
        dst: Opnd,
        what: Built,
        slots: Rc<[usize]>,
        args: u32,
    },
    /// `[a, b, ...]` (type.coll.2).
    List {
        dst: Opnd,
        args: u32,
        n: u32,
    },
    /// `local.f.g.name(args)`, or `name(args)` on a register or constant
    /// receiver with no path (trait.method.2): jump to `t`, the walker's
    /// path, unless the receiver is a value whose method the tier calls
    /// (ADR 0363).
    MethodGuard {
        recv: Opnd,
        path: Box<[&'a str]>,
        name: &'a str,
        t: u32,
        /// The last user method it allowed, by the receiver's type.
        cache: Cell<Option<(bool, usize, &'a FnDecl)>>,
    },
    /// The call the `MethodGuard` at `guard` allowed, with the arguments in
    /// the registers from `args`.
    MethodRun {
        recv: Opnd,
        path: Box<[&'a str]>,
        name: &'a str,
        mutating: bool,
        args: u32,
        n: u32,
        dst: Opnd,
        pos: Pos,
        guard: u32,
    },
    /// `recv[idx]` on a list or map (type.coll.4, type.map.3).
    Index {
        dst: Opnd,
        recv: Opnd,
        idx: Opnd,
        pos: Pos,
    },
    /// `recv.name` on a struct, or a `ref` to one (type.struct.6, own.ref.3);
    /// `cache` is the last struct and field index.
    Field {
        dst: Opnd,
        recv: Opnd,
        name: &'a str,
        cache: Cell<(usize, usize)>,
        pos: Pos,
    },
    /// Jump to `t` unless the local holds a list or map.
    IfNotListMap {
        slot: u32,
        t: u32,
    },
    /// `local[idx] = val`, or `local[idx] op= val` (type.coll.4, type.map.3, type.map.7).
    SetIndex {
        slot: u32,
        idx: Opnd,
        val: Opnd,
        op: Option<BinOp>,
        at: Pos,
        pos: Pos,
    },
    /// `{k: v, ...}` from `n` key and value pairs in the registers from
    /// `args`, or `{}` as a set where the checker typed it one (type.coll.2).
    Map {
        dst: Opnd,
        args: u32,
        n: u32,
        set: bool,
    },
    /// `recv.name(args)` through trait `tr` with the receiver in `recv`: jump
    /// to `t` unless the tier can call the implementation `dispatch` picks,
    /// which is left in `now` (`None`: the built-in `eq`, `cmp` or `to_str`).
    /// `cache` keeps the last pick, by the type `dispatch` looks at.
    TraitGuard {
        recv: Opnd,
        tr: usize,
        name: &'a str,
        t: u32,
        cache: Cell<Option<(u8, usize, Option<&'a FnDecl>)>>,
        now: Cell<Option<&'a FnDecl>>,
    },
    /// The call the `TraitGuard` at `guard` allowed.
    TraitRun {
        recv: Opnd,
        tr: usize,
        args: u32,
        n: u32,
        dst: Opnd,
        pos: Pos,
        guard: u32,
    },
    /// `MethodGuard`'s fallback for a receiver already evaluated into
    /// `recv`: the walker's method call from there on (ADR 0363).
    MethodOn {
        recv: Opnd,
        e: &'a Expr,
        dst: Opnd,
        exit: u32,
    },
    /// The walker's `hit` before a statement or a `while` turn, with run
    /// limits or coverage on (run.limit.1-3, test.cover.1).
    Hit(Pos),
    /// An expression run on the tree walker.
    Expr {
        e: &'a Expr,
        dst: Opnd,
        exit: u32,
    },
    /// A statement run on the tree walker.
    Stmt {
        s: &'a Stmt,
        exit: u32,
    },
    /// The end of a block (conc.task.4): its tasks finish, its bindings go.
    EndBlock {
        mark: u32,
        pos: Pos,
    },
    Truncate(u32),
    /// `for name in lo..hi`: stop at `end`, or bind the counter.
    RangeTest {
        name: &'a str,
        i: Opnd,
        hi: Opnd,
        inclusive: bool,
        end: u32,
    },
    RangeNext {
        i: Opnd,
        end: u32,
        top: u32,
    },
    /// `for name in xs` (expr.loop.4): the list, set or channel is in `it`.
    IterStart {
        it: Opnd,
        idx: Opnd,
    },
    IterNext {
        name: &'a str,
        it: Opnd,
        idx: Opnd,
        end: u32,
        pos: Pos,
    },
    /// `if name := src` (expr.if.5): jump to `t` when `src` is absent or failed.
    Bind {
        name: &'a str,
        src: Opnd,
        t: u32,
    },
    /// `break` (when `brk`) or `continue`.
    Leave {
        exit: u32,
        brk: bool,
    },
    Ret {
        src: Opnd,
        pos: Pos,
        exit: u32,
    },
    End {
        src: Opnd,
    },
}

#[inline(always)]
fn get<'e>(o: Opnd, env: &'e Env<'_>, base: usize, regs: &'e [Value], consts: &'e [Value]) -> &'e Value {
    let i = (o & !KIND) as usize;
    match o & KIND {
        LOCAL => &env[base + i].1,
        REG => &regs[i],
        _ => &consts[i],
    }
}

/// The value of an operand: a register's is moved out, others are copied.
#[inline(always)]
fn take(o: Opnd, env: &Env<'_>, base: usize, regs: &mut [Value], consts: &[Value]) -> Value {
    if o & KIND == REG {
        return std::mem::replace(&mut regs[(o & !KIND) as usize], Value::Unit);
    }
    get(o, env, base, regs, consts).clone()
}

#[inline(always)]
fn put(o: Opnd, v: Value, env: &mut Env<'_>, base: usize, regs: &mut [Value]) {
    let i = (o & !KIND) as usize;
    match o & KIND {
        LOCAL => env[base + i].1 = v,
        REG => regs[i] = v,
        _ => {}
    }
}

/// `put` of an int, overwriting an int in place.
#[inline(always)]
fn put_int(o: Opnd, r: i64, env: &mut Env<'_>, base: usize, regs: &mut [Value]) {
    let i = (o & !KIND) as usize;
    let slot = match o & KIND {
        LOCAL => &mut env[base + i].1,
        REG => &mut regs[i],
        _ => return,
    };
    if let Value::Int(x) = slot {
        *x = r;
    } else {
        *slot = Value::Int(r);
    }
}

/// Release a register an instruction read in place, so it holds no share
/// of a list or string that the program may change next (own.val.2).
#[inline(always)]
fn done(o: Opnd, regs: &mut [Value]) {
    if o & KIND == REG {
        regs[(o & !KIND) as usize] = Value::Unit;
    }
}

/// Whether evaluating `e` may change a local's slot: through a `mut` or
/// `move` argument, a method (which may mutate its receiver), a move, or a
/// block. An operand read from a local before such a sibling is copied first.
fn writes_locals(e: &Expr, moves: &crate::fasthash::FastMap<Pos, &'static str>) -> bool {
    writes(e, None, moves)
}

/// The local a place expression starts from.
fn root(e: &Expr) -> Option<&str> {
    match e {
        Expr::Name(n, _) => Some(n),
        Expr::Field { recv, .. } | Expr::Index { recv, .. } => root(recv),
        _ => None,
    }
}

/// Whether evaluating `e` may change the local `t`, or any local when `t`
/// is `None`. Values are independent (own.val.1), so a call changes only
/// its receiver's and its `mut` arguments' locals.
fn writes(e: &Expr, t: Option<&str>, moves: &crate::fasthash::FastMap<Pos, &'static str>) -> bool {
    let w = |e: &Expr| writes(e, t, moves);
    // A place a call may change: a temporary changes no local.
    let hits = |e: &Expr| match t {
        None => true,
        Some(t) => root(e) == Some(t),
    };
    let arg = |a: &Arg| ((a.mutable || a.moved) && hits(&a.value)) || (t.is_none() && a.param.is_some()) || w(&a.value);
    match e {
        Expr::Int(..) | Expr::Float(..) | Expr::Str(..) | Expr::Bytes(..) | Expr::Char(..) | Expr::Bool(..) | Expr::None(_) => false,
        Expr::Name(n, pos) => moves.contains_key(pos) && t.is_none_or(|t| t == n),
        Expr::Binary { lhs, rhs, .. } => w(lhs) || w(rhs),
        Expr::Unary { op: UnOp::Neg | UnOp::Not, operand, .. } => w(operand),
        Expr::Field { recv, .. } => w(recv),
        Expr::Index { recv, index, .. } => w(recv) || w(index),
        Expr::Call { callee, args, .. } => !matches!(**callee, Expr::Name(..)) || args.iter().any(arg),
        Expr::Method { recv, args, .. } => t.is_none() || hits(recv) || w(recv) || args.iter().any(arg),
        Expr::List(items, _) | Expr::Set(items, _) => items.iter().any(w),
        Expr::Map(pairs, _) => pairs.iter().any(|(k, v)| w(k) || w(v)),
        // An early exit leaves before any later read, so only the values count.
        Expr::Try { expr, context, .. } => w(expr) || context.as_deref().is_some_and(w),
        Expr::Else { value, otherwise, .. } => {
            w(value)
                || match &**otherwise {
                    Else::Value(d) => w(d),
                    Else::Leave(s) => match &**s {
                        Stmt::Break(_) | Stmt::Continue(_) | Stmt::Return(None, _) => false,
                        Stmt::Return(Some(x), _) => w(x),
                        _ => true,
                    },
                    Else::Block { .. } => true,
                }
        }
        Expr::Interp(parts, _) => parts.iter().any(|p| match p {
            InterpPart::Lit(_) => false,
            InterpPart::Expr(x) | InterpPart::Fmt(x, _) => w(x),
        }),
        _ => true,
    }
}

/// Whether a function body has a construct the tier leaves to the walker as
/// a whole function: tasks, mutexes and `defer` end their blocks in ways the
/// tier doesn't model (conc.task.4, own.defer.1).
fn compilable(b: &Block) -> bool {
    fn expr(e: &Expr) -> bool {
        match e {
            Expr::Unary { op: UnOp::Spawn, .. } => false,
            Expr::Call { callee, args, .. } => {
                !matches!(&**callee, Expr::Name(n, _) if matches!(n.as_str(), "TaskGroup" | "Channel" | "Mutex"))
                    && args.iter().all(|a| expr(&a.value))
            }
            Expr::If { branches, els, .. } => {
                branches.iter().all(|(c, b)| {
                    let c = match c {
                        Cond::Expr(e) | Cond::Bind { value: e, .. } => e,
                    };
                    expr(c) && compilable(b)
                }) && els.as_ref().is_none_or(|b| compilable(b))
            }
            Expr::Match { scrutinee, arms, .. } => expr(scrutinee) && arms.iter().all(|a| a.guard.as_ref().is_none_or(expr) && compilable(&a.body)),
            Expr::Binary { lhs, rhs, .. } => expr(lhs) && expr(rhs),
            Expr::Unary { operand, .. } => expr(operand),
            Expr::Method { recv, args, .. } => expr(recv) && args.iter().all(|a| expr(&a.value)),
            Expr::Field { recv, .. } => expr(recv),
            Expr::Index { recv, index, .. } => expr(recv) && expr(index),
            Expr::List(items, _) | Expr::Set(items, _) => items.iter().all(expr),
            Expr::Map(pairs, _) => pairs.iter().all(|(k, v)| expr(k) && expr(v)),
            Expr::Try { expr: e, context, .. } => expr(e) && context.as_deref().is_none_or(expr),
            Expr::Else { value, otherwise, .. } => expr(value) && other(otherwise),
            Expr::Interp(parts, _) => parts.iter().all(|p| match p {
                InterpPart::Lit(_) => true,
                InterpPart::Expr(x) | InterpPart::Fmt(x, _) => expr(x),
            }),
            Expr::Comp { elem, value, clauses, .. } => {
                expr(elem)
                    && value.as_deref().is_none_or(expr)
                    && clauses.iter().all(|c| !c.par && expr(&c.iter) && c.cond.as_ref().is_none_or(expr))
            }
            Expr::Ref { value, .. } => expr(value),
            Expr::Range { lo, hi, .. } => lo.as_deref().is_none_or(expr) && hi.as_deref().is_none_or(expr),
            _ => true,
        }
    }
    fn other(o: &Else) -> bool {
        match o {
            Else::Value(e) => expr(e),
            Else::Leave(s) => stmt(s),
            Else::Block { body, .. } => compilable(body),
        }
    }
    fn stmt(s: &Stmt) -> bool {
        match s {
            Stmt::With { .. } | Stmt::Defer(..) | Stmt::Unsafe { .. } => false,
            Stmt::For { par: true, .. } => false,
            Stmt::Let { value, otherwise, .. } => expr(value) && otherwise.as_ref().is_none_or(other),
            Stmt::Assign { target, value, .. } => expr(target) && expr(value),
            Stmt::While { cond, body, .. } => expr(cond) && compilable(body),
            Stmt::For { iter, body, .. } => expr(iter) && compilable(body),
            Stmt::Return(e, _) => e.as_ref().is_none_or(expr),
            Stmt::Expr(e) => expr(e),
            Stmt::Handle { value, otherwise } => expr(value) && other(otherwise),
            Stmt::Break(_) | Stmt::Continue(_) => true,
        }
    }
    b.iter().all(stmt)
}

/// What a `Build` makes: a struct, a variant (shared empty field list when
/// it has none, as the walker's call does), or a variant through its enum.
#[derive(Clone, Copy)]
enum Built {
    Struct(usize),
    Variant(usize, usize),
    Qualified(usize, usize),
}

/// Whether an expression has a `?`: inside `e else ...` one that finds
/// `none` makes the whole value `none` (err.prop.4), which the walker's
/// `else_value` handles.
fn has_try(e: &Expr) -> bool {
    match e {
        Expr::Try { .. } => true,
        Expr::Binary { lhs, rhs, .. } => has_try(lhs) || has_try(rhs),
        Expr::Unary { operand, .. } => has_try(operand),
        Expr::Field { recv, .. } => has_try(recv),
        Expr::Index { recv, index, .. } => has_try(recv) || has_try(index),
        Expr::Call { args, .. } => args.iter().any(|a| has_try(&a.value)),
        Expr::Method { recv, args, .. } => has_try(recv) || args.iter().any(|a| has_try(&a.value)),
        Expr::List(items, _) | Expr::Set(items, _) => items.iter().any(has_try),
        Expr::Interp(parts, _) => parts.iter().any(|p| match p {
            InterpPart::Lit(_) => false,
            InterpPart::Expr(x) | InterpPart::Fmt(x, _) => has_try(x),
        }),
        Expr::Int(..)
        | Expr::Float(..)
        | Expr::Str(..)
        | Expr::Bytes(..)
        | Expr::Char(..)
        | Expr::Bool(..)
        | Expr::None(_)
        | Expr::Name(..) => false,
        _ => true,
    }
}

struct Compiler<'c, 'a, W: Write> {
    it: &'c mut Interp<'a, W>,
    ins: Vec<Ins<'a>>,
    consts: Vec<Value>,
    next_reg: u32,
    nregs: u32,
    /// The name of each slot, innermost last.
    scope: Vec<&'a str>,
    blocks: Vec<(u32, Pos, bool)>,
    loops: Vec<LoopInfo>,
    /// The loops around the current point, innermost last, by index in `loops`.
    loop_stack: Vec<u32>,
    exits: Vec<ExitInfo>,
    /// The exit of the current point, until a block or loop starts or ends.
    exit_cache: Option<u32>,
    /// Instructions that aren't a hand-off to the walker.
    native: usize,
}

impl<'c, 'a, W: Write> Compiler<'c, 'a, W> {
    fn emit(&mut self, i: Ins<'a>) -> u32 {
        if !matches!(
            i,
            Ins::Expr { .. }
                | Ins::Stmt { .. }
                | Ins::EndBlock { .. }
                | Ins::Truncate(_)
                | Ins::End { .. }
                | Ins::Hit(_)
        ) {
            self.native += 1;
        }
        self.ins.push(i);
        (self.ins.len() - 1) as u32
    }

    fn here(&self) -> u32 {
        self.ins.len() as u32
    }

    fn reg(&mut self) -> Opnd {
        let r = self.next_reg;
        self.next_reg += 1;
        self.nregs = self.nregs.max(self.next_reg);
        REG | r
    }

    fn konst(&mut self, v: Value) -> Opnd {
        self.consts.push(v);
        CONST | (self.consts.len() - 1) as u32
    }

    fn depth(&self) -> u32 {
        self.scope.len() as u32
    }

    fn local(&self, name: &str) -> Option<u32> {
        self.scope.iter().rposition(|n| name_eq(n, name)).map(|i| i as u32)
    }

    fn exit(&mut self) -> u32 {
        if let Some(x) = self.exit_cache {
            return x;
        }
        self.exits.push(ExitInfo { blocks: self.blocks.clone(), lp: self.loop_stack.last().copied() });
        let x = (self.exits.len() - 1) as u32;
        self.exit_cache = Some(x);
        x
    }

    /// Point the jump at `at` to `t`.
    fn patch(&mut self, at: u32, t: u32) {
        match &mut self.ins[at as usize] {
            Ins::Jump(x)
            | Ins::JumpIf { t: x, .. }
            | Ins::BrCmp { t: x, .. }
            | Ins::BrCmpK { t: x, .. }
            | Ins::IfNotListMap { t: x, .. }
            | Ins::MethodGuard { t: x, .. }
            | Ins::TraitGuard { t: x, .. }
            | Ins::ElseTest { t: x, .. }
            | Ins::IfNotStruct { t: x, .. }
            | Ins::Arm { t: x, .. }
            | Ins::Bind { t: x, .. }
            | Ins::RangeTest { end: x, .. }
            | Ins::IterNext { end: x, .. } => *x = t,
            _ => unreachable!("not a jump"),
        }
    }

    /// The constant an expression always has, as the walker computes it.
    fn literal(&mut self, e: &'a Expr) -> Option<Opnd> {
        match e {
            Expr::Int(..) | Expr::Float(..) | Expr::Str(..) | Expr::Bytes(..) | Expr::Char(..) | Expr::Bool(..) | Expr::None(_) => {
                let v = self.it.eval(e, &mut Vec::new()).ok()?;
                Some(self.konst(v))
            }
            _ => None,
        }
    }

    /// The int a literal has as an operand of an int operator, when the walker reads it as one.
    fn int_literal(&self, e: &Expr) -> Option<i64> {
        match e {
            Expr::Int(v, pos) if !self.it.lit_maybe(pos) || !self.it.resolved.float_literals.contains(pos) => Some(*v),
            _ => None,
        }
    }

    /// An operand for `e`, without an instruction when it is a local or a constant.
    fn operand(&mut self, e: &'a Expr) -> Opnd {
        if let Expr::Name(n, pos) = e
            && !self.it.resolved.moves.contains_key(pos)
            && let Some(slot) = self.local(n)
        {
            return LOCAL | slot;
        }
        if let Some(k) = self.literal(e) {
            return k;
        }
        let r = self.reg();
        self.expr_to(e, r);
        r
    }

    /// Operands for expressions evaluated left to right: a local read
    /// before a sibling that may change it is copied first.
    fn operands<const N: usize>(&mut self, es: [&'a Expr; N]) -> [Opnd; N] {
        let mut out = [NONE; N];
        for i in 0..N {
            let mut o = self.operand(es[i]);
            if o & KIND == LOCAL && es[i + 1..].iter().any(|e| writes(e, root(es[i]), &self.it.resolved.moves)) {
                let r = self.reg();
                self.emit(Ins::Move { dst: r, src: o });
                o = r;
            }
            out[i] = o;
        }
        out
    }

    fn delegate(&mut self, e: &'a Expr, dst: Opnd) {
        let exit = self.exit();
        self.emit(Ins::Expr { e, dst, exit });
    }

    fn delegate_stmt(&mut self, s: &'a Stmt) {
        let exit = self.exit();
        self.emit(Ins::Stmt { s, exit });
        if let Stmt::Let { name, .. } = s {
            self.scope.push(name);
        }
    }

    /// Code that leaves the value of `e` in `dst`.
    fn expr_to(&mut self, e: &'a Expr, dst: Opnd) {
        // std.tensor.20: a fused chain of tensor operations is one instruction.
        if !self.it.resolved.fused.is_empty()
            && matches!(e, Expr::Binary { .. } | Expr::Unary { .. } | Expr::Method { .. })
            && self.it.resolved.fused.contains(&e.pos())
            && let Some(plan) = crate::fuse::plan(e)
        {
            let srcs: Box<[Opnd]> = plan.names.iter().map(|n| self.operand(n)).collect();
            self.emit(Ins::Fused { dst, srcs, prog: plan.ops.into() });
            return;
        }
        match e {
            Expr::Int(..) | Expr::Float(..) | Expr::Str(..) | Expr::Bytes(..) | Expr::Char(..) | Expr::Bool(..) | Expr::None(_) => {
                match self.literal(e) {
                    Some(k) => {
                        self.emit(Ins::Move { dst, src: k });
                    }
                    None => self.delegate(e, dst),
                }
            }
            Expr::Name(n, pos) => match self.local(n) {
                Some(slot) if self.it.resolved.moves.contains_key(pos) => {
                    self.emit(Ins::MoveOut { dst, slot });
                }
                Some(slot) => {
                    self.emit(Ins::Move { dst, src: LOCAL | slot });
                }
                // mod.const.1, type.enum.3: constants are computed before `main`, and unit variants are fixed.
                None => match self.fixed(n, *pos) {
                    Some(k) => {
                        self.emit(Ins::Move { dst, src: k });
                    }
                    None => self.delegate(e, dst),
                },
            },
            Expr::Binary { op: BinOp::And | BinOp::Or, .. } => {
                let t = if dst & KIND == LOCAL { self.reg() } else { dst };
                let jf = self.cond_jump(e, false);
                let (yes, no) = (self.konst(Value::Bool(true)), self.konst(Value::Bool(false)));
                self.emit(Ins::Move { dst: t, src: yes });
                let j = self.emit(Ins::Jump(0));
                let at = self.here();
                for x in jf {
                    self.patch(x, at);
                }
                self.emit(Ins::Move { dst: t, src: no });
                let end = self.here();
                self.patch(j, end);
                if t != dst {
                    self.emit(Ins::Move { dst, src: t });
                }
            }
            Expr::Binary { op, lhs, rhs, pos } => {
                if let Some(k) = self.int_literal(rhs) {
                    let a = self.operand(lhs);
                    self.emit(Ins::BinK { op: *op, dst, a, k, pos: *pos });
                } else {
                    let [a, b] = self.operands([&**lhs, &**rhs]);
                    self.emit(Ins::Bin { op: *op, dst, a, b, pos: *pos });
                }
            }
            Expr::Unary { op: UnOp::Not, operand, .. } => {
                let src = self.operand(operand);
                self.emit(Ins::Not { dst, src });
            }
            Expr::Unary { op: UnOp::Neg, operand, pos } => {
                let src = self.operand(operand);
                self.emit(Ins::Neg { dst, src, pos: *pos });
            }
            Expr::Call { callee, args, pos } => self.call(e, callee, args, *pos, dst),
            Expr::Method { recv, name, name_pos, args } => self.method(e, recv, name, args, *name_pos, dst),
            Expr::Try { expr, context, pos } if !self.it.resolved.else_try.contains(pos) => {
                let src = self.operand(expr);
                let r = &self.it.resolved;
                let (absent, optional) = (r.absent_try.contains(pos), r.optional_try.contains(pos));
                let exit = self.exit();
                self.emit(Ins::Try { dst, src, context: context.as_deref(), absent, optional, pos: *pos, exit });
            }
            Expr::Else { value, otherwise, .. }
                if !has_try(value) && !matches!(**otherwise, Else::Block { err: Some(_), .. }) =>
            {
                let t = if dst & KIND == REG { dst } else { self.reg() };
                self.else_to(value, otherwise, t);
                if t != dst && dst != NONE {
                    self.emit(Ins::Move { dst, src: t });
                }
            }
            Expr::Match { scrutinee, arms, .. } => {
                let t = if dst & KIND == LOCAL { self.reg() } else { dst };
                let src = self.reg();
                self.expr_to(scrutinee, src);
                let mut ends = Vec::new();
                for arm in arms {
                    let test = self.emit(Ins::Arm { src, pat: &arm.pat, t: 0 });
                    let mark = self.depth();
                    // expr.match.6: fields bind by position; `_` skips one.
                    if let Pattern::Variant { bindings: Some(names), .. } = &arm.pat {
                        self.scope.extend(names.iter().filter(|(n, _)| n != "_").map(|(n, _)| n.as_str()));
                    }
                    // expr.match.11: a false guard drops the bindings and tries the next arm.
                    let fails = arm.guard.as_ref().map(|g| self.cond_jump(g, false));
                    self.block(&arm.body, t);
                    let bound = self.depth() > mark;
                    if bound {
                        self.scope.truncate(mark as usize);
                        self.emit(Ins::Truncate(mark));
                    }
                    ends.push(self.emit(Ins::Jump(0)));
                    if let Some(fails) = fails {
                        let here = self.here();
                        for f in fails {
                            self.patch(f, here);
                        }
                        if bound {
                            self.emit(Ins::Truncate(mark));
                        }
                    }
                    let next = self.here();
                    self.patch(test, next);
                }
                let end = self.here();
                for j in ends {
                    self.patch(j, end);
                }
                let unit = self.konst(Value::Unit);
                self.emit(Ins::Move { dst: src, src: unit });
                if t != dst {
                    self.emit(Ins::Move { dst, src: t });
                }
            }
            Expr::Interp(parts, _) => {
                let first = self.next_reg;
                let exprs: Vec<&'a Expr> = parts
                    .iter()
                    .filter_map(|p| match p {
                        InterpPart::Lit(_) => None,
                        InterpPart::Expr(x) | InterpPart::Fmt(x, _) => Some(x),
                    })
                    .collect();
                let regs: Vec<Opnd> = exprs.iter().map(|_| self.reg()).collect();
                // Each part's text is made right after its value, as the walker does: a `to_str` may run.
                let specs = parts.iter().filter_map(|p| match p {
                    InterpPart::Lit(_) => None,
                    InterpPart::Expr(_) => Some(None),
                    InterpPart::Fmt(_, spec) => Some(Some(spec)),
                });
                for ((x, r), spec) in exprs.into_iter().zip(regs).zip(specs) {
                    self.expr_to(x, r);
                    self.emit(Ins::Text { r, spec, pos: x.pos() });
                }
                self.emit(Ins::Interp { dst, parts, args: first });
            }
            Expr::List(items, _) => {
                let first = self.next_reg;
                let regs: Vec<Opnd> = items.iter().map(|_| self.reg()).collect();
                for (x, r) in items.iter().zip(regs) {
                    self.expr_to(x, r);
                }
                self.emit(Ins::List { dst, args: first, n: items.len() as u32 });
            }
            Expr::Map(pairs, pos) => {
                let first = self.next_reg;
                let regs: Vec<Opnd> = pairs.iter().flat_map(|_| [self.reg(), self.reg()]).collect();
                for ((k, v), r) in pairs.iter().zip(regs.chunks(2)) {
                    self.expr_to(k, r[0]);
                    self.expr_to(v, r[1]);
                }
                let set = pairs.is_empty() && self.it.resolved.empty_sets.contains(pos);
                self.emit(Ins::Map { dst, args: first, n: pairs.len() as u32, set });
            }
            Expr::Index { recv, index, pos } if !matches!(**index, Expr::Range { .. }) => {
                let [recv, idx] = self.operands([&**recv, &**index]);
                self.emit(Ins::Index { dst, recv, idx, pos: *pos });
            }
            Expr::Field { recv, name, name_pos }
                if !self.it.resolved.std_calls.contains_key(name_pos)
                    && let Expr::Name(tn, _) = &**recv
                    && self.local(tn).is_none()
                    && !self.it.prog.consts.iter().any(|c| c.name == *tn)
                    && let Some(&en) = self.it.enums.get(tn.as_str()) =>
            {
                let v = self.it.variant_index(en, name);
                let k = self.konst(Value::Enum(en, v, Rc::clone(&self.it.no_fields)));
                self.emit(Ins::Move { dst, src: k });
            }
            Expr::Field { recv, name, name_pos } if self.plain_field(recv, *name_pos) => {
                let recv = self.operand(recv);
                self.emit(Ins::Field { dst, recv, name, cache: Cell::new((usize::MAX, 0)), pos: *name_pos });
            }
            Expr::If { branches, els, else_err: None, .. } => {
                let t = if dst & KIND == LOCAL { self.reg() } else { dst };
                self.if_chain(branches, els, t);
                if t != dst {
                    self.emit(Ins::Move { dst, src: t });
                }
            }
            _ => self.delegate(e, dst),
        }
    }

    /// The value of a name that isn't a local: a constant, or a unit variant.
    fn fixed(&mut self, n: &str, pos: Pos) -> Option<Opnd> {
        let v = match self.it.consts.get(n) {
            Some(v) => v.clone(),
            None if self.it.prog.consts.iter().any(|c| c.name == n) => return None,
            None => {
                let &(e, v) = self.it.resolved.variants.get(&pos)?;
                Value::Enum(e, v, Rc::clone(&self.it.no_fields))
            }
        };
        Some(self.konst(v))
    }

    /// `e else d`, `e else return ...`, or `e else` and a block, into `dst` (err.else.1-6).
    fn else_to(&mut self, value: &'a Expr, otherwise: &'a Else, dst: Opnd) {
        self.expr_to(value, dst);
        let test = self.emit(Ins::ElseTest { src: dst, t: 0 });
        let j = self.emit(Ins::Jump(0));
        let at = self.here();
        self.patch(test, at);
        match otherwise {
            Else::Value(d) => self.expr_to(d, dst),
            Else::Leave(s) => self.stmt(s, NONE),
            Else::Block { body, .. } => self.block(body, dst),
        }
        let end = self.here();
        self.patch(j, end);
    }

    /// `recv.name` reads a field of a value: not a standard-module constant
    /// or a unit variant named through its enum (std.ui.17, type.enum.3).
    fn plain_field(&self, recv: &Expr, name_pos: Pos) -> bool {
        if self.it.resolved.std_calls.contains_key(&name_pos) {
            return false;
        }
        match recv {
            Expr::Name(n, _) => self.local(n).is_some(),
            Expr::TypeArgs { .. } => false,
            _ => true,
        }
    }

    /// `f(args)`: `print`, or a function with no `mut` parameters; any other
    /// call (constructors, built-ins, `mut` arguments) runs on the walker.
    fn call(&mut self, e: &'a Expr, callee: &'a Expr, args: &'a [Arg], pos: Pos, dst: Opnd) {
        let Expr::Name(n, _) = callee else { return self.delegate(e, dst) };
        if n == "print" {
            let src = self.operand(&args[0].value);
            self.emit(Ins::Print { src, pos });
            if dst != NONE {
                let unit = self.konst(Value::Unit);
                self.emit(Ins::Move { dst, src: unit });
            }
            return;
        }
        let it = &*self.it;
        let special =
            matches!(n.as_str(), "unreachable" | "assert" | "assert_eq" | "Atomic" | "TaskGroup" | "Channel" | "Mutex");
        // type.struct.3, type.enum.6: a struct or variant built from its arguments.
        if !special && self.local(n).is_none() && args.iter().all(|a| a.param.is_none() && !a.mutable) {
            let built = if let Some(&st) = it.structs.get(n.as_str()) {
                let has_inv = !it.prog.structs[st].invariants.is_empty();
                (!has_inv).then(|| (Built::Struct(st), &it.prog.structs[st].fields))
            } else {
                it.resolved
                    .variants
                    .get(&callee.pos())
                    .map(|&(en, v)| (Built::Variant(en, v), &it.prog.enums[en].variants[v].fields))
            };
            if let Some((what, fields)) = built {
                return self.build(what, fields, args, dst);
            }
        }
        let plain = !matches!(
            n.as_str(),
            "unreachable" | "assert" | "assert_eq" | "Atomic" | "TaskGroup" | "Channel" | "Mutex"
        ) && self.local(n).is_none()
            && !it.structs.contains_key(n.as_str())
            && !it.resolved.variants.contains_key(&callee.pos())
            && !it.resolved.type_args.contains_key(&callee.pos());
        let f = match it.fns.get(n.as_str()) {
            Some(&f) if plain => f,
            _ => return self.delegate(e, dst),
        };
        if f.receiver.is_some()
            || f.params.iter().any(|p| p.mutable)
            || f.params.len() != args.len()
            || args.iter().any(|a| a.label.is_some() || a.param.is_some() || a.mutable)
        {
            return self.delegate(e, dst);
        }
        let first = self.next_reg;
        let regs: Vec<Opnd> = args.iter().map(|_| self.reg()).collect();
        for (a, r) in args.iter().zip(regs) {
            self.expr_to(&a.value, r);
        }
        self.emit(Ins::Call { f, dst, args: first, pos: callee.pos() });
    }

    fn build(&mut self, what: Built, fields: &'a [crate::ast::Field], args: &'a [Arg], dst: Opnd) {
        let slots: Rc<[usize]> = crate::ast::field_slots(fields, args)
            .into_iter()
            .map(|(slot, _)| slot.expect("checked: field exists"))
            .collect();
        let first = self.next_reg;
        let regs: Vec<Opnd> = args.iter().map(|_| self.reg()).collect();
        for (a, r) in args.iter().zip(regs) {
            self.expr_to(&a.value, r);
        }
        self.emit(Ins::Build { dst, what, slots, args: first });
    }

    /// Jumps taken when `e` is `when`, to be patched to their target.
    fn cond_jump(&mut self, e: &'a Expr, when: bool) -> Vec<u32> {
        match e {
            Expr::Binary {
                op: op @ (BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge),
                lhs,
                rhs,
                pos,
            } => {
                if let Some(k) = self.int_literal(rhs) {
                    let a = self.operand(lhs);
                    vec![self.emit(Ins::BrCmpK { op: *op, a, k, when, t: 0, pos: *pos })]
                } else {
                    let [a, b] = self.operands([&**lhs, &**rhs]);
                    vec![self.emit(Ins::BrCmp { op: *op, a, b, when, t: 0, pos: *pos })]
                }
            }
            // `a and b` is false when either is; true when both are.
            Expr::Binary { op: BinOp::And, lhs, rhs, .. } => {
                if !when {
                    let mut j = self.cond_jump(lhs, false);
                    j.extend(self.cond_jump(rhs, false));
                    j
                } else {
                    let skip = self.cond_jump(lhs, false);
                    let j = self.cond_jump(rhs, true);
                    let at = self.here();
                    for x in skip {
                        self.patch(x, at);
                    }
                    j
                }
            }
            Expr::Binary { op: BinOp::Or, lhs, rhs, .. } => {
                if when {
                    let mut j = self.cond_jump(lhs, true);
                    j.extend(self.cond_jump(rhs, true));
                    j
                } else {
                    let skip = self.cond_jump(lhs, true);
                    let j = self.cond_jump(rhs, false);
                    let at = self.here();
                    for x in skip {
                        self.patch(x, at);
                    }
                    j
                }
            }
            Expr::Unary { op: UnOp::Not, operand, .. } => self.cond_jump(operand, !when),
            _ => {
                let src = self.operand(e);
                vec![self.emit(Ins::JumpIf { src, when, t: 0 })]
            }
        }
    }

    /// `if`/`elif`/`else` (expr.if.1-5), its value in `dst`.
    fn if_chain(&mut self, branches: &'a [(Cond, Block)], els: &'a Option<Block>, dst: Opnd) {
        let mut ends = Vec::new();
        for (cond, body) in branches {
            let next = match cond {
                Cond::Expr(c) => {
                    let next = self.cond_jump(c, false);
                    self.block(body, dst);
                    next
                }
                // expr.if.5: the branch runs with the value bound, when there is one.
                Cond::Bind { name, value, .. } => {
                    let src = self.operand(value);
                    let b = self.emit(Ins::Bind { name, src, t: 0 });
                    let mark = self.depth();
                    self.scope.push(name);
                    self.block(body, dst);
                    self.scope.truncate(mark as usize);
                    self.emit(Ins::Truncate(mark));
                    vec![b]
                }
            };
            ends.push(self.emit(Ins::Jump(0)));
            let at = self.here();
            for x in next {
                self.patch(x, at);
            }
        }
        match els {
            Some(body) => self.block(body, dst),
            None if dst != NONE => {
                let unit = self.konst(Value::Unit);
                self.emit(Ins::Move { dst, src: unit });
            }
            None => {}
        }
        let end = self.here();
        for x in ends {
            self.patch(x, end);
        }
    }

    /// A block: its statements, the last one's value in `dst` (Unit when it
    /// has none), then its end (conc.task.4).
    fn block(&mut self, stmts: &'a Block, dst: Opnd) {
        let mark = self.depth();
        let last = stmts.last().map(Stmt::pos);
        self.blocks.push((mark, last.unwrap_or(Pos { line: 0, col: 0 }), last.is_some()));
        self.exit_cache = None;
        for (i, s) in stmts.iter().enumerate() {
            let value = if i + 1 == stmts.len() { dst } else { NONE };
            self.stmt(s, value);
        }
        if dst != NONE && !matches!(stmts.last(), Some(Stmt::Expr(_))) {
            let unit = self.konst(Value::Unit);
            self.emit(Ins::Move { dst, src: unit });
        }
        self.blocks.pop();
        self.exit_cache = None;
        // A block that bound nothing has no tasks to finish and nothing to drop.
        if self.depth() > mark {
            self.scope.truncate(mark as usize);
            match last {
                Some(pos) => self.emit(Ins::EndBlock { mark, pos }),
                None => self.emit(Ins::Truncate(mark)),
            };
        }
    }

    /// Start a loop at depth `depth`; `cont` is patched once known.
    fn loop_start(&mut self, depth: u32) -> u32 {
        self.loops.push(LoopInfo { outer: self.blocks.len(), depth, brk: 0, cont: 0 });
        let l = (self.loops.len() - 1) as u32;
        self.loop_stack.push(l);
        self.exit_cache = None;
        l
    }

    fn loop_end(&mut self) {
        self.loop_stack.pop();
        self.exit_cache = None;
    }

    fn stmt(&mut self, s: &'a Stmt, dst: Opnd) {
        let mark = self.next_reg;
        // run.limit.3, test.cover.1: a step, as the walker's `stmt` takes one.
        if self.it.hook {
            self.emit(Ins::Hit(s.pos()));
        }
        match s {
            // trace.watch.1: the walker shows each assignment and `mut` binding under `--watch` (ADR 0471).
            Stmt::Assign { .. } | Stmt::Let { mutable: true, .. } if self.it.tracing & trace::WATCH != 0 => {
                self.delegate_stmt(s)
            }
            Stmt::Let { name, value, otherwise: Some(o), .. }
                if !has_try(value) && !matches!(o, Else::Block { err: Some(_), .. }) =>
            {
                let r = self.reg();
                self.else_to(value, o, r);
                self.emit(Ins::Push { name, src: r });
                self.scope.push(name);
            }
            Stmt::Assign { target: Expr::Field { recv, name, .. }, op, value, pos }
                if let Expr::Name(n, _) = &**recv
                    && let Some(slot) = self.local(n)
                    && (op.is_none() || self.effect_free(value)) =>
            {
                // The walker's path for a local struct; any other value takes its general path.
                let guard = self.emit(Ins::IfNotStruct { slot, t: 0 });
                let val = self.operand(value);
                self.emit(Ins::SetField { slot, name, val, op: *op, pos: *pos });
                let j = self.emit(Ins::Jump(0));
                let general = self.here();
                self.patch(guard, general);
                let exit = self.exit();
                self.emit(Ins::Stmt { s, exit });
                let end = self.here();
                self.patch(j, end);
            }
            Stmt::Let { name, value, otherwise: None, .. } => {
                let src = self.operand(value);
                self.emit(Ins::Push { name, src });
                self.scope.push(name);
            }
            Stmt::Assign { target: Expr::Name(n, _), op, value, pos } if self.local(n).is_some() => {
                let slot = LOCAL | self.local(n).expect("a local");
                match op {
                    None => self.expr_to(value, slot),
                    Some(op) => {
                        // expr.op.2: the old value is read before the right side runs.
                        let old = if writes_locals(value, &self.it.resolved.moves) {
                            let r = self.reg();
                            self.emit(Ins::Move { dst: r, src: slot });
                            r
                        } else {
                            slot
                        };
                        match self.int_literal(value) {
                            Some(k) => self.emit(Ins::BinK { op: *op, dst: slot, a: old, k, pos: *pos }),
                            None => {
                                let b = self.operand(value);
                                self.emit(Ins::Bin { op: *op, dst: slot, a: old, b, pos: *pos })
                            }
                        };
                    }
                }
            }
            Stmt::Assign { target: Expr::Index { recv, index, pos: at }, op, value, pos }
                if !matches!(**index, Expr::Range { .. })
                    && let Expr::Name(n, _) = &**recv
                    && let Some(slot) = self.local(n)
                    && (op.is_none() || self.effect_free(value)) =>
            {
                // The walker's path for a local list or map; any other value takes its general path.
                let guard = self.emit(Ins::IfNotListMap { slot, t: 0 });
                let idx = self.operand(index);
                let idx = if idx & KIND == LOCAL && writes_locals(value, &self.it.resolved.moves) {
                    let r = self.reg();
                    self.emit(Ins::Move { dst: r, src: idx });
                    r
                } else {
                    idx
                };
                let val = self.operand(value);
                self.emit(Ins::SetIndex { slot, idx, val, op: *op, at: *at, pos: *pos });
                let j = self.emit(Ins::Jump(0));
                let general = self.here();
                self.patch(guard, general);
                let exit = self.exit();
                self.emit(Ins::Stmt { s, exit });
                let end = self.here();
                self.patch(j, end);
            }
            Stmt::While { cond, body, .. } => {
                let top = self.here();
                let depth = self.depth();
                let l = self.loop_start(depth);
                self.loops[l as usize].cont = top;
                let out = self.cond_jump(cond, false);
                if self.it.hook {
                    // A turn of a `while` is a step too.
                    self.emit(Ins::Hit(s.pos()));
                }
                self.block(body, NONE);
                self.emit(Ins::Jump(top));
                self.loop_end();
                let end = self.here();
                self.loops[l as usize].brk = end;
                for x in out {
                    self.patch(x, end);
                }
            }
            Stmt::For { par: false, var, mutable: false, iter, body, pos, .. }
                if !self.it.par.as_ref().is_some_and(|p| p.contains_key(pos)) =>
            {
                let depth = self.depth();
                if let Expr::Range { lo: Some(lo), hi: Some(hi), inclusive, .. } = iter {
                    let (i, h) = (self.reg(), self.reg());
                    self.expr_to(lo, i);
                    self.expr_to(hi, h);
                    let top = self.emit(Ins::RangeTest { name: var, i, hi: h, inclusive: *inclusive, end: 0 });
                    let l = self.loop_start(depth);
                    self.scope.push(var);
                    self.block(body, NONE);
                    self.scope.truncate(depth as usize);
                    self.emit(Ins::Truncate(depth));
                    let next = self.emit(Ins::RangeNext { i, end: 0, top });
                    self.loop_end();
                    let end = self.here();
                    self.patch(top, end);
                    if let Ins::RangeNext { end: e, .. } = &mut self.ins[next as usize] {
                        *e = end;
                    }
                    self.loops[l as usize].cont = next;
                    self.loops[l as usize].brk = end;
                } else {
                    let (it, idx) = (self.reg(), self.reg());
                    self.expr_to(iter, it);
                    self.emit(Ins::IterStart { it, idx });
                    let top = self.emit(Ins::IterNext { name: var, it, idx, end: 0, pos: iter.pos() });
                    let l = self.loop_start(depth);
                    self.scope.push(var);
                    self.block(body, NONE);
                    self.scope.truncate(depth as usize);
                    self.emit(Ins::Truncate(depth));
                    self.emit(Ins::Jump(top));
                    self.loop_end();
                    let end = self.here();
                    self.patch(top, end);
                    let unit = self.konst(Value::Unit);
                    self.emit(Ins::Move { dst: it, src: unit });
                    self.loops[l as usize].cont = top;
                    self.loops[l as usize].brk = end;
                }
            }
            Stmt::Break(_) | Stmt::Continue(_) => {
                let exit = self.exit();
                self.emit(Ins::Leave { exit, brk: matches!(s, Stmt::Break(_)) });
            }
            Stmt::Return(value, pos) => {
                let src = match value {
                    Some(e) => self.operand(e),
                    None => self.konst(Value::Unit),
                };
                let exit = self.exit();
                self.emit(Ins::Ret { src, pos: *pos, exit });
            }
            Stmt::Expr(e) => self.expr_to(e, dst),
            _ => self.delegate_stmt(s),
        }
        self.next_reg = mark;
    }

    /// An expression whose evaluation has no effect and can't trap, so it
    /// may run before a read the walker does first.
    /// `recv.name(args)` where `recv` is a local or a chain of its fields:
    /// a built-in method, or a user method without `mut` parameters, decided
    /// at run time; trait, associated, standard-module and generic calls,
    /// and receivers in a `ref` cell or a handle, take the walker's path.
    fn method(&mut self, e: &'a Expr, recv: &'a Expr, name: &'a str, args: &'a [Arg], pos: Pos, dst: Opnd) {
        // trace.watch.3: a mutating method may change the watched value through `self`; the walker's `invoke` shows it.
        if self.it.tracing & trace::WATCH != 0 && self.it.mutating.contains(name) {
            return self.delegate(e, dst);
        }
        let r = &self.it.resolved;
        let mut path = Vec::new();
        let mut at = recv;
        while let Expr::Field { recv, name, name_pos } = at {
            if r.std_calls.contains_key(name_pos) {
                return self.delegate(e, dst);
            }
            path.push(name.as_str());
            at = recv;
        }
        path.reverse();
        if path.is_empty()
            && let Expr::Name(tn, _) = at
            && self.local(tn).is_none()
            && !r.std_calls.contains_key(&pos)
            && !self.it.prog.consts.iter().any(|c| c.name == *tn)
            && let Some(&en) = self.it.enums.get(tn.as_str())
            && let Some(v) = self.it.prog.enums[en].variants.iter().position(|v| v.name == name)
            && args.iter().all(|a| a.param.is_none() && !a.mutable)
        {
            let fields = &self.it.prog.enums[en].variants[v].fields;
            return self.build(Built::Qualified(en, v), fields, args, dst);
        }
        let slot = match at {
            Expr::Name(n, p) if !r.moves.contains_key(p) => self.local(n),
            _ => None,
        };
        // Any other receiver of a method that doesn't mutate it is evaluated
        // here, and the guard's fallback hands the walker its value.
        let mutating = self.it.mutating.contains(name);
        let chosen = match slot {
            Some(slot) => Some(LOCAL | slot),
            None if mutating || matches!(at, Expr::TypeArgs { .. }) => None,
            None => Some(NONE),
        };
        let Some(mut recv_op) = chosen else { return self.delegate(e, dst) };
        let plain = |a: &Arg| a.label.is_none() && a.param.is_none() && !a.mutable && !a.moved;
        if let Some(&Call::Trait(tr)) = r.calls.get(&pos)
            && !mutating
            && !r.assoc_calls.contains_key(&pos)
            && !r.std_calls.contains_key(&pos)
            && !r.type_args.contains_key(&pos)
            && args.iter().all(plain)
            && (slot.is_some() || !matches!(at, Expr::Name(..)))
        {
            return self.trait_call(e, recv, tr, name, args, pos, dst);
        }
        if r.calls.contains_key(&pos)
            || r.assoc_calls.contains_key(&pos)
            || r.std_calls.contains_key(&pos)
            || r.type_args.contains_key(&pos)
            || args.iter().any(|a| a.label.is_some() || a.param.is_some() || a.mutable || a.moved)
            || (slot.is_some() && args.iter().any(|a| writes(&a.value, root(recv), &r.moves)))
        {
            return self.delegate(e, dst);
        }
        let path: Box<[&'a str]> = if recv_op == NONE {
            recv_op = match recv {
                Expr::Name(n, p) if self.local(n).is_none() => match self.fixed(n, *p) {
                    Some(k) => k,
                    None => return self.delegate(e, dst),
                },
                _ => {
                    let t = self.reg();
                    self.expr_to(recv, t);
                    t
                }
            };
            Box::new([])
        } else {
            path.into()
        };
        let guard =
            self.emit(Ins::MethodGuard { recv: recv_op, path: path.clone(), name, t: 0, cache: Cell::new(None) });
        let first = self.next_reg;
        let regs: Vec<Opnd> = args.iter().map(|_| self.reg()).collect();
        for (a, r) in args.iter().zip(regs) {
            self.expr_to(&a.value, r);
        }
        let n = args.len() as u32;
        self.emit(Ins::MethodRun { recv: recv_op, path, name, mutating, args: first, n, dst, pos, guard });
        let j = self.emit(Ins::Jump(0));
        let general = self.here();
        self.patch(guard, general);
        if recv_op & KIND == LOCAL {
            self.delegate(e, dst);
        } else {
            let exit = self.exit();
            self.emit(Ins::MethodOn { recv: recv_op, e, dst, exit });
        }
        let end = self.here();
        self.patch(j, end);
    }

    /// `recv.name(args)` through a trait (trait.res.1): the receiver is
    /// evaluated first, as the walker does, then the guard picks the
    /// implementation the walker's `dispatch` picks (ADR 0363).
    #[allow(clippy::too_many_arguments)]
    fn trait_call(
        &mut self,
        e: &'a Expr,
        recv: &'a Expr,
        tr: usize,
        name: &'a str,
        args: &'a [Arg],
        pos: Pos,
        dst: Opnd,
    ) {
        let t = self.reg();
        self.expr_to(recv, t);
        let guard =
            self.emit(Ins::TraitGuard { recv: t, tr, name, t: 0, cache: Cell::new(None), now: Cell::new(None) });
        let first = self.next_reg;
        let regs: Vec<Opnd> = args.iter().map(|_| self.reg()).collect();
        for (a, r) in args.iter().zip(regs) {
            self.expr_to(&a.value, r);
        }
        self.emit(Ins::TraitRun { recv: t, tr, args: first, n: args.len() as u32, dst, pos, guard });
        let j = self.emit(Ins::Jump(0));
        let general = self.here();
        self.patch(guard, general);
        let exit = self.exit();
        self.emit(Ins::MethodOn { recv: t, e, dst, exit });
        let end = self.here();
        self.patch(j, end);
    }

    fn effect_free(&self, e: &Expr) -> bool {
        match e {
            Expr::Int(..) | Expr::Float(..) | Expr::Str(..) | Expr::Bytes(..) | Expr::Char(..) | Expr::Bool(..) | Expr::None(_) => true,
            Expr::Name(n, pos) => self.local(n).is_some() && !self.it.resolved.moves.contains_key(pos),
            _ => false,
        }
    }
}

impl<'a, W: Write> Interp<'a, W> {
    /// The compiled body of `f`, compiling it on its first call.
    #[inline]
    pub(super) fn compiled(&mut self, f: &'a FnDecl) -> Option<Rc<Code<'a>>> {
        if !self.tier {
            return None;
        }
        let i =
            (f as *const FnDecl as usize).wrapping_sub(self.prog.fns.as_ptr() as usize) / std::mem::size_of::<FnDecl>();
        match self.codes.get(i) {
            Some(Slot::Code(c)) => Some(Rc::clone(c)),
            Some(Slot::Walker) => None,
            Some(Slot::Untried) if std::ptr::eq(&self.prog.fns[i], f) => {
                let slot = self.compile(f);
                let code = match &slot {
                    Slot::Code(c) => Some(Rc::clone(c)),
                    _ => None,
                };
                self.codes[i] = slot;
                code
            }
            _ => None,
        }
    }

    fn compile(&mut self, f: &'a FnDecl) -> Slot<'a> {
        let Some(body) = f.body.as_ref() else { return Slot::Walker };
        if !compilable(body) {
            return Slot::Walker;
        }
        let mut scope: Vec<&'a str> = Vec::new();
        if f.receiver.is_some() {
            scope.push("self");
        }
        scope.extend(f.params.iter().map(|p| p.name.as_str()));
        let params = scope.len();
        let mut c = Compiler {
            it: self,
            ins: Vec::new(),
            consts: Vec::new(),
            next_reg: 0,
            nregs: 0,
            scope,
            blocks: Vec::new(),
            loops: Vec::new(),
            loop_stack: Vec::new(),
            exits: Vec::new(),
            exit_cache: None,
            native: 0,
        };
        let result = c.reg();
        c.block(body, result);
        c.emit(Ins::End { src: result });
        if c.native == 0 {
            return Slot::Walker;
        }
        let (ins, consts, nregs, exits, loops) = (c.ins, c.consts, c.nregs as usize, c.exits, c.loops);
        let str_eq = self.dispatch(self.resolved.std.eq, "eq", &Value::Str(Rc::from(""))).is_none();
        let code = Code { ins, consts, nregs, params, exits, loops, str_eq };
        Slot::Code(Rc::new(code))
    }

    /// `AIL_BYTECODE_STATS=1`: how much of the run the tier carried.
    pub(super) fn report_stats(&self) {
        if std::env::var_os("AIL_BYTECODE_STATS").is_none() {
            return;
        }
        let compiled = self.codes.iter().filter(|c| matches!(c, Slot::Code(_))).count();
        let tried = self.codes.iter().filter(|c| !matches!(c, Slot::Untried)).count();
        let [walker, tier] = self.stats.calls;
        let (ins, handoffs) = (self.stats.ins, self.stats.handoffs);
        let native = if ins == 0 { 0.0 } else { 100.0 * (ins - handoffs) as f64 / ins as f64 };
        eprintln!(
            "bytecode: {compiled} of {tried} functions called compiled; calls {tier} on the tier, {walker} on the walker; \
             {ins} instructions, {handoffs} hand-offs ({native:.1}% native)"
        );
        let mut kinds: Vec<_> = self.stats.kinds.iter().flatten().collect();
        kinds.sort_by(|a, b| b.1.cmp(a.1));
        for (k, n) in kinds {
            eprintln!("  {n:>9} {k}");
        }
    }

    /// Run a compiled body in `env`, which holds its receiver and
    /// parameters, as `block` runs the body: on return `env` holds just those.
    pub(super) fn run_code(&mut self, code: &Code<'a>, env: &mut Env<'a>) -> Flow<Value> {
        let entry = env.len();
        let base = entry - code.params;
        let mut regs = self.reg_pool.pop().unwrap_or_default();
        regs.resize(code.nregs, Value::Unit);
        let result = if self.stats.on {
            self.exec::<true>(code, env, base, &mut regs)
        } else {
            self.exec::<false>(code, env, base, &mut regs)
        };
        // trace.snap.1: a trap keeps the frame's locals as `block` does.
        if matches!(result, Err(Exit::Trap(_))) && self.snap.is_some() {
            self.snap_block(env);
        }
        env.truncate(entry);
        regs.clear();
        self.reg_pool.push(regs);
        result
    }

    /// The end of a block (conc.task.4): its tasks and groups finish, then its bindings go.
    #[inline(always)]
    fn end_block(&mut self, env: &mut Env<'a>, at: usize, pos: Pos) -> Result<(), Trap> {
        if self.conc.is_some() {
            let r = self.end_tasks(&env[at..], pos);
            env.truncate(at);
            return r;
        }
        env.truncate(at);
        Ok(())
    }

    /// Leave the blocks around a delegated node or a `break`, `continue` or
    /// `return` as the walker's blocks would; the new `pc` for a loop exit.
    #[inline(never)]
    fn leave(&mut self, code: &Code<'a>, exit: u32, how: Exit, env: &mut Env<'a>, base: usize) -> Flow<usize> {
        let info = &code.exits[exit as usize];
        match how {
            Exit::Trap(t) => Err(Exit::Trap(t)),
            Exit::Return => {
                for &(mark, pos, has_last) in info.blocks.iter().rev() {
                    if has_last {
                        self.end_block(env, base + mark as usize, pos)?;
                    } else {
                        env.truncate(base + mark as usize);
                    }
                }
                Err(Exit::Return)
            }
            Exit::Break | Exit::Continue => {
                let l = code.loops[info.lp.expect("checked: `break` and `continue` are inside loops") as usize];
                for &(mark, pos, has_last) in info.blocks[l.outer..].iter().rev() {
                    if has_last {
                        self.end_block(env, base + mark as usize, pos)?;
                    } else {
                        env.truncate(base + mark as usize);
                    }
                }
                env.truncate(base + l.depth as usize);
                Ok(if matches!(how, Exit::Break) { l.brk } else { l.cont } as usize)
            }
        }
    }

    /// The value at a local and a chain of struct fields, unless a step
    /// isn't a struct (a `ref` cell takes the walker's path).
    fn recv_at<'e>(&self, env: &'e Env<'a>, at: usize, path: &[&str]) -> Option<&'e Value> {
        let mut v = &env[at].1;
        for name in path {
            let Value::Struct(s, fields) = v else { return None };
            v = &fields[self.field_index(*s, name)];
        }
        Some(v)
    }

    /// The user method a receiver of this type calls, when the tier can run it.
    fn user_method(&self, is_enum: bool, index: usize, name: &str, path_len: usize) -> Option<&'a FnDecl> {
        let prog = self.prog;
        let type_name = if is_enum { &prog.enums[index].name } else { &prog.structs[index].name };
        let f = *self.methods.get(type_name.as_str())?.get(name)?;
        let ok = f.params.iter().all(|p| !p.mutable) && (path_len == 0 || f.receiver.is_none_or(|r| !r.mutable));
        ok.then_some(f)
    }

    /// Whether `MethodRun` can make this call (ADR 0363), keeping the user
    /// method it calls in `cache`. A receiver that isn't a local's place
    /// can't take `mut self`, so it counts as a path.
    #[inline]
    fn method_allowed(
        &self,
        recv: Option<&Value>,
        name: &str,
        path_len: usize,
        cache: &Cell<Option<(bool, usize, &'a FnDecl)>>,
    ) -> bool {
        let (is_enum, index) = match recv {
            None | Some(Value::Ref(_) | Value::Handle(_) | Value::Atomic(_)) => return false,
            Some(Value::Struct(s, _)) => (false, *s),
            Some(Value::Enum(e, _, _)) => (true, *e),
            Some(_) => return true,
        };
        if let Some((e, i, _)) = cache.get()
            && e == is_enum
            && i == index
        {
            return true;
        }
        match self.user_method(is_enum, index, name, path_len) {
            Some(f) => {
                cache.set(Some((is_enum, index, f)));
                true
            }
            None => false,
        }
    }

    fn exec<const COUNT: bool>(
        &mut self,
        code: &Code<'a>,
        env: &mut Env<'a>,
        base: usize,
        regs: &mut [Value],
    ) -> Flow<Value> {
        let consts = &code.consts[..];
        let mut pc = 0usize;
        macro_rules! get {
            ($o:expr) => {
                get($o, env, base, regs, consts)
            };
        }
        macro_rules! take {
            ($o:expr) => {
                take($o, env, base, regs, consts)
            };
        }
        loop {
            if COUNT {
                self.stats.ins += 1;
            }
            match &code.ins[pc] {
                Ins::BinK { op, dst, a, k, pos } => {
                    pc += 1;
                    if let Value::Int(x) = get!(*a)
                        && let Some(r) = int_arith(*op, *x, *k)
                    {
                        put_int(*dst, r, env, base, regs);
                        continue;
                    }
                    if let Value::Int(x) = get!(*a)
                        && let Some(t) = int_cmp(*op, *x, *k)
                    {
                        put(*dst, Value::Bool(t), env, base, regs);
                        continue;
                    }
                    let x = take!(*a);
                    let v = self.binop(*op, x, Value::Int(*k), *pos)?;
                    put(*dst, v, env, base, regs);
                }
                Ins::Bin { op, dst, a, b, pos } => {
                    pc += 1;
                    if let (Value::Int(x), Value::Int(y)) = (get!(*a), get!(*b))
                        && let Some(v) = int_op(*op, *x, *y)
                    {
                        match v {
                            Value::Int(r) => put_int(*dst, r, env, base, regs),
                            v => put(*dst, v, env, base, regs),
                        }
                        continue;
                    }
                    if let (Value::Str(x), Value::Str(y)) = (get!(*a), get!(*b))
                        && code.str_eq
                        && matches!(op, BinOp::Eq | BinOp::Ne)
                    {
                        let v = (x == y) == (*op == BinOp::Eq);
                        put(*dst, Value::Bool(v), env, base, regs);
                        continue;
                    }
                    let x = take!(*a);
                    let y = take!(*b);
                    let v = self.binop(*op, x, y, *pos)?;
                    put(*dst, v, env, base, regs);
                }
                Ins::BrCmpK { op, a, k, when, t, pos } => {
                    let c = match get!(*a) {
                        Value::Int(x) => int_cmp(*op, *x, *k).expect("a comparison"),
                        _ => {
                            let x = take!(*a);
                            self.binop(*op, x, Value::Int(*k), *pos)?.bool()
                        }
                    };
                    pc = if c == *when { *t as usize } else { pc + 1 };
                }
                Ins::BrCmp { op, a, b, when, t, pos } => {
                    let c = match (get!(*a), get!(*b)) {
                        (Value::Int(x), Value::Int(y)) => int_cmp(*op, *x, *y).expect("a comparison"),
                        // expr.cmp.1: strings are equal when their bytes are.
                        (Value::Str(x), Value::Str(y)) if code.str_eq && matches!(op, BinOp::Eq | BinOp::Ne) => {
                            (x == y) == (*op == BinOp::Eq)
                        }
                        _ => {
                            let x = take!(*a);
                            let y = take!(*b);
                            self.binop(*op, x, y, *pos)?.bool()
                        }
                    };
                    pc = if c == *when { *t as usize } else { pc + 1 };
                }
                Ins::JumpIf { src, when, t } => {
                    let c = get!(*src).bool();
                    pc = if c == *when { *t as usize } else { pc + 1 };
                }
                Ins::Jump(t) => pc = *t as usize,
                Ins::Move { dst, src } => {
                    pc += 1;
                    let v = take!(*src);
                    put(*dst, v, env, base, regs);
                }
                Ins::Push { name, src } => {
                    pc += 1;
                    let v = take!(*src);
                    env.push((name, v));
                }
                Ins::MoveOut { dst, slot } => {
                    pc += 1;
                    let v = std::mem::replace(&mut env[base + *slot as usize].1, Value::Unit);
                    put(*dst, v, env, base, regs);
                }
                Ins::Not { dst, src } => {
                    pc += 1;
                    let v = Value::Bool(!get!(*src).bool());
                    put(*dst, v, env, base, regs);
                }
                Ins::Fused { dst, srcs, prog } => {
                    pc += 1;
                    let vals: Vec<Value> = srcs.iter().map(|s| get!(*s).clone()).collect();
                    for s in srcs.iter() {
                        done(*s, regs);
                    }
                    let v = self.fused_values(prog, &vals)?;
                    put(*dst, v, env, base, regs);
                }
                Ins::Neg { dst, src, pos } => {
                    pc += 1;
                    let v = match take!(*src) {
                        Value::Float(x) => Value::Float(-x),
                        Value::Num(k, bits) => negate(k, bits, *pos)?,
                        // std.tensor.14: elementwise on a tensor.
                        Value::Tensor(t) => match t.neg() {
                            Ok(r) => Value::Tensor(std::rc::Rc::new(r)),
                            Err(f) => return Err(Trap { rule: f.rule, pos: *pos, message: f.message }.into()),
                        },
                        v => {
                            let v = v.int();
                            match v.checked_neg() {
                                Some(r) => Value::Int(r),
                                None => {
                                    let message = format!("`-({v})` overflows `int`");
                                    return Err(Trap { rule: "expr.arith.1", pos: *pos, message }.into());
                                }
                            }
                        }
                    };
                    put(*dst, v, env, base, regs);
                }
                Ins::Call { f, dst, args, pos } => {
                    pc += 1;
                    let mut callee = self.env_pool.pop().unwrap_or_else(|| Vec::with_capacity(16));
                    for (k, p) in f.params.iter().enumerate() {
                        let v = std::mem::replace(&mut regs[*args as usize + k], Value::Unit);
                        callee.push((p.name.as_str(), v));
                    }
                    let result = self.run_body(f, &mut callee, *pos)?;
                    callee.clear();
                    self.env_pool.push(callee);
                    put(*dst, result, env, base, regs);
                }
                Ins::Print { src, pos } => {
                    pc += 1;
                    let v = take!(*src);
                    self.print(&v, *pos)?;
                }
                Ins::Index { dst, recv, idx, pos } => {
                    pc += 1;
                    let v = match get!(*recv) {
                        Value::List(items) => {
                            let i = get!(*idx).int();
                            element(items, i, *pos)?.clone()
                        }
                        Value::Map(t) => {
                            let k = get!(*idx);
                            map_value(t, k, || key_text(k), *pos)?.clone()
                        }
                        Value::Bytes(b) => {
                            let i = get!(*idx).int();
                            bytes::at(b, i, *pos)?
                        }
                        // std.tensor.15: `t[i]` drops axis 0.
                        Value::Tensor(t) => match t.row(get!(*idx).int()) {
                            Ok(r) => Value::Tensor(std::rc::Rc::new(r)),
                            Err(f) => return Err(Trap { rule: f.rule, pos: *pos, message: f.message }.into()),
                        },
                        other => unreachable!("checked: list, map, bytes or tensor, found {other:?}"),
                    };
                    done(*recv, regs);
                    done(*idx, regs);
                    put(*dst, v, env, base, regs);
                }
                Ins::Field { dst, recv, name, cache, pos } => {
                    pc += 1;
                    let v = match get!(*recv) {
                        Value::Struct(s, fields) => {
                            let (cs, ci) = cache.get();
                            let i = if cs == *s {
                                ci
                            } else {
                                let i = self.field_index(*s, name);
                                cache.set((*s, i));
                                i
                            };
                            fields[i].clone()
                        }
                        _ => {
                            let v = take!(*recv);
                            let Value::Struct(s, fields) = deref(v, *pos)? else { unreachable!("checked: struct") };
                            fields[self.field_index(s, name)].clone()
                        }
                    };
                    done(*recv, regs);
                    put(*dst, v, env, base, regs);
                }
                Ins::IfNotListMap { slot, t } => {
                    pc = if matches!(env[base + *slot as usize].1, Value::List(_) | Value::Map(_)) {
                        pc + 1
                    } else {
                        *t as usize
                    };
                }
                Ins::SetIndex { slot, idx, val, op, at, pos } => {
                    pc += 1;
                    let slot = base + *slot as usize;
                    let i = take!(*idx);
                    let is_map = matches!(env[slot].1, Value::Map(_));
                    // type.map.7: `m[k] op= v` on a missing key starts from zero.
                    let missing = op.is_some() && matches!(&env[slot].1, Value::Map(t) if t.get(&i).is_none());
                    let steps = [if is_map { Step::Key(i, *at) } else { Step::Index(i.int(), *at) }];
                    let v = match op {
                        None => take!(*val),
                        Some(op) if missing => {
                            let rhs = take!(*val);
                            self.binop(*op, zero_like(&rhs), rhs, *pos)?
                        }
                        Some(op) => {
                            let old = peek_at(env, slot, &steps)?.clone();
                            let rhs = take!(*val);
                            self.binop(*op, old, rhs, *pos)?
                        }
                    };
                    *slot_mut_at(env, slot, &steps, op.is_none() || missing)? = v;
                }
                Ins::Try { dst, src, context, absent, optional, pos, exit } => match take!(*src) {
                    Value::None if *absent => {
                        let how = self.absent(*context, *pos, env)?;
                        pc = self.leave(code, *exit, how, env, base)?;
                    }
                    Value::None if *optional => {
                        self.returned = Some((Value::None, *pos));
                        pc = self.leave(code, *exit, Exit::Return, env, base)?;
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
                        pc = self.leave(code, *exit, Exit::Return, env, base)?;
                    }
                    v => {
                        pc += 1;
                        let v = if *optional { Value::unwrap_present(v) } else { v };
                        put(*dst, v, env, base, regs);
                    }
                },
                Ins::ElseTest { src, t } => {
                    let r = (*src & !KIND) as usize;
                    if matches!(regs[r], Value::None | Value::Fail(_)) {
                        pc = *t as usize;
                    } else {
                        pc += 1;
                        let v = std::mem::replace(&mut regs[r], Value::Unit);
                        regs[r] = Value::unwrap_present(v);
                    }
                }
                Ins::IfNotStruct { slot, t } => {
                    pc = if matches!(env[base + *slot as usize].1, Value::Struct(..)) { pc + 1 } else { *t as usize };
                }
                Ins::SetField { slot, name, val, op, pos } => {
                    pc += 1;
                    let at = base + *slot as usize;
                    let Value::Struct(s, fields) = &env[at].1 else { unreachable!("checked by the guard") };
                    let index = self.field_index(*s, name);
                    let v = match op {
                        None => take!(*val),
                        Some(op) => {
                            let old = fields[index].clone();
                            let rhs = take!(*val);
                            self.binop(*op, old, rhs, *pos)?
                        }
                    };
                    let Value::Struct(_, fields) = &mut env[at].1 else { unreachable!("checked: struct") };
                    Rc::make_mut(fields)[index] = v;
                }
                Ins::Arm { src, pat, t } => {
                    let v = &regs[(*src & !KIND) as usize];
                    if !self.matches(pat, v) {
                        pc = *t as usize;
                    } else {
                        pc += 1;
                        if let (Pattern::Variant { bindings: Some(names), .. }, Value::Enum(_, _, fields)) = (pat, v) {
                            for ((n, _), value) in names.iter().zip(fields.iter()) {
                                if n != "_" {
                                    env.push((n.as_str(), value.clone()));
                                }
                            }
                        }
                    }
                }
                Ins::Interp { dst, parts, args } => {
                    pc += 1;
                    let mut text = String::new();
                    let mut k = *args as usize;
                    for part in parts.iter() {
                        match part {
                            InterpPart::Lit(t) => text.push_str(t),
                            InterpPart::Expr(_) | InterpPart::Fmt(..) => {
                                let Value::Str(t) = &regs[k] else { unreachable!("a part's text") };
                                text.push_str(t);
                                regs[k] = Value::Unit;
                                k += 1;
                            }
                        }
                    }
                    put(*dst, Value::Str(Rc::from(text)), env, base, regs);
                }
                Ins::Text { r, spec, pos } => {
                    pc += 1;
                    let r = (*r & !KIND) as usize;
                    if spec.is_some() || !matches!(regs[r], Value::Str(_)) {
                        let v = std::mem::replace(&mut regs[r], Value::Unit);
                        let t = match spec {
                            None => self.text(&v, *pos)?,
                            Some(spec) => self.formatted(&v, spec, *pos)?,
                        };
                        regs[r] = Value::Str(Rc::from(t));
                    }
                }
                Ins::Build { dst, what, slots, args } => {
                    pc += 1;
                    let first = *args as usize;
                    let n = match what {
                        Built::Struct(s) => self.prog.structs[*s].fields.len(),
                        Built::Variant(e, v) | Built::Qualified(e, v) => self.prog.enums[*e].variants[*v].fields.len(),
                    };
                    let mut values = vec![Value::Unit; n];
                    for (k, &slot) in slots.iter().enumerate() {
                        values[slot] = std::mem::replace(&mut regs[first + k], Value::Unit);
                    }
                    let v = match what {
                        Built::Struct(s) => Value::Struct(*s, Rc::new(values)),
                        Built::Variant(e, v) if values.is_empty() => Value::Enum(*e, *v, Rc::clone(&self.no_fields)),
                        Built::Variant(e, v) | Built::Qualified(e, v) => Value::Enum(*e, *v, Rc::new(values)),
                    };
                    put(*dst, v, env, base, regs);
                }
                Ins::Map { dst, args, n, set } => {
                    pc += 1;
                    let first = *args as usize;
                    let mut t = Table::default();
                    for kv in regs[first..first + 2 * *n as usize].chunks_mut(2) {
                        let k = std::mem::replace(&mut kv[0], Value::Unit);
                        t.insert(k, Some(std::mem::replace(&mut kv[1], Value::Unit)));
                    }
                    let v = if *set { Value::Set(Rc::new(t)) } else { Value::Map(Rc::new(t)) };
                    put(*dst, v, env, base, regs);
                }
                Ins::List { dst, args, n } => {
                    pc += 1;
                    let first = *args as usize;
                    let items: Vec<Value> = regs[first..first + *n as usize]
                        .iter_mut()
                        .map(|r| std::mem::replace(r, Value::Unit))
                        .collect();
                    put(*dst, Value::List(Rc::new(items)), env, base, regs);
                }
                Ins::MethodGuard { recv, path, name, t, cache } => {
                    let ok = if *recv & KIND == LOCAL {
                        self.method_allowed(self.recv_at(env, base + *recv as usize, path), name, path.len(), cache)
                    } else {
                        self.method_allowed(Some(get!(*recv)), name, 1, cache)
                    };
                    pc = if ok { pc + 1 } else { *t as usize };
                }
                Ins::MethodRun { recv, path, name, mutating, args, n, dst, pos, guard } => {
                    pc += 1;
                    let place = *recv & KIND == LOCAL;
                    let at = base + (*recv & !KIND) as usize;
                    // A register receiver is moved out; a local's is read in place.
                    let owned = if place { None } else { Some(take!(*recv)) };
                    let me = match &owned {
                        Some(v) => v,
                        None => self.recv_at(env, at, path).expect("checked by the guard"),
                    };
                    let user = match me {
                        Value::Struct(s, _) => Some((false, *s)),
                        Value::Enum(e, _, _) => Some((true, *e)),
                        _ => None,
                    };
                    let (first, n) = (*args as usize, *n as usize);
                    let result = match user {
                        // trait.method.2: the receiver's type's method, as `invoke` calls it.
                        Some((is_enum, index)) => {
                            let Ins::MethodGuard { cache, .. } = &code.ins[*guard as usize] else {
                                unreachable!("a guard")
                            };
                            let Some((e, i, f)) = cache.get() else { unreachable!("set by the guard") };
                            debug_assert!(e == is_enum && i == index);
                            let mut callee = self.env_pool.pop().unwrap_or_else(|| Vec::with_capacity(16));
                            let moved = f.receiver.is_some_and(|r| r.mutable);
                            let me = match owned {
                                Some(v) => v,
                                None if moved => Value::Unit,
                                None => self.recv_at(env, at, path).expect("checked by the guard").clone(),
                            };
                            callee.push(("self", me));
                            for (k, p) in f.params.iter().enumerate() {
                                callee.push((p.name.as_str(), std::mem::replace(&mut regs[first + k], Value::Unit)));
                            }
                            // trait.method.3: a `mut self` receiver moves into the call and back.
                            if moved {
                                callee[0].1 = std::mem::replace(&mut env[at].1, Value::Unit);
                            }
                            let result = self.run_body(f, &mut callee, *pos)?;
                            if moved {
                                env[at].1 = std::mem::replace(&mut callee[0].1, Value::Unit);
                            }
                            callee.clear();
                            self.env_pool.push(callee);
                            result
                        }
                        None => {
                            let mut a = self.arg_pool.pop().unwrap_or_default();
                            a.extend(regs[first..first + n].iter_mut().map(|r| std::mem::replace(r, Value::Unit)));
                            let recv = match owned {
                                Some(v) => Operand::Value(v),
                                None if *mutating => {
                                    let mut p = Place::local(at);
                                    let mut v = &env[at].1;
                                    for name in path.iter() {
                                        let Value::Struct(s, fields) = v else { unreachable!("checked by the guard") };
                                        let i = self.field_index(*s, name);
                                        p.path.push(Step::Field(i));
                                        v = &fields[i];
                                    }
                                    Operand::Place(p)
                                }
                                None => {
                                    Operand::Value(self.recv_at(env, at, path).expect("checked by the guard").clone())
                                }
                            };
                            self.builtin_with(recv, name, a, *pos, env)?
                        }
                    };
                    put(*dst, result, env, base, regs);
                }
                Ins::TraitGuard { recv, tr, name, t, cache, now } => {
                    let v = get!(*recv);
                    // What `dispatch` decides by: the value's type constructor.
                    let key = match v {
                        Value::Ref(_) | Value::Handle(_) | Value::Atomic(_) => None,
                        Value::Struct(s, _) => Some((0, *s)),
                        Value::Enum(e, _, _) => Some((1, *e)),
                        Value::Int(_) => Some((2, 0)),
                        Value::Float(_) => Some((3, 0)),
                        Value::Str(_) => Some((4, 0)),
                        Value::Char(_) => Some((5, 0)),
                        Value::Bool(_) => Some((6, 0)),
                        Value::List(_) => Some((7, 0)),
                        _ => Some((8, 0)),
                    };
                    let pick = key.map(|(k, i)| match cache.get() {
                        Some((ck, ci, f)) if ck == k && ci == i => f,
                        _ => {
                            let f = self.dispatch(*tr, name, v);
                            cache.set(Some((k, i, f)));
                            f
                        }
                    });
                    // A `mut` receiver or parameter takes the walker's path.
                    let ok = pick.is_some_and(|f| {
                        f.is_none_or(|f| f.receiver.is_none_or(|r| !r.mutable) && f.params.iter().all(|p| !p.mutable))
                    });
                    if ok {
                        now.set(pick.flatten());
                        pc += 1;
                    } else {
                        pc = *t as usize;
                    }
                }
                Ins::TraitRun { recv, tr, args, n, dst, pos, guard } => {
                    pc += 1;
                    let Ins::TraitGuard { now, .. } = &code.ins[*guard as usize] else { unreachable!("a guard") };
                    let me = take!(*recv);
                    let first = *args as usize;
                    let result = match now.get() {
                        // trait.method.2: the implementation, as `invoke` calls it.
                        Some(f) => {
                            let mut callee = self.env_pool.pop().unwrap_or_else(|| Vec::with_capacity(16));
                            callee.push(("self", me));
                            for (k, p) in f.params.iter().enumerate() {
                                callee.push((p.name.as_str(), std::mem::replace(&mut regs[first + k], Value::Unit)));
                            }
                            let result = self.run_body(f, &mut callee, *pos)?;
                            callee.clear();
                            self.env_pool.push(callee);
                            result
                        }
                        // trait.std.1-3: the built-in `to_str`, `eq` and `cmp`, as `trait_method` makes them.
                        None => {
                            let std = self.resolved.std;
                            if *tr == std.to_str {
                                Value::Str(Rc::from(self.text(&me, *pos)?))
                            } else {
                                debug_assert!(*n == 1);
                                let other = std::mem::replace(&mut regs[first], Value::Unit);
                                if *tr == std.eq {
                                    Value::Bool(self.equal(&me, &other, *pos)?)
                                } else {
                                    let o = self.compare(&me, &other, *pos)?;
                                    self.ordering_value(o)
                                }
                            }
                        }
                    };
                    put(*dst, result, env, base, regs);
                }
                Ins::MethodOn { recv, e, dst, exit } => match {
                    self.stats.handoffs += 1;
                    if let Some(k) = &mut self.stats.kinds {
                        *k.entry(kind_of_expr(e)).or_default() += 1;
                    }
                    let Expr::Method { recv: r, name, name_pos, args } = e else { unreachable!("a method call") };
                    let v = take!(*recv);
                    self.method_with_value(r, v, name, args, *name_pos, env)
                } {
                    Ok(v) => {
                        pc += 1;
                        put(*dst, v, env, base, regs);
                    }
                    Err(how) => pc = self.leave(code, *exit, how, env, base)?,
                },
                Ins::Expr { e, dst, exit } => match {
                    self.stats.handoffs += 1;
                    if let Some(k) = &mut self.stats.kinds {
                        *k.entry(kind_of_expr(e)).or_default() += 1;
                    }
                    self.eval(e, env)
                } {
                    Ok(v) => {
                        pc += 1;
                        put(*dst, v, env, base, regs);
                    }
                    Err(how) => pc = self.leave(code, *exit, how, env, base)?,
                },
                Ins::Hit(pos) => {
                    pc += 1;
                    self.hit(*pos)?;
                }
                Ins::Stmt { s, exit } => match {
                    self.stats.handoffs += 1;
                    if let Some(k) = &mut self.stats.kinds {
                        *k.entry(Cow::Borrowed(kind_of_stmt(s))).or_default() += 1;
                    }
                    // The statement's `Hit` ran; the walker's `stmt` takes the step again.
                    if self.hook {
                        self.unhit();
                    }
                    self.stmt(s, env)
                } {
                    Ok(_) => pc += 1,
                    Err(how) => pc = self.leave(code, *exit, how, env, base)?,
                },
                Ins::EndBlock { mark, pos } => {
                    pc += 1;
                    self.end_block(env, base + *mark as usize, *pos)?;
                }
                Ins::Truncate(mark) => {
                    pc += 1;
                    env.truncate(base + *mark as usize);
                }
                Ins::RangeTest { name, i, hi, inclusive, end } => {
                    let (i, hi) = (get!(*i).int(), get!(*hi).int());
                    if if *inclusive { i <= hi } else { i < hi } {
                        env.push((name, Value::Int(i)));
                        pc += 1;
                    } else {
                        pc = *end as usize;
                    }
                }
                Ins::RangeNext { i, end, top } => {
                    let Value::Int(n) = &mut regs[(*i & !KIND) as usize] else { unreachable!("a counter") };
                    if *n == i64::MAX {
                        pc = *end as usize;
                    } else {
                        *n += 1;
                        pc = *top as usize;
                    }
                }
                Ins::IterStart { it, idx } => {
                    pc += 1;
                    let r = (*it & !KIND) as usize;
                    if let Value::Set(t) = &regs[r] {
                        regs[r] = Value::List(Rc::new(t.key_list()));
                    } else if let Value::Bytes(b) = &regs[r] {
                        regs[r] = Value::List(Rc::new(b.iter().map(|x| bytes::byte(*x)).collect()));
                    }
                    regs[(*idx & !KIND) as usize] = Value::Int(0);
                }
                Ins::IterNext { name, it, idx, end, pos } => {
                    let (r, n) = ((*it & !KIND) as usize, (*idx & !KIND) as usize);
                    let item = match &regs[r] {
                        Value::List(items) => {
                            let Value::Int(k) = regs[n] else { unreachable!("an index") };
                            let item = items.get(k as usize).cloned();
                            regs[n] = Value::Int(k + 1);
                            item
                        }
                        // conc.share.7: each value received, until the channel is closed and empty.
                        &Value::Handle(Handle::Channel(c)) => self.chan_next(c, *pos)?,
                        other => unreachable!("checked: iterable, found {other:?}"),
                    };
                    match item {
                        Some(v) => {
                            env.push((name, v));
                            pc += 1;
                        }
                        None => pc = *end as usize,
                    }
                }
                Ins::Bind { name, src, t } => match take!(*src) {
                    Value::None | Value::Fail(_) => pc = *t as usize,
                    v => {
                        env.push((name, Value::unwrap_present(v)));
                        pc += 1;
                    }
                },
                Ins::Leave { exit, brk } => {
                    pc = self.leave(code, *exit, if *brk { Exit::Break } else { Exit::Continue }, env, base)?;
                }
                Ins::Ret { src, pos, exit } => {
                    let v = take!(*src);
                    self.returned = Some((v, *pos));
                    self.leave(code, *exit, Exit::Return, env, base)?;
                }
                Ins::End { src } => return Ok(take!(*src)),
            }
        }
    }
}
