# 11. Concurrency

Chapter key: `conc`

> **Status:** Drafted (partial). Specified: memory-model foundations, the task
> model, scope-bound tasks, and communication through `Channel[T]` and
> `Mutex[T]`. Not yet designed: cancellation, `select`, parallel loops,
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
