//! Tasks and their communication (spec 11, ADRs 0095-0096, 0100,
//! 0186-0189): `spawn` (conc.task.1-3), `Task.wait()` (conc.task.5-6),
//! `TaskGroup` (conc.task.4, conc.task.8), deadlock (conc.task.7),
//! `Channel[T]` (conc.share.3-4) and `Mutex[T]` (conc.share.2, conc.share.5).
//!
//! The reference interpreter's scheduler is deterministic (ADR 0186). Each
//! task runs its interpreter on its own OS thread, so a task can block
//! anywhere in its call stack, but only one task runs at a time: a task runs
//! until it blocks or finishes and then hands the turn to the next ready
//! task, first in, first out. A new task runs at once, and its spawner
//! resumes first when it blocks or finishes. Values never cross threads
//! while two run, which is what makes sharing `Rc` values sound.

use std::collections::VecDeque;
use std::io::{self, Write};
use std::rc::Rc;
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::thread::JoinHandle;

use super::{Env, Exit, Failure, Flow, Interp, Trap, Value, build};
use crate::ast::{Arg, Block, Expr, FnDecl, Program};
use crate::check::Resolved;
use crate::diag::Pos;

/// A task's stack: as deep as the main interpreter thread's.
const TASK_STACK: usize = 256 * 1024 * 1024;

/// A reference to scheduler state, by index.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Handle {
    Task(usize),
    Group(usize),
    Channel(usize),
    Mutex(usize),
}

impl Handle {
    pub(super) fn text(self) -> String {
        match self {
            Handle::Task(i) => format!("Task#{i}"),
            Handle::Group(i) => format!("TaskGroup#{i}"),
            Handle::Channel(i) => format!("Channel#{i}"),
            Handle::Mutex(i) => format!("Mutex#{i}"),
        }
    }
}

/// The running task's view of the scheduler.
pub(super) struct Ctx {
    sched: Arc<Sched>,
    me: usize,
}

struct Sched {
    state: Mutex<State>,
    turn: Condvar,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Status {
    Ready,
    Running,
    Blocked,
    Done,
}

struct TaskRec {
    status: Status,
    result: Option<Result<Value, Trap>>,
    /// Tasks waiting for this one to finish.
    waiters: Vec<usize>,
    /// Waited, or added to a group (conc.task.5).
    consumed: bool,
    /// Woken only to report that every task is blocked (conc.task.7).
    deadlock: bool,
    /// What it is blocked on, for the deadlock message.
    waiting: &'static str,
    /// A task running outside the scheduler (conc.par.10), until joined.
    pooled: Option<super::parallel::Pooled>,
    /// Asked to stop by `cancel` (conc.cancel.1).
    cancelled: bool,
}

struct Chan {
    items: VecDeque<Value>,
    capacity: usize,
    closed: bool,
    waiters: Vec<usize>,
}

struct Cell {
    value: Value,
    holder: Option<usize>,
    waiters: Vec<usize>,
}

struct State {
    current: usize,
    ready: VecDeque<usize>,
    tasks: Vec<TaskRec>,
    chans: Vec<Chan>,
    cells: Vec<Cell>,
    groups: Vec<Vec<usize>>,
    /// Per group: `cancel` was called, so tasks added later start cancelled (conc.cancel.1).
    groups_cancelled: Vec<bool>,
    /// The random schedule's state, under `--schedule=random` (conc.sched.1).
    rng: Option<u64>,
    /// What tasks other than the main one printed, in order, until the main
    /// task copies it to standard output.
    output: Vec<u8>,
    shutdown: bool,
    /// Each task's thread, by task.
    threads: Vec<(usize, JoinHandle<()>)>,
}

// SAFETY: `State` holds `Rc` values, but only the task holding the turn
// (`current`) touches them or runs, and the turn passes under this mutex,
// which orders every access.
unsafe impl Send for State {}

/// Moves values into a task's thread; see `State`.
struct Turned<T>(T);
// SAFETY: as for `State`: the receiving thread doesn't run until the
// sender hands it the turn.
unsafe impl<T> Send for Turned<T> {}

fn new_task(status: Status) -> TaskRec {
    TaskRec { status, result: None, waiters: Vec::new(), consumed: false, deadlock: false, waiting: "", pooled: None, cancelled: false }
}

impl Sched {
    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Make a blocked task ready again.
    fn wake(st: &mut State, id: usize) {
        if st.tasks[id].status == Status::Blocked {
            st.tasks[id].status = Status::Ready;
            st.ready.push_back(id);
        }
    }

    /// The next ready task to run: the oldest, or under a random schedule
    /// one picked from the seed (conc.sched.1).
    fn next_ready(st: &mut State) -> Option<usize> {
        match st.rng.as_mut() {
            Some(r) if st.ready.len() > 1 => {
                let i = (next_random(r) % st.ready.len() as u64) as usize;
                st.ready.remove(i)
            }
            _ => st.ready.pop_front(),
        }
    }

    /// conc.cancel.1: mark task `t` cancelled and wake it if it is blocked,
    /// so a blocking call can return early.
    fn cancel(st: &mut State, t: usize) {
        st.tasks[t].cancelled = true;
        Sched::wake(st, t);
    }

    /// Give the turn to `next`.
    fn hand_to(&self, st: &mut State, next: usize) {
        st.current = next;
        st.tasks[next].status = Status::Running;
        self.turn.notify_all();
    }

    /// Wait until it is `me`'s turn again.
    fn await_turn<'s>(&'s self, mut st: MutexGuard<'s, State>, me: usize, pos: Pos) -> Result<MutexGuard<'s, State>, Trap> {
        while st.current != me {
            st = self.turn.wait(st).unwrap_or_else(|e| e.into_inner());
        }
        if st.shutdown {
            return Err(cancelled(pos));
        }
        if std::mem::take(&mut st.tasks[me].deadlock) {
            return Err(deadlock(&st, me, pos));
        }
        Ok(st)
    }

    /// Block `me`, already registered as a waiter, until woken (conc.task.2).
    /// `waiting` says for what, e.g. "a value from a channel".
    fn block<'s>(
        &'s self,
        mut st: MutexGuard<'s, State>,
        me: usize,
        waiting: &'static str,
        pos: Pos,
    ) -> Result<MutexGuard<'s, State>, Trap> {
        if st.shutdown {
            return Err(cancelled(pos));
        }
        st.tasks[me].waiting = waiting;
        match Sched::next_ready(&mut st) {
            Some(next) => {
                st.tasks[me].status = Status::Blocked;
                self.hand_to(&mut st, next);
                self.await_turn(st, me, pos)
            }
            // conc.task.7: nothing else can run, so nothing can wake `me`.
            None => Err(deadlock(&st, me, pos)),
        }
    }

    /// conc.sched.1: a switch point. Under a random schedule, `me` may hand
    /// the turn to another ready task, picked from the seed, before it goes on.
    fn switch<'s>(&'s self, mut st: MutexGuard<'s, State>, me: usize, pos: Pos) -> Result<MutexGuard<'s, State>, Trap> {
        if st.ready.is_empty() {
            return Ok(st);
        }
        let Some(r) = st.rng.as_mut() else { return Ok(st) };
        if next_random(r) & 1 == 0 {
            return Ok(st);
        }
        let next = Sched::next_ready(&mut st).expect("not empty");
        st.tasks[me].status = Status::Ready;
        st.ready.push_back(me);
        self.hand_to(&mut st, next);
        self.await_turn(st, me, pos)
    }

    /// `me` has finished: record its result, wake its waiters, and pass the turn on.
    fn finish(&self, me: usize, result: Result<Value, Trap>) {
        let mut st = self.lock();
        st.tasks[me].status = Status::Done;
        st.tasks[me].result = Some(result);
        for w in std::mem::take(&mut st.tasks[me].waiters) {
            Sched::wake(&mut st, w);
        }
        if st.shutdown {
            return;
        }
        if let Some(next) = Sched::next_ready(&mut st) {
            self.hand_to(&mut st, next);
        } else if let Some(stuck) = st.tasks.iter().position(|t| t.status == Status::Blocked) {
            // conc.task.7: every task left is blocked; the first one traps.
            st.tasks[stuck].deadlock = true;
            self.hand_to(&mut st, stuck);
        }
    }
}

/// The random schedule's seed (conc.sched.1): `None` for the fixed order.
static SCHEDULE: std::sync::Mutex<Option<u64>> = std::sync::Mutex::new(None);

/// `--schedule=random:SEED` (conc.sched.1): every scheduler created after
/// this picks its switch points from `seed`.
pub fn set_schedule(seed: Option<u64>) {
    *SCHEDULE.lock().unwrap_or_else(|e| e.into_inner()) = seed;
}

/// The random schedule's seed, if one is set.
pub fn schedule_seed() -> Option<u64> {
    *SCHEDULE.lock().unwrap_or_else(|e| e.into_inner())
}

/// xorshift64*: small, fast and the same on every target, so a seed
/// replays the same interleaving everywhere (conc.sched.1).
fn next_random(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x >> 12;
    x ^= x << 25;
    x ^= x >> 27;
    *state = x;
    x.wrapping_mul(0x2545_f491_4f6c_dd1d)
}

/// conc.task.7: `me` and every other unfinished task are blocked.
fn deadlock(st: &State, me: usize, pos: Pos) -> Trap {
    let who = if me == 0 { "the main task".to_string() } else { format!("task {me}") };
    let others = st.tasks.iter().filter(|t| t.status == Status::Blocked).count();
    let message = format!(
        "deadlock: {who} waits for {}, and {} other task{} blocked",
        st.tasks[me].waiting,
        if others == 0 { "no".to_string() } else { others.to_string() },
        if others == 1 { " is" } else { "s are" },
    );
    Trap { rule: "conc.task.7", pos, message }
}

fn cancelled(pos: Pos) -> Trap {
    Trap { rule: "conc.task.4", pos, message: "the task was stopped because the program ended".into() }
}

/// The program is ending: stop every unfinished task, one at a time so that
/// still only one thread touches values, and wait for every thread.
pub(super) fn shutdown(ctx: Option<Ctx>) {
    let Some(ctx) = ctx else { return };
    let (threads, pooled) = {
        let mut st = ctx.sched.lock();
        st.shutdown = true;
        let pooled: Vec<_> = st.tasks.iter_mut().filter_map(|t| t.pooled.take()).collect();
        (std::mem::take(&mut st.threads), pooled)
    };
    // conc.par.10: pooled tasks have no effects; let them finish.
    for task in pooled {
        let _ = task.join();
    }
    for (id, t) in threads {
        {
            let mut st = ctx.sched.lock();
            if st.tasks[id].status != Status::Done {
                ctx.sched.hand_to(&mut st, id);
            }
        }
        let _ = t.join();
    }
}

impl<'a, W: Write> Interp<'a, W> {
    /// The scheduler, created by the first task or handle; the program's
    /// main task is task 0.
    fn ctx(&mut self) -> &Ctx {
        self.conc.get_or_insert_with(|| {
            let state = State {
                current: 0,
                ready: VecDeque::new(),
                tasks: vec![new_task(Status::Running)],
                chans: Vec::new(),
                cells: Vec::new(),
                groups: Vec::new(),
                groups_cancelled: Vec::new(),
                // A zero state would stay zero.
                rng: schedule_seed().map(|s| s ^ 0x9e37_79b9_7f4a_7c15 | 1),
                output: Vec::new(),
                shutdown: false,
                threads: Vec::new(),
            };
            Ctx { sched: Arc::new(Sched { state: Mutex::new(state), turn: Condvar::new() }), me: 0 }
        })
    }

    /// conc.cancel.2: whether the running task was cancelled. The main
    /// task and a task outside the scheduler never are.
    pub(super) fn cancelled(&self) -> bool {
        self.conc.as_ref().is_some_and(|ctx| ctx.sched.lock().tasks[ctx.me].cancelled)
    }

    /// conc.sched.1: a switch point outside a handle's methods (an atomic
    /// operation, `task.cancelled()`); does nothing without a random schedule.
    pub(super) fn switch_point(&mut self, pos: Pos) -> Result<(), Trap> {
        let Some(ctx) = &self.conc else { return Ok(()) };
        let (sched, me) = (Arc::clone(&ctx.sched), ctx.me);
        let st = sched.lock();
        drop(sched.switch(st, me, pos)?);
        Ok(())
    }

    /// Print from a task other than the main one, or copy what tasks printed
    /// to standard output first. Returns whether `text` was taken.
    pub(super) fn task_print(&mut self, text: &str) -> bool {
        let Some(ctx) = &self.conc else { return false };
        let mut st = ctx.sched.lock();
        if ctx.me != 0 {
            st.output.extend_from_slice(text.as_bytes());
            st.output.push(b'\n');
            return true;
        }
        let pending = std::mem::take(&mut st.output);
        drop(st);
        if !pending.is_empty() && !self.closed {
            let _ = self.out.write_all(&pending);
        }
        false
    }

    /// `spawn f(args)` (conc.task.1): copy the arguments (conc.share.1) and
    /// start the task, which runs until it blocks or finishes.
    pub(super) fn spawn(&mut self, operand: &'a Expr, pos: Pos, env: &mut Env<'a>) -> Flow<Value> {
        let (f, args): (&'a FnDecl, &'a [Arg]) = match operand {
            Expr::Call { callee, args, .. } => {
                let (Expr::Name(n, _) | Expr::TypeArgs { name: n, .. }) = &**callee else {
                    unreachable!("checked: a named function")
                };
                (self.fns[n.as_str()], args)
            }
            Expr::Method { recv, name, args, .. } => {
                let (Expr::Name(t, _) | Expr::TypeArgs { name: t, .. }) = &**recv else {
                    unreachable!("checked: an associated function")
                };
                (self.methods[t.as_str()][name.as_str()], args)
            }
            _ => unreachable!("checked: `spawn` takes a call"),
        };
        let mut values = Vec::with_capacity(args.len());
        for a in args {
            values.push(self.eval(&a.value, env)?);
        }
        if self.tracing & super::trace::EFFECTS != 0 {
            self.trace_task(true, &f.full_name(), &values, pos);
        }
        let pooled = self.pool_task(f, &values, pos);
        let ctx = self.ctx();
        let (sched, me) = (Arc::clone(&ctx.sched), ctx.me);
        let mut st = sched.lock();
        // conc.par.10: a pure task runs on its own thread, outside the
        // scheduler; `wait` joins it.
        if let Some(task) = pooled {
            let mut rec = new_task(Status::Done);
            rec.pooled = Some(task);
            st.tasks.push(rec);
            return Ok(Value::Handle(Handle::Task(st.tasks.len() - 1)));
        }
        // Reclaim the threads of finished tasks.
        let (done, live) = std::mem::take(&mut st.threads).into_iter().partition(|(_, t)| t.is_finished());
        st.threads = live;
        for (_, t) in done {
            let _ = t.join();
        }
        let id = st.tasks.len();
        let mut rec = new_task(Status::Ready);
        // conc.cancel.1: a task started by a cancelled task starts cancelled.
        rec.cancelled = st.tasks[me].cancelled;
        st.tasks.push(rec);
        // SAFETY: `run` and `run_tests` stop and join every task thread
        // before the program and checker facts they borrow are dropped.
        let (prog, resolved, f) = unsafe {
            (
                std::mem::transmute::<&Program, &'static Program>(self.prog),
                std::mem::transmute::<&Resolved, &'static Resolved>(self.resolved),
                std::mem::transmute::<&FnDecl, &'static FnDecl>(f),
            )
        };
        let moved = Turned((values, Arc::clone(&sched)));
        let thread = std::thread::Builder::new().stack_size(TASK_STACK).spawn(move || {
            let moved = moved;
            let (args, sched) = moved.0;
            let st = sched.lock();
            let st = match sched.await_turn(st, id, pos) {
                Ok(st) => st,
                Err(t) => {
                    sched.finish(id, Err(t));
                    return;
                }
            };
            drop(st);
            let mut it = build(prog, resolved, io::sink());
            it.conc = Some(Ctx { sched: Arc::clone(&sched), me: id });
            let mut result = it.call(f, None, args, pos).map(|(v, _)| v);
            drop(it);
            // conc.share.5: a task that ends holding a lock releases it, and traps.
            let mut st = sched.lock();
            for cell in st.cells.iter_mut().filter(|c| c.holder == Some(id)) {
                cell.holder = None;
                if result.is_ok() {
                    let message = "the task ended while holding a mutex from `lock`; call `unlock`".into();
                    result = Err(Trap { rule: "conc.share.5", pos: f.pos, message });
                }
            }
            let waiters: Vec<usize> = st.cells.iter_mut().flat_map(|c| std::mem::take(&mut c.waiters)).collect();
            for w in waiters {
                Sched::wake(&mut st, w);
            }
            drop(st);
            sched.finish(id, result);
        });
        match thread {
            Ok(t) => st.threads.push((id, t)),
            Err(e) => {
                st.tasks[id].status = Status::Done;
                return Err(Trap { rule: "conc.task.3", pos, message: format!("can't start a task: {e}") }.into());
            }
        }
        // The new task runs first; its spawner resumes next. Under a random
        // schedule, a coin decides whether the spawner goes on instead
        // (conc.sched.1).
        if let Some(r) = st.rng.as_mut()
            && next_random(r) & 1 == 0
        {
            st.ready.push_back(id);
            return Ok(Value::Handle(Handle::Task(id)));
        }
        st.tasks[me].status = Status::Ready;
        st.ready.push_front(me);
        sched.hand_to(&mut st, id);
        drop(sched.await_turn(st, me, pos)?);
        Ok(Value::Handle(Handle::Task(id)))
    }

    /// `TaskGroup()`, `Channel[T](capacity)`, `Mutex(value)`.
    pub(super) fn conc_constructor(&mut self, name: &str, args: &'a [Arg], pos: Pos, env: &mut Env<'a>) -> Flow<Value> {
        let arg = match args.first() {
            Some(a) => Some(self.eval(&a.value, env)?),
            None => None,
        };
        let ctx = self.ctx();
        let mut st = ctx.sched.lock();
        Ok(Value::Handle(match (name, arg) {
            ("Channel", Some(cap)) => {
                let capacity = cap.int();
                if capacity < 1 {
                    let message = format!("a channel's capacity must be at least 1, found {capacity}");
                    return Err(Trap { rule: "conc.share.4", pos, message }.into());
                }
                let capacity = usize::try_from(capacity).unwrap_or(usize::MAX);
                st.chans.push(Chan { items: VecDeque::new(), capacity, closed: false, waiters: Vec::new() });
                Handle::Channel(st.chans.len() - 1)
            }
            ("Mutex", Some(value)) => {
                st.cells.push(Cell { value, holder: None, waiters: Vec::new() });
                Handle::Mutex(st.cells.len() - 1)
            }
            _ => {
                st.groups.push(Vec::new());
                st.groups_cancelled.push(false);
                Handle::Group(st.groups.len() - 1)
            }
        }))
    }

    /// A method of a task, group, channel or mutex.
    pub(super) fn handle_method(&mut self, h: Handle, name: &str, args: &'a [Arg], pos: Pos, env: &mut Env<'a>) -> Flow<Value> {
        let mut a = Vec::with_capacity(args.len());
        for arg in args {
            a.push(self.eval(&arg.value, env)?);
        }
        let ctx = self.ctx();
        let (sched, me) = (Arc::clone(&ctx.sched), ctx.me);
        let sched = &*sched;
        let mut st = sched.lock();
        st = sched.switch(st, me, pos)?;
        let trap = |rule: &'static str, message: String| -> Flow<Value> { Err(Trap { rule, pos, message }.into()) };
        Ok(match (h, name) {
            (Handle::Task(t), "wait") => {
                if self.tracing & super::trace::EFFECTS != 0 {
                    self.trace_task(false, &Handle::Task(t).text(), &[], pos);
                }
                st.tasks[t].consumed = true;
                drop(st);
                return Ok(match self.join(sched, me, t, pos)? {
                    Ok(v) => v,
                    Err(trap) => self.trapped(&trap, pos),
                });
            }
            (Handle::Task(t), "cancel") => {
                Sched::cancel(&mut st, t);
                Value::Unit
            }
            (Handle::Group(g), "cancel") => {
                st.groups_cancelled[g] = true;
                for t in st.groups[g].clone() {
                    Sched::cancel(&mut st, t);
                }
                Value::Unit
            }
            (Handle::Group(g), "add") => {
                let Value::Handle(Handle::Task(t)) = a[0] else { unreachable!("checked: a task") };
                if st.groups_cancelled[g] {
                    Sched::cancel(&mut st, t);
                }
                st.tasks[t].consumed = true;
                st.groups[g].push(t);
                Value::Unit
            }
            (Handle::Group(g), "wait_all") => {
                drop(st);
                return Ok(match self.wait_group(sched, me, g, pos)? {
                    Some(trap) => self.trapped(&trap, pos),
                    None => Value::Unit,
                });
            }
            (Handle::Channel(c), "send") => {
                let v = a.pop().expect("checked: one argument");
                loop {
                    let ch = &mut st.chans[c];
                    if ch.closed {
                        return trap("conc.share.4", format!("send on a closed channel `{}`", h.text()));
                    }
                    if ch.items.len() < ch.capacity {
                        ch.items.push_back(v);
                        for w in std::mem::take(&mut ch.waiters) {
                            Sched::wake(&mut st, w);
                        }
                        break;
                    }
                    ch.waiters.push(me);
                    st = sched.block(st, me, "room in a channel", pos)?;
                }
                Value::Unit
            }
            (Handle::Channel(c), "recv") => loop {
                let cancelled = st.tasks[me].cancelled;
                let ch = &mut st.chans[c];
                if let Some(v) = ch.items.pop_front() {
                    for w in std::mem::take(&mut ch.waiters) {
                        Sched::wake(&mut st, w);
                    }
                    break v;
                }
                // conc.cancel.2: a cancelled task's `recv` returns early.
                if ch.closed || cancelled {
                    break Value::None;
                }
                ch.waiters.push(me);
                st = sched.block(st, me, "a value from a channel", pos)?;
            },
            (Handle::Channel(c), _) => {
                let ch = &mut st.chans[c];
                if ch.closed {
                    return trap("conc.share.4", format!("channel `{}` is already closed", h.text()));
                }
                ch.closed = true;
                for w in std::mem::take(&mut ch.waiters) {
                    Sched::wake(&mut st, w);
                }
                Value::Unit
            }
            (Handle::Mutex(m), "unlock") => {
                let cell = &mut st.cells[m];
                if cell.holder != Some(me) {
                    return trap("conc.share.5", "`unlock` without holding the mutex from `lock`".into());
                }
                cell.value = a.pop().expect("checked: one argument");
                cell.holder = None;
                for w in std::mem::take(&mut cell.waiters) {
                    Sched::wake(&mut st, w);
                }
                Value::Unit
            }
            (Handle::Mutex(m), _) => {
                // Every access waits while another task holds the lock.
                loop {
                    let cell = &mut st.cells[m];
                    match cell.holder {
                        Some(h) if h == me => {
                            if name == "lock" {
                                return trap("conc.share.5", "`lock` while already holding the mutex".into());
                            }
                            break;
                        }
                        Some(_) => {
                            cell.waiters.push(me);
                            st = sched.block(st, me, "a mutex another task holds", pos)?;
                        }
                        None => break,
                    }
                }
                let cell = &mut st.cells[m];
                match name {
                    "get" => cell.value.clone(),
                    "lock" => {
                        cell.holder = Some(me);
                        cell.value.clone()
                    }
                    "set" => {
                        cell.value = a.pop().expect("checked: one argument");
                        Value::Unit
                    }
                    _ => std::mem::replace(&mut cell.value, a.pop().expect("checked: one argument")),
                }
            }
            _ => unreachable!("checked: a method of `{}`", h.text()),
        })
    }

    /// The next value of channel `c` for `for v in ch` (conc.share.7): the
    /// oldest value, or `None` once it is closed and empty. Waits like `recv`.
    pub(super) fn chan_next(&mut self, c: usize, pos: Pos) -> Flow<Option<Value>> {
        let ctx = self.ctx();
        let (sched, me) = (Arc::clone(&ctx.sched), ctx.me);
        let mut st = sched.lock();
        st = sched.switch(st, me, pos)?;
        loop {
            let cancelled = st.tasks[me].cancelled;
            let ch = &mut st.chans[c];
            if let Some(v) = ch.items.pop_front() {
                for w in std::mem::take(&mut ch.waiters) {
                    Sched::wake(&mut st, w);
                }
                return Ok(Some(v));
            }
            // conc.cancel.2: a cancelled task's loop ends like a closed channel's.
            if ch.closed || cancelled {
                return Ok(None);
            }
            ch.waiters.push(me);
            st = sched.block(st, me, "a value from a channel", pos)?;
        }
    }

    /// `with m as mut x` body (conc.share.6): take the lock, run the body
    /// with the value in `x`, then store `x` and release it, however the
    /// body ends. A trap releases it and keeps the value from before.
    pub(super) fn with_block(&mut self, mutex: &'a Expr, var: &'a str, body: &'a Block, pos: Pos, env: &mut Env<'a>) -> Flow<()> {
        let Value::Handle(h @ Handle::Mutex(m)) = self.eval(mutex, env)? else { unreachable!("checked: a mutex") };
        let v = self.handle_method(h, "lock", &[], pos, env)?;
        let depth = env.len();
        env.push((var, v));
        let r = self.block(body, env);
        let v = env.drain(depth..).next().map(|(_, v)| v);
        let ctx = self.ctx();
        let mut st = ctx.sched.lock();
        let cell = &mut st.cells[m];
        if let (Some(v), false) = (v, matches!(r, Err(Exit::Trap(_)))) {
            cell.value = v;
        }
        cell.holder = None;
        for w in std::mem::take(&mut cell.waiters) {
            Sched::wake(&mut st, w);
        }
        r.map(|_| ())
    }

    /// Wait for task `t` (conc.task.6): its result, or its trap.
    fn join(&mut self, sched: &Sched, me: usize, t: usize, pos: Pos) -> Result<Result<Value, Trap>, Trap> {
        let pooled = sched.lock().tasks[t].pooled.take();
        if let Some(task) = pooled {
            return Ok(super::parallel::unpool(task, pos));
        }
        let mut st = sched.lock();
        while st.tasks[t].status != Status::Done {
            st.tasks[t].waiters.push(me);
            st = sched.block(st, me, "a task to finish", pos)?;
        }
        Ok(st.tasks[t].result.take().unwrap_or(Ok(Value::Unit)))
    }

    /// Wait for every task in group `g` (conc.task.8); the first trap.
    fn wait_group(&mut self, sched: &Sched, me: usize, g: usize, pos: Pos) -> Result<Option<Trap>, Trap> {
        let tasks = std::mem::take(&mut sched.lock().groups[g]);
        let mut first = None;
        for t in tasks {
            if let Err(trap) = self.join(sched, me, t, pos)? {
                first.get_or_insert(trap);
            }
        }
        Ok(first)
    }

    /// A failure carrying `Trapped { rule, message }` (conc.task.6).
    fn trapped(&self, trap: &Trap, pos: Pos) -> Value {
        let s = self.structs["Trapped"];
        let message = format!("{}:{}: {}", trap.pos.line, trap.pos.col, trap.message);
        let error = Value::Struct(s, Rc::new(vec![Value::Str(Rc::from(trap.rule)), Value::Str(Rc::from(message))]));
        Value::Fail(Rc::new(Failure { error, context: Vec::new(), trace: vec![pos] }))
    }

    /// conc.task.4: when a block ends, wait for the tasks it spawned and
    /// didn't wait (it left early), and for its groups. A trapped task in a
    /// group that nothing waited explicitly traps here (conc.task.8).
    #[cold]
    #[inline(never)]
    pub(super) fn end_tasks(&mut self, locals: &[(&'a str, Value)], pos: Pos) -> Result<(), Trap> {
        let handles: Vec<Handle> = locals
            .iter()
            .filter_map(|(_, v)| match v {
                Value::Handle(h @ (Handle::Task(_) | Handle::Group(_))) => Some(*h),
                _ => None,
            })
            .collect();
        if handles.is_empty() {
            return Ok(());
        }
        let ctx = self.ctx();
        let (sched, me) = (Arc::clone(&ctx.sched), ctx.me);
        let mut first = None;
        for h in handles {
            match h {
                Handle::Task(t) => {
                    if !std::mem::replace(&mut sched.lock().tasks[t].consumed, true) {
                        let _ = self.join(&sched, me, t, pos)?;
                    }
                }
                Handle::Group(g) => {
                    if let Some(trap) = self.wait_group(&sched, me, g, pos)? {
                        first.get_or_insert(trap);
                    }
                }
                _ => {}
            }
        }
        match first {
            Some(trap) => Err(trap),
            None => Ok(()),
        }
    }
}
