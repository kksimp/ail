# 11. Concurrency

Chapter key: `conc`

> **Status:** Drafted (partial). Specified: memory-model foundations, the task
> model, scope-bound tasks, and communication. Not yet designed: cancellation,
> `select`, parallel loops, and the formal model.

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
a := spawn load_profile(1)
b := spawn load_profile(2)
print(a.wait()?)
print(b.wait()?)
```

## Communication

**conc.share.1** — A value passed to a spawned task, or sent between tasks, is
observably independent in the receiving task (`own.val.1`). *(ADR 0100)*

**conc.share.2** — Mutable state shared between tasks MUST be held in a
synchronised standard-library type (e.g. `Mutex[T]`, `Atomic[T]`).
*(ADR 0100, ADR 0063)*

**conc.share.3** — `Channel[T]` is bounded. Sending blocks while it's full,
and receiving blocks while it's empty. *(ADR 0100)*
