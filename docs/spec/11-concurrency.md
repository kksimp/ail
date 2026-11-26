# 11. Concurrency

Chapter key: `conc`

> **Status:** Drafted (partial). Specified: memory-model foundations, the task
> model, scope-bound tasks, communication through `Channel[T]` and
> `Mutex[T]`, `Atomic[int]` and `Atomic[bool]`, cancellation, random-schedule
> testing, parallel loops and `select`. Atomic orderings (`AtomicOrder`)
> are specified with the atomics. Not yet designed: the formal weak-memory model.

## Memory model

**conc.mm.1** — [definitional] The AIL memory model is defined on an abstract,
weakly-ordered machine. It MUST NOT be defined in terms of, or guarantee, the
memory-ordering behaviour of any particular hardware architecture. Programs
MUST NOT rely on hardware ordering stronger than the model guarantees, even on
targets that provide it. *(ADR 0025)*

> **Rationale:** a model tied to x86-64's strong (TSO) ordering would force
> barriers throughout ARM64 code. A model tied to ARM64 would leak hardware
> details. See [ADR 0008](../decisions/0008-isa-priority-policy.md).

## Data races

**conc.race.1** — A program that contains no `unsafe` code MUST NOT be able to
cause a data race. A data race is two unsynchronised accesses to the same
memory location from different threads, at least one of which is a write and
at least one of which is not atomic. *(ADR 0025)*

> **Note:** the enforcement mechanism (type system, ownership, task
> confinement) is not yet designed. The precise definition of
> "unsynchronised" will be given by the formal model.

## Atomics

**conc.atomic.1** — An atomic operation that does not state a memory ordering
has sequentially consistent ordering. *(ADR 0025)*

**conc.atomic.2** — An atomic operation MAY explicitly request a weaker
ordering. The language MUST provide at least acquire, release,
acquire-release and relaxed orderings. *(ADR 0025)*

> **Note:** an implementation SHOULD report atomic operations whose ordering
> could be weakened without changing behaviour. It MUST NOT weaken an
> ordering on its own initiative.

**conc.atomic.3** — `Atomic(v)` (or `Atomic[T](v)`) makes an atomic cell
holding `v`. `T` MUST be `int` or `bool`. An atomic is a handle: copies of
it, copies passed to tasks, and the copies a parallel loop's iterations see
are the same cell, so tasks and `par for` share it without a `ref` or a
`Mutex`. *(ADR 0320)*

**conc.atomic.4** — `a.load()` returns the value, `a.store(v)` replaces it,
and `a.cas(old, new)` replaces it with `new` only if it equals `old`,
returning whether it did. Each is one indivisible step, sequentially
consistent (`conc.atomic.1`) unless `order:` says otherwise
(`conc.atomic.7`). *(ADR 0320)*

**conc.atomic.5** — On an `Atomic[int]`, `a.add(n)` adds `n` in one
indivisible step and returns the value from before. An add whose result
would leave `int` traps and leaves the cell unchanged. *(ADR 0320)*

```ail rules=conc.atomic.3,conc.atomic.5
fn count(hits Atomic[int], n int)
    for _ in 0..n
        hits.add(1)

fn main() !
    hits := Atomic(0)
    g := TaskGroup()
    for _ in 0..4
        g.add(spawn count(hits, 100))
    g.wait_all()?
    print(hits.load())
```

**conc.atomic.6** — A call of an atomic's method, directly or through the
functions a loop body calls, makes a `for` loop not proven independent
(`conc.par.2`), since the operations may see the order of the iterations.
`par for` asserts that the result doesn't depend on it, and such a loop
may run on the pool (`conc.par.8`): iterations then update the one cell in
any order. Iterations that already ran can't be undone, so if one traps
the loop reports that trap instead of running again in order
(`conc.par.11`). A task that calls atomic methods but has no other
ordering effect may still run on its own thread (`conc.par.10`).
*(ADR 0320)*

```ail rules=conc.atomic.6
fn main()
    total := Atomic(0)
    par for i in 0..1000
        total.add(i)
    print(total.load())
```

**conc.atomic.7** — A trailing named argument `order: O` gives an atomic
method a weaker ordering (`conc.atomic.2`). `O` is written as a variant of
the prelude enum `AtomicOrder`: `SeqCst`, `Acquire`, `Release`, `AcqRel` or
`Relaxed`; any other expression is an error, so the checker always knows
the ordering. `load` accepts `SeqCst`, `Acquire` and `Relaxed`; `store`
accepts `SeqCst`, `Release` and `Relaxed`; `add` and `cas` accept all
five. A `cas` that fails reads with the read half of its ordering
(`Relaxed` for `Release`, `Acquire` for `AcqRel`). The orderings mean what
they mean in the C++20 memory model. *(ADR 0352)*

```ail rules=conc.atomic.2,conc.atomic.7
fn publish(data Atomic[int], ready Atomic[bool])
    data.store(42, order: Relaxed)
    ready.store(true, order: Release)   # data is visible to an Acquire load that sees true

fn main()
    data := Atomic(0)
    ready := Atomic(false)
    publish(data, ready)
    if ready.load(order: Acquire)
        print(data.load(order: Relaxed))
```

## Tasks

**conc.task.1** — `spawn e`, where `e` is a function call, starts a new task
that evaluates `e` concurrently. It yields a task handle. *(ADR 0095)*

**conc.task.2** — Any function may block. A blocked task is suspended without
blocking other tasks. No function or call is marked as asynchronous.
*(ADR 0095)*

**conc.task.3** — Tasks are scheduled by the runtime onto operating-system
threads. *(ADR 0095)*

**conc.task.4** — Every task spawned within a block completes before that
block exits. Long-lived tasks are spawned into a `TaskGroup`, which waits for
its tasks when its owning scope exits. *(ADR 0096)*

**conc.task.5** — A task handle is a unique value. Before its block ends, it
MUST be consumed by `.wait()` or by transfer to a `TaskGroup`. Otherwise it's
a compile-time error. *(ADR 0096)*

**conc.task.6** — `h.wait()` blocks until the task completes and yields its
result. If the task trapped, `wait()` fails with a `Trapped` error carrying
the trap's diagnostic. *(ADR 0096, ADR 0047)*

```ail rules=conc.task.1,conc.task.6
struct Profile
    id int

fn load_profile(id int) Profile
    Profile(id)

fn main() !
    a := spawn load_profile(1)
    b := spawn load_profile(2)
    print(a.wait()?)
    print(b.wait()?)
```

The called function must be a declared function or associated function,
and no argument is passed `mut` (`conc.share.1`). If the task's function
returns `T ! E`, `wait()` returns `T ! E | Trapped`: the task's own failure
passes through. `Trapped` is a prelude error type,
`error struct Trapped { rule str, message str }`, where `message` starts
with the trap's `LINE:COL`. *(ADR 0187)*

A task handle's type is `Task[T]`. It can't be written in a signature or a
field, so a handle never leaves the block that spawned it; many handles whose
results are wanted go into a `TaskGroup[T]` (`conc.task.8`). It can only be
bound by `t := spawn f(x)`, waited, or added to a group, and it must be
consumed exactly once on every path: not inside only one branch of an `if`
or `match`, a loop, or an `else` (`conc.task.5`). A block left early (by
`return`, `?`, `break` or `continue`) still waits for its unconsumed tasks
(`conc.task.4`).

```ail error=conc.task.5
fn work(x int) int
    x

fn main()
    t := spawn work(1)
    print("never waited")
```

**conc.task.7** — If every unfinished task is blocked, so that none can
continue, the program traps (a deadlock) instead of hanging. The trap is
reported in a blocked task and names what it waits for. *(ADR 0186)*

**conc.task.8** — `TaskGroup()` makes an empty group. `g.add(t)` moves task
`t` into it; a `TaskGroup()` holds only tasks whose function returns nothing.
`g.wait_all()` waits for every task in the group and returns `! Trapped`,
failing with the first trapped task's `Trapped`. `TaskGroup[T]()` is a group
of tasks whose function returns `T`: `wait_all()` returns `List[T] ! Trapped`,
the results in the order the tasks were added. *(ADR 1204)* When the block that holds
the group ends, it waits for the tasks not yet waited; if one of them
trapped, that trap ends the owner's task too, since nothing else would
report it. *(ADR 0187)*

```ail rules=conc.task.4,conc.task.8
fn work(id int)
    print("work {id}")

fn main() !
    g := TaskGroup()
    for i in 1..=3
        g.add(spawn work(i))
    g.wait_all()?
```

```ail rules=conc.task.5,conc.task.8
fn square(n int) int
    n * n

fn main() !
    g := TaskGroup[int]()
    for i in 1..=4
        g.add(spawn square(i))
    print(g.wait_all()?)
```

**conc.task.10** — A field of a `unique` struct (`own.uniq.1`) may be a
`TaskGroup`; a field of a struct that isn't `unique`, or of an enum variant,
can't. The struct owns the group: tasks added through the field run until
the struct is released (`own.uniq.3`, `own.uniq.7`), and its release waits for
them as the end of a group's block does (`conc.task.8`), so every task is
waited exactly once (`conc.task.5`). The field can't be assigned. A task added
to a group whose owner was already released, through a copy of the group made
before, is waited at once. A field holds a `TaskGroup()` of tasks that return
nothing, not a `TaskGroup[T]`. *(ADR 0996, ADR 1204)*

```ail rules=conc.task.10
unique struct Pool
    tasks TaskGroup
    out Channel[int]

fn work(n int, out Channel[int])
    out.send(n * n)

fn Pool.start(mut self, n int)
    ch := self.out
    self.tasks.add(spawn work(n, ch))

fn main()
    mut p := Pool(tasks: TaskGroup(), out: Channel[int](4))
    p.start(3)
    p.start(4)
    a := p.out.recv() else 0
    b := p.out.recv() else 0
    print(a + b)
```

## Cancellation

**conc.cancel.1** — `t.cancel()` asks task `t` to stop, and `g.cancel()`
asks every task of group `g` to stop, including tasks added to it later.
A task started by a cancelled task starts cancelled. Cancelling doesn't end
a task and doesn't consume its handle: the scope still waits for it
(`conc.task.4`, `conc.task.5`). *(ADR 0321)*

**conc.cancel.2** — `task.cancelled()` returns whether the running task was
cancelled; it is `false` in the main task. `task.check()` returns
`! Cancelled`, failing with the prelude error
`error struct Cancelled "the task was cancelled"` once the task is
cancelled, so `task.check()?` stops a task at a point of its choosing. In a
cancelled task, blocking calls return early: `ch.recv()` returns `none` and
`for v in ch` ends, even if a value could arrive later, and a blocked call
is woken by the cancel; `time.sleep_ms` returns at once; and `proc.run`
fails with `ProcError` of kind `"cancelled"` without starting the process.
*(ADR 0321)*

**conc.cancel.3** — A cancel reaches the whole task tree: cancelling a task
also cancels every unfinished task it started, and the tasks those
started, as `conc.cancel.1` does for a group's tasks. A task records the
task that started it. Tasks the scheduler runs on their own threads
(`conc.par.10`) are pure, so a cancel has nothing to stop in them.
*(ADR 0351)*

**conc.cancel.4** — In a cancelled task, `ch.send(v)` on a full channel
doesn't wait for room: it drops `v` and returns. A send with room still
queues the value, and a send on a closed channel still traps
(`conc.share.4`). *(ADR 0351)*

```ail rules=conc.cancel.1,conc.cancel.4
fn worker(jobs Channel[int])
    for v in jobs
        print("got {v}")
    print("stopped: {task.cancelled()}")

fn main() !
    jobs := Channel[int](4)
    g := TaskGroup()
    g.add(spawn worker(jobs))
    jobs.send(1)
    g.cancel()
    g.wait_all()?
```

**conc.task.9** — [definitional] The order in which ready tasks run is unspecified. A
program's output MUST NOT depend on it except through `wait`, channels and
mutexes. *(ADR 0186)*

> **Note:** `--deterministic` and `--schedule` run one task at a time and in a fixed
> order, so the output is repeatable: a new task runs at once until it
> blocks or finishes, its spawner resumes next, and other ready tasks run
> first in, first out. Native code runs tasks in parallel (`conc.task.3`)
> unless the program is run with `--deterministic` or `--schedule`
> (`conc.sched.3`).

**conc.sched.1** — `ail test --schedule=random:SEED` (and `ail run` with the
same option) runs tasks on a random schedule decided by `SEED`: at each
`spawn` a coin picks whether the new task or its spawner goes first; at
every method call of a task, group, channel or mutex, every atomic
operation, `task.cancelled()` and `task.check()`, a coin picks whether the
running task hands the turn to another ready task; and the next task is
picked at random from the ready ones. Pooled loops and tasks
(`conc.par.8–10`) run on the scheduler instead. The same seed gives the
same interleaving on every run and target, so a failure can be replayed;
`--schedule=random` without a seed picks one. *(ADR 0322)*

**conc.sched.3** — `ail run --deterministic` (and `--schedule=fixed`) runs
tasks one at a time on the fixed schedule of the note under `conc.task.9`:
a new task runs at once until it blocks or finishes, its spawner resumes
next, a task that waits joins the line when what it waits for changes, and
other ready tasks run first in, first out. Pooled loops and tasks run on the
scheduler too (`conc.par.9`). Native programs pay for the mode only when
they ask for it: the scheduler is read from the process environment once,
when the first task starts, and a build for `--schedule=random` adds switch
points at atomic operations. *(ADRs 0186, 0207)*

**conc.sched.2** — When a test fails under a random schedule, `ail test`
prints `schedule seed SEED: replay with ...` after its summary, and with
`--json` adds `"schedule_seed":SEED` to the summary object. *(ADR 0322)*

## Communication

**conc.share.1** — A value passed to a spawned task, or sent between tasks, is
observably independent in the receiving task (`own.val.1`). *(ADR 0100)*

**conc.share.2** — Mutable state shared between tasks MUST be held in a
synchronised standard-library type (e.g. `Mutex[T]`, `Atomic[T]`).
*(ADR 0100, ADR 0063)*

**conc.share.3** — `Channel[T]` is bounded. Sending blocks while it's full,
and receiving blocks while it's empty. *(ADR 0100)*

**conc.share.4** — `Channel[T](capacity)` makes a channel that holds up to
`capacity` values; a capacity below 1 traps. `ch.send(v)` adds a copy of
`v`, and traps if the channel is closed. `ch.recv()` returns `T?`: the
oldest value, or `none` once the channel is closed and empty. `ch.close()`
closes it; closing it again traps. A channel, like a mutex, is a handle:
copies of it, and copies passed to tasks, are the same channel.
*(ADR 0188)*

```ail rules=conc.share.3,conc.share.4
fn produce(out Channel[int])
    for i in 1..=3
        out.send(i)
    out.close()

fn main() !
    ch := Channel[int](2)
    p := spawn produce(ch)
    while true
        v := ch.recv() else break
        print(v)
    p.wait()?
```

**conc.share.5** — `Mutex(v)` (or `Mutex[T](v)`) holds a value shared
between tasks. `m.get()` returns a copy, `m.set(v)` replaces it, and
`m.swap(v)` replaces it and returns the old value; each is one indivisible
step. For a read-modify-write, `x := m.lock()` waits until no other task
holds the lock, takes it, and returns a copy; `m.unlock(v)` stores `v` and
releases it. Other tasks' `get`, `set`, `swap` and `lock` wait while it is
held. `lock` while already holding it, `unlock` without holding it, and a
task that ends while holding it each trap. *(ADR 0189)*

```ail rules=conc.race.1,conc.share.2,conc.share.5
fn add(m Mutex[int], n int)
    x := m.lock()
    m.unlock(x + n)

fn main()
    m := Mutex(0)
    g := TaskGroup()
    for i in 1..=4
        g.add(spawn add(m, i))
    g.wait_all() else
        print("a task trapped")
    print(m.get())
```

**conc.share.6** — `with m as mut x` followed by a block holds mutex `m`
(a `Mutex[T]`) for the block: it waits and takes the lock as `m.lock()`
does, binds `x`, a `mut` local of type `T`, to a copy of the value, and runs
the block. When the block ends by any path (its end, `break`, `continue`,
`return` or `?`), the value of `x` is stored in `m` and the lock is
released. If the block traps, the lock is released and `m` keeps the value
it had before the block. The block MUST NOT name `m` again. A `with` whose
`m` isn't a `Mutex`, a `with m as x` without `mut`, and a use of `m` in the
block are compile-time errors. `with` is reserved (`lex.kw.1`). *(ADR 0301)*

```ail rules=conc.share.6
fn add(m Mutex[int], n int)
    with m as mut x
        x += n

fn main()
    m := Mutex(0)
    g := TaskGroup()
    for i in 1..=4
        g.add(spawn add(m, i))
    g.wait_all() else
        print("a task trapped")
    print(m.get())
```

```ail error=conc.share.6
fn main()
    m := Mutex(0)
    with m as x
        print(x)
```

**conc.share.7** — `for v in ch`, where `ch` is a `Channel[T]`, receives
values in turn as `ch.recv()` does and runs the body for each, ending when
the channel is closed and empty; a comprehension clause `for v in ch` does
the same (`expr.comp.2`). Like `recv`, it waits while the channel is empty
and open, so a channel nobody closes makes the loop wait for ever, which is
a deadlock once no other task can run (`conc.task.7`). A `break` leaves the
values not yet received in the channel. *(ADR 0300)*

```ail rules=conc.share.7
fn produce(out Channel[int])
    for i in 1..=3
        out.send(i)
    out.close()

fn main() !
    ch := Channel[int](2)
    p := spawn produce(ch)
    for v in ch
        print(v)
    p.wait()?
```

**conc.share.8** — `ch.try_recv()` returns `T?`: the oldest value if one is
waiting now, else `none`, at once, whether the channel is open or closed.
`ch.recv_for(ms)` waits as `recv` does for at most `ms` milliseconds (`ms`
below 0 counts as 0) and returns `none` when the time runs out or the channel
is closed and empty. Neither ever reports a deadlock (`conc.task.7`). Under
`--deterministic` no time passes: `recv_for` waits as `recv` does, and when
every other task is blocked, where `recv` would report a deadlock, its time
is up and it returns `none`. A cancelled
task's `recv_for` returns `none` as `recv` does (`conc.cancel.4`). *(ADR 0996)*

```ail rules=conc.share.8
fn main()
    ch := Channel[int](2)
    a := ch.try_recv() else -1
    ch.send(7)
    b := ch.try_recv() else -1
    c := ch.recv_for(10) else -1
    print("{a} {b} {c}")
```

**conc.select.1** — `task.select(chans List[Channel[T]]) int?` waits until
one of the channels has a value waiting and returns its index in `chans`;
when several do, the lowest index. It takes no value: `chans[i].recv()` then
returns the value at once. It returns `none` when every channel is closed
and empty, and in a cancelled task (`conc.cancel.2`). The channels may be
a child's `out`, `err` (`std.proc.5`) and ordinary channels of one element
type. Other tasks run while it waits; when none can, it is a deadlock
(`conc.task.7`) unless a child's stream could still wake it. *(ADR 0427)*

```ail norun rules=conc.select.1
# norun: the order of the two streams depends on how the child's writes race, so
# the output differs between runs; the example is checked, not compared.
use std.proc
use std.task

fn main() !
    c := proc.start("sh", ["-c", "echo a; echo b >&2"])?
    mut open := true
    while open
        i := task.select([c.out, c.err])
        if i == none
            open = false
        elif i == 0
            print("out {c.out.recv()}")
        else
            print("err {c.err.recv()}")
    proc.wait(move c)?
```

**conc.select.2** — A `select` statement waits on several channel operations
and runs the block of the one that happens. It is `select` and an indented
block of arms. An arm is a head, `=>`, and an indented block or one statement
on the same line (as a `match` arm, `expr.match.9`). The heads are:
`v := ch.recv()` (a receive arm), `ch.send(x)` (a send arm), `after ms` and
`closed`. A `select` has at least one receive or send arm. `select` is
reserved (`lex.kw.1`); `after` and `closed` are words only at the start of an
arm. *(ADR 1102)*

**conc.select.3** — A `select` first evaluates, once and in source order,
every arm's channel, each send arm's value and the `after` time. Then it waits
until an arm can go ahead and runs the first arm in source order that can: a
receive arm when its channel has a value waiting, which it takes (the oldest)
and binds to the arm's name for the arm's block, as a `T`, not a `T?`; a send
arm when its channel has room, which queues its value. Exactly one arm's
operation happens; the values of the other send arms are dropped. The
channels may have different element types. The name of a receive arm can't
be reassigned and exists only in its block; `_ := ch.recv()` drops the value.
A send arm on a closed channel traps as `send` does (`conc.share.4`). Other
tasks run while it waits. *(ADR 1102)*

```ail rules=conc.select.2,conc.select.3,conc.select.5
fn numbers(out Channel[int])
    for i in 1..=3
        out.send(i)
    out.close()

fn words(out Channel[str])
    out.send("done")
    out.close()

fn main() !
    nums := Channel[int](4)
    msgs := Channel[str](4)
    a := spawn numbers(nums)
    b := spawn words(msgs)
    mut total := 0
    mut last := ""
    while true
        select
            n := nums.recv() =>
                total += n
            m := msgs.recv() =>
                last = m
            closed => break
    a.wait()?
    b.wait()?
    print("{total} {last}")
```

```ail rules=conc.select.3,conc.select.4
fn main()
    small := Channel[int](1)
    big := Channel[int](4)
    for i in 1..=3
        select
            small.send(i) => print("small {i}")
            big.send(i) => print("big {i}")
    select
        small.send(4) => print("sent")
        after 0 => print("small is full")
```

```ail error=conc.select.2
fn main()
    ch := Channel[int](1)
    select
        ch.recv() => print("a value")
        closed => print("closed")
```

**conc.select.4** — An `after ms =>` arm runs when `ms` milliseconds (an
`int`; below 0 counts as 0) pass with no other arm ready; `after 0` runs at
once when no other arm is ready. A `select` has at most one. A `select` with
an `after` arm is never a deadlock (`conc.task.7`). Under `--deterministic` no
time passes: `after 0` is as above, and a longer `after` runs once every other
task is blocked, as `recv_for` does (`conc.share.8`). *(ADR 1102)*

```ail rules=conc.select.4
fn main()
    ch := Channel[int](1)
    select
        v := ch.recv() =>
            print("got {v}")
        after 10 =>
            print("nothing yet")
        closed =>
            print("closed")
```

**conc.select.5** — A `closed =>` arm runs when every channel of the
`select`'s receive arms is closed and empty, so a loop around a `select` can
end. Until then a closed, empty channel's arm never runs and the other arms
go on. A `select` with a receive arm MUST have exactly one `closed` arm, and
one without a receive arm MUST NOT have one. *(ADR 1102)*

```ail error=conc.select.5
fn main()
    ch := Channel[int](1)
    select
        v := ch.recv() => print(v)
        after 0 => print("none yet")
```

**conc.select.6** — Without an `after` arm, a `select` that waits when no other
task can run is a deadlock (`conc.task.7`). In a cancelled task a `select`
never waits (`conc.cancel.2`): when no arm can go ahead it runs its `closed`
arm, else its `after` arm, else no arm. A `select` with a `closed` or an
`after` arm always runs an arm, so it leaves its block when every arm does
(`expr.block.3`). `task.select` (`conc.select.1`) stays for a list of
channels of one type. *(ADR 1102)*

```ail rules=conc.select.6
fn worker(jobs Channel[int])
    while true
        select
            j := jobs.recv() => print("job {j}")
            closed => break
    print("stopped: {task.cancelled()}")

fn main() !
    jobs := Channel[int](4)
    g := TaskGroup()
    g.add(spawn worker(jobs))
    g.cancel()
    g.wait_all()?
```

## Parallel execution

**conc.par.1** — An implementation MAY run the iterations of an independent
loop (`conc.par.2`) concurrently, on any number of cores and vector lanes.
The observable result MUST be identical to running the iterations in order:
the same values, bit for bit (floats included), and, if iterations trap,
the trap of the lowest-index trapping iteration. *(ADR 0236)*

**conc.par.2** — A `for` loop's iterations are *independent* when its body:
(1) has no `io`, `unsafe` or `spawn` effect and no capability, through every
call (`eff.infer.3`); (2) doesn't leave the loop early: no `break` of this
loop, `return`, `?`, or `else return`; (3) calls no method of a task,
`TaskGroup`, `Channel` or `Mutex` handle, receives from no channel with
`for v in ch` and holds no mutex with `with`, directly or through the
functions it calls; and
(4) writes only places that `conc.par.3` or `conc.par.4` allow, or places
rooted at names declared in the body (including the loop variable). Calls
with a `mut` argument and methods with `mut self` write their place. A
write through a `ref` declared outside the body (`c.n = i`) is shared by every
iteration and is an error in a `par for`. *(ADR 0236, ADR 0358)*

**conc.par.3** — In a loop over a range, a write to `r[i]` (optionally
followed by more fields or indexes), where `r` is declared outside the body,
`i` is the loop variable, and the steps before `[i]` are fields, literals or
names declared outside the body, is allowed when every write to `r` in the
body uses the same steps up to `[i]` and every read of `r` either starts with
those steps or only takes a `len()`. *(ADR 0236)*

**conc.par.4** — A variable declared outside the body as an `int` (a
parameter of type `int`, or bound to an integer literal) may be written by
`x += e` when the body neither reads it nor writes it otherwise. Such
accumulators combine in a fixed order; a sum overflows, and traps, exactly
where the sequential sum would. A float accumulator is not allowed: float
`+` isn't associative. *(ADR 0236)*

```ail rules=conc.par.2,conc.par.3,conc.par.4
fn main()
    mut squares := [0, 0, 0, 0]
    mut total := 0
    for i in 0..4
        squares[i] = i * i
        total += i
    print("{squares} {total}")
```

**conc.par.5** — `par for x in e` means `for x in e` and asserts that the
iterations are independent. The obligations of `conc.par.2–4` that the
checker can decide still apply, and a `par for` that provably fails one is a
compile-time error. The assertion covers only what the checker can't
decide: that writes `r[e]` with another index pick distinct elements, that
reads of other elements of `r` don't meet those writes, and that an
accumulator is an `int`. A checked build MAY trap when two iterations write
the same element. `par` is reserved (`lex.kw.1`) and is followed only by
`for`. *(ADR 0237)*

```ail error=conc.par.5
fn main()
    xs := [1, 2, 3]
    par for x in xs
        print(x)
```

**conc.par.6** — `ail effects --parallel FILE` prints one line per `for`
loop in source order: `FILE:LINE:COL ID for VAR: VERDICT (RULE: REASON)`,
with ` at FILE:LINE:COL` after the reason when it points at a write or exit
inside the body. `ID` is the enclosing function's stable id (`eff.infer.6`),
`par for` replaces `for` for an annotated loop, and `VERDICT` is
`parallel`, `asserted` (a `par for` that holds only by its assertion) or
`sequential`. With `--json` it prints one object
`{"ail_api":1,"loops":[{"at","fn","var","par","verdict","rule","reason","reason_at"},...]}`. *(ADR 0236)*

**conc.par.7** — Scheduling is the implementation's choice and MUST NOT
change results (`conc.par.1`). The native runtime uses one work-stealing
pool for tasks and parallel loops, runs a loop with too little work
sequentially, and finishes a parallel loop before the statement after it
starts, so it is scope-bound like `conc.task.4`. *(ADR 0238)*

**conc.par.8** — A `par for` loop that `conc.par.5` accepts runs on the
thread pool. So does every loop `conc.par.2–4` prove independent, unless
`AIL_PARALLEL=0` or the run is deterministic (`conc.par.9`). A loop on the pool
must also write only list elements, `int` accumulators, each accumulator by
one `+=` outside nested loops, and lists it builds (`conc.par.16`); any other loop, a loop that captures a handle
or a `ref`, and a loop with fewer than twice as many iterations as the workers
it would use (`conc.par.17`), runs in order. Its body may read fields, `match`, bind with
`if x :=`, `x := e else ...` and `a, b := e`, and make tuples, maps, sets and comprehensions; it
may not use `?`, read a field as `self.f`, or write a field of a variable it doesn't declare.
A loop the judge keeps in order may still run its head on the pool (`conc.par.20`).
*(ADR 0272, ADR 0275, ADR 0911, ADR 0946, ADR 1143)*

**conc.par.14** — A loop the compiler chose (not a written `par for`) runs
in order unless its iterations together cost at least 65536 operations, where
one iteration costs the operations of its body (nested loops count 16 times,
a `/` or `%` 16, or 4 and 6 by an integer literal; on x86-64 a `/` by anything else counts
32 and a `%` 96 in a loop whose work the count has in full,
a call 8 plus the cost of the function it names, found the same way; a function that
reaches itself, or has a loop, counts at least 4096). An iteration of at least 4096 operations sends any loop of two or more
iterations to the pool, whatever the worker count. A written `par for` whose body calls a
function counts as that heavy, so it goes to the pool from two iterations. A loop the compiler chose whose work the count has in full
needs 1048576 operations instead, however heavy one iteration is: its body, and every
function it reaches, has no loop, no recursion, no call of a method other than a
constant-time read-only one, no call of a function whose body the count does not have, and makes no
string, list, map or set. Such operations take a fraction of a nanosecond each, and
starting a region costs tens of microseconds. When the body of such a loop over a range is
one store `c[i] = e` to a list of floats, with `e` made of `+`, `-`, `*`, `/`, elements `x[i]`, names and number
literals, the loop needs 16777216 operations: in order it runs several elements at a time
(`native.vec.4`), and a chunk on the pool does not. The result and the trap are
the sequential ones either way. Such a loop may sit inside a loop that runs in order
(a `for` the compiler didn't choose, or a `while`): each run of the outer body tests the
inner loop's work again, and a loop inside a loop on the pool runs in order in its chunk.
A loop that builds a list by `push` and whose work is only a called function's loop or
recursion, inside such a loop, needs at least 16384 iterations: each run of the outer body
pays a dispatch and a gather, so the estimate of 4096 for a callee's loop is not trusted there.
A loop that builds a list by `push` (`conc.par.16`) runs in order when one iteration costs
fewer than 16 operations, and when it costs fewer than 256 and an element of a built list is
not a number (`int`, `float`, `bool`, `char`, a byte or a sized number): the chunks' lists are
copied once more when they are joined, and making structs, lists and texts is bound by memory,
which the pool does not speed up.
*(ADR 0911, ADR 0987, ADR 1155, ADR 1157)*

```ail rules=conc.par.14
fn main()
    mut grid := List[int]()
    for _ in 0..1000
        grid.push(1)
    mut next := grid
    mut step := 0
    while step < 3
        for i in 0..1000
            next[i] = grid[i] + grid[(i + 1) % 1000]
        grid = next
        step += 1
    print("{grid[0]} {grid[999]}")
```

**conc.par.13** — A written `par for` that the native compiler runs in order, because its
body has a construct the chunked version doesn't take (a `?`, a constructor with a path, a method on a
captured list, an assignment to a field of `self`, and so on), prints
`FILE:LINE:COL: warning[conc.par.13]: `par for V` runs in order: REASON` on standard error
when the program is built, and `ail check` warns the same way. The result is the sequential one. A loop the compiler chose
says nothing. *(ADR 0911, ADR 1143)*

**conc.par.9** — The pool has at most one thread per hardware thread, up to 64,
unless the environment variable `AIL_THREADS` sets the number. A region
uses only as many of them as its work repays (`conc.par.17`). With one
thread, or under `--deterministic`, every loop and task runs as if there
were no pool. With `AIL_PAR_LOG` set, the run says on standard error
how each pooled loop and task ran. *(ADR 0272, ADR 0275)*

**conc.par.15** — A run of two or more adjacent `name := f(args)` statements, where every `f` is
a free function with no impure effect and is heavy (it has a loop, is recursive, or reaches a
function with a loop), no argument is `mut`, every argument is a name, a literal, or
`name + k` / `name - k` with a literal `k`, and no argument mentions a name the run defines
earlier, may run as a fork-join: the calls after the first start as tasks while the pool has
room (fewer than twice as many live tasks as workers), the first runs on the calling thread,
and the results are taken in source order. A task that traps is run again in order on the
calling thread, so the trap is the sequential one (`conc.par.1`). A function that calls itself
in such a run has a second copy for the calls made in order, which never forks; results and
traps are the same. `AIL_PARALLEL=0` and
`--deterministic` run the calls one after the other. `ail effects --parallel` lists the runs
that qualify. *(ADR 0914)*

**conc.par.17** — The native pool is sized to the machine, not to a fixed
number: up to 64 threads, one per hardware thread. A parallel loop or tensor
kernel uses one worker per 65536 operations of its work (the cost model of
`conc.par.14`), at least 2 and at most the pool's size, split into four
chunks per worker, so a loop that scales runs on 16 cores rather than 8 and a
small one stays on few. A body with a loop, or a call of a function that has one
or reaches itself, has work the count only bounds from below: such a region uses
the whole pool. `AIL_THREADS` still sets the pool's size. With
`AIL_PAR_LOG` set, each region prints `par: N iterations of about W operations
-> K of T workers, C chunks (conc.par.17)` on standard error. Results never
depend on the number of workers (`conc.par.1`). *(ADR 0946)*

**conc.par.18** — `task.threads()` returns the pool's cap: at the start
`AIL_THREADS`, or the cores the process may use (`conc.par.17`).
`task.set_threads(n)` sets the cap to `n`, clamped to 1 to 64, from the next
parallel loop, tensor kernel or task that starts. Workers already past the cap
finish what they run and take nothing more. A cap of 1 runs every loop in
order (`conc.par.9`). Calling it while a parallel loop runs traps.
`set_threads` has the `io` effect, so no independent loop or pure function
calls it. Results never depend on the cap (`conc.par.1`). *(ADR 1018)*

```ail rules=conc.par.18
fn main()
    task.set_threads(2)
    print(task.threads())
```

**conc.par.19** — On Windows a waiting thread polls, so waking the workers and
seeing them finish costs a parallel region about two timer ticks (31 ms). After
two regions in a row that each ended within 40 ms, a loop the compiler chose
runs in order; a later region that takes longer (a written `par for`, a tensor
kernel) lets them go to the pool again. A written `par for` always goes to the
pool. Results never depend on it (`conc.par.1`). *(ADR 1115)*

```ail rules=conc.par.19
fn main()
    mut total := 0
    for _ in 0..4
        mut out := List[int]()
        for i in 0..20000
            out.push(i * i % 7)
        total += out.len()
    print(total)
```

**conc.par.20** — A `for` loop over `lo..hi` or a list variable that `conc.par.2–4` keep in
order, and that no `for` or `while` of its function encloses, *splits* when its leading `x := e` and `a, b := e` statements (with no `else` that
leaves), its *head*, are independent on their own by those rules and are heavy (a loop, or a
call of a free function that has a loop or calls itself, as in `conc.par.15`), the rest of the body, its *tail*, uses one to four of the names the head binds, writes
nothing the head reads (by assignment, a method call on it, or a `mut` argument) nor the list
iterated, and has no `break`, `return` or `?` that leaves the loop. Not split: a loop in a
`mut self` method, and, in a program that has a `ref`, a loop whose tail calls a function. A
split loop runs in blocks of 4096 iterations: the head of each block on the pool, keeping the
values the tail uses in order, then the tail in order over them. A block too small to repay
the pool (`conc.par.14`), or run with one worker, runs as written; one where the head traps
runs in order as written too, so the output,
the result and the trap are the sequential ones (`conc.par.1`, `conc.par.11`). `AIL_PARALLEL=0`
and `--deterministic` run every block in order. *(ADR 1143)*

```ail rules=conc.par.20
fn steps(start int) int
    mut n := start
    mut c := 0
    while n != 1
        n = if n % 2 == 0 then n / 2 else 3 * n + 1
        c += 1
    c

fn main()
    mut best := 1
    mut best_steps := 0
    for i in 1..10000
        s := steps(i)
        if s > best_steps
            best = i
            best_steps = s
    print("{best} {best_steps}")
```

**conc.par.16** — A loop *builds* an outer list `out` when the only mention of
`out` in its body is one statement `out.push(e)` at the top of the body (not
under `if`, `match` or a nested loop), the `push` is the built-in one, and no
`continue` outside nested loops can skip it. A built list does not make the
loop's iterations interfere (`conc.par.2`): iteration `k` pushes element `k`.
On the pool each chunk pushes into a list of its own; when no chunk traps, the
chunks' lists are appended to `out` in chunk order, so `out` is the sequential
list (`conc.par.1`); otherwise `out` is untouched and the loop runs in order
(`conc.par.11`). A `[e par for x in xs]` comprehension with no filter builds
its list this way. *(ADR 0938)*

```ail rules=conc.par.16
fn cube(i int) int
    i * i * i

fn main()
    mut out := List[int]()
    for i in 0..1000
        out.push(cube(i))
    print("{out.len()} {out[10]} {out[999]}")
```

**conc.par.10** — A task whose function has no `io`, `unsafe`, `spawn` or
capability effect and calls no method of a handle, directly or through the
functions it calls, and whose arguments hold no handle, runs on its own
thread outside the one-at-a-time scheduler (`conc.task.3`); `wait`, a
group's `wait_all` and the end of its block join it. Any other task runs on
the scheduler. A pooled task that returns a handle traps. *(ADR 0273)*

**conc.par.11** — If an iteration on the pool traps, a captured value holds a
handle, an accumulator doesn't hold an `int`, or the running total of an
accumulator would leave `int`, the loop discards what the pool computed and
runs in order from its first iteration. Because the body has no effect, the
result, and the trap if any, is the sequential one (`conc.par.1`). A body on
the pool can't block on another task: it has no handle. *(ADR 0274)*

```ail rules=conc.par.1,conc.par.7,conc.par.8,conc.par.11
fn main()
    mut squares := List[int]()
    for _ in 0..64
        squares.push(0)
    mut total := 0
    par for i in 0..64
        squares[i] = i * i
        total += i
    print("{squares[63]} {total}")
```

**conc.par.12** — `[e par for x in xs if c]` means `[e for x in xs if c]`
(`expr.comp.1`) and computes the elements on the thread pool when the
`par for` loop of the same body would run there (`conc.par.8–9`); the list
is in source order and identical to the sequential one (`conc.par.1`). The
comprehension MUST be a list comprehension with that one clause. Its
element and filter MUST meet `conc.par.2` (1)–(3) and write nothing outside
themselves; one that provably doesn't, like an element that prints or a
clause over a channel, is a compile-time error. A trap, or a value the pool
can't copy, makes it compute the elements in order (`conc.par.11`). A short
list, one thread, or `--deterministic`, computes them in order.
*(ADR 0302)*

```ail rules=conc.par.12
fn square(x int) int
    x * x

fn main()
    xs := [n for n in 1..=64]
    squares := [square(x) par for x in xs]
    print("{squares[0]} {squares[63]}")
```

```ail error=conc.par.12
fn show(x int) int
    print(x)
    x

fn main()
    ys := [show(x) par for x in [1, 2, 3]]
    print(ys)
```

## Runnable examples for the rules above

The rules without an example in the sections above are checked here. Each
block runs natively; the `norun` ones show a trap or a command line
the gate can't run.

```ail rules=conc.atomic.1,conc.atomic.4
fn main()
    a := Atomic(5)
    print(a.cas(5, 7))
    print(a.load())
    a.store(9)
    print(a.load())
    print(a.cas(5, 1))
```

```ail rules=conc.task.2,conc.task.3,conc.task.9
fn wait_for(ch Channel[int]) int
    v := ch.recv() else 0
    v * 10

fn main() !
    ch := Channel[int](1)
    t := spawn wait_for(ch)
    ch.send(4)
    print(t.wait()?)
```

```ail norun rules=conc.task.7
fn main() !
    ch := Channel[int](1)
    print(ch.recv() else 0)
```

```ail rules=conc.par.10,conc.par.9
fn busy(n int) int
    mut total := 0
    for i in 0..n
        total += i
    total

fn main() !
    a := spawn busy(10)
    b := spawn busy(20)
    print(a.wait()? + b.wait()?)
```

```ail rules=conc.cancel.2
fn work(ch Channel[int]) int ! Cancelled
    v := ch.recv() else 0
    task.check()?
    v

fn main()
    ch := Channel[int](1)
    t := spawn work(ch)
    t.cancel()
    ch.send(5)
    print(t.wait() else -1)
```

```ail rules=conc.cancel.3
fn inner(ch Channel[int])
    v := ch.recv() else 0
    print("inner cancelled: {task.cancelled()}")

fn outer(ch Channel[int])
    t := spawn inner(ch)
    t.wait() else return

fn main() !
    ch := Channel[int](1)
    g := TaskGroup()
    g.add(spawn outer(ch))
    g.cancel()
    g.wait_all()?
    print("done")
```

```ail rules=conc.share.1
fn bump(xs List[int]) int
    mut xs2 := xs
    xs2.push(9)
    xs2.len()

fn main() !
    xs := [1, 2]
    t := spawn bump(xs)
    print(t.wait()?)
    print(xs.len())
```

```ail tool=effects,--parallel,FILE expect=parallel rules=conc.par.6
fn main()
    mut total := 0
    for i in 0..10
        total += i
    par for i in 0..10
        total += i
    print(total)
```

```ail norun rules=conc.mm.1
fn main()
    a := Atomic(0)
    a.store(1, order: Release)
    print(a.load(order: Acquire))
```

```ail norun rules=conc.sched.1,conc.sched.2,conc.sched.3
# norun: the gate runs no test. Run this with `ail test --schedule=random:7`;
# a failing run prints the seed (conc.sched.2). `ail run --deterministic` runs the
# fixed schedule (conc.sched.3).
fn take(ch Channel[int]) int
    ch.recv() else 0

fn main() !
    ch := Channel[int](1)
    t := spawn take(ch)
    ch.send(4)
    print(t.wait()?)
```
