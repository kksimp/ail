# 11. Concurrency

Chapter key: `conc`

> **Status:** Drafted (partial). Only the memory-model foundations are
> specified so far. Tasks, threads, async, channels, the data-race-freedom
> mechanism, and the formal model are not yet designed.

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
