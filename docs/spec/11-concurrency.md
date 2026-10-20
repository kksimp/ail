# 11. Concurrency

Chapter key: `conc`

> **Status:** Drafted (partial). Specified: memory-model foundations, the task
> model, scope-bound tasks, communication through `Channel[T]` and
> `Mutex[T]`, and parallel loops. Not yet designed: cancellation, `select`,
> `Atomic[T]`, and the formal model.

## Memory model

**conc.mm.1** — The AIL memory model is defined on an abstract,
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

```ail
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
field, so a handle never leaves the block that spawned it. It can only be
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
`t` into it; a group holds only tasks whose function returns nothing.
`g.wait_all()` waits for every task in the group and returns `! Trapped`,
failing with the first trapped task's `Trapped`. When the block that holds
the group ends, it waits for the tasks not yet waited; if one of them
trapped, that trap ends the owner's task too, since nothing else would
report it. *(ADR 0187)*

```ail
fn work(id int)
    print("work {id}")

fn main() !
    g := TaskGroup()
    for i in 1..=3
        g.add(spawn work(i))
    g.wait_all()?
```

**conc.task.9** — The order in which ready tasks run is unspecified. A
program's output MUST NOT depend on it except through `wait`, channels and
mutexes. *(ADR 0186)*

> **Note:** the bootstrap interpreter runs one task at a time and in a fixed
> order, so its output is repeatable: a new task runs at once until it
> blocks or finishes, its spawner resumes next, and other ready tasks run
> first in, first out. Native code runs tasks in parallel (`conc.task.3`).

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

```ail
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

```ail
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
`TaskGroup`, `Channel` or `Mutex` handle, directly or through the functions
it calls; and
(4) writes only places that `conc.par.3` or `conc.par.4` allow, or places
rooted at names declared in the body (including the loop variable). Calls
with a `mut` argument and methods with `mut self` write their place. *(ADR 0236)*

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

```ail
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
thread pool. With `ail run --parallel`, so does every loop `conc.par.2–4`
prove independent. A loop on the pool must also write only list elements
and `int` accumulators, each accumulator by one `+=` outside nested loops;
any other loop, and a loop with fewer than twice as many iterations as the
pool has threads, runs in order. *(ADR 0272, ADR 0275)*

**conc.par.9** — The pool has one thread per hardware thread, at most 8,
unless the environment variable `AIL_THREADS` sets the number. With one
thread, or under `--deterministic`, every loop and task runs as if there
were no pool. With `AIL_PAR_LOG` set, the interpreter says on standard error
how each pooled loop and task ran. *(ADR 0272, ADR 0275)*

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

```ail
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

