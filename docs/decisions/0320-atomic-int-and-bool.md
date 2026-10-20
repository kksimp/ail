# 0320. `Atomic[int]` and `Atomic[bool]`: shared cells for tasks and `par for`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 11 Concurrency
- **Spec rules:** `conc.atomic.3`–`conc.atomic.6`, `conc.share.2`

## Context

ADR 0025 settled that atomics default to SeqCst, and `conc.share.2` names
`Atomic[T]` as a synchronised type, but nothing provided one. A counter
shared by tasks needed a `Mutex` (a scheduler round trip per update), and a
`par for` body couldn't touch shared state at all: handles and `ref` cells
make a loop run in order (`conc.par.8`). The designer wants AIL to use four
cores by default, and a hit counter or progress total is the commonest
shared state in agent tools.

Measured on a 4-core Linux x86-64 container (best of 3, release binary),
four tasks adding 250,000 values each:

| Version | Tokens (mean of 5 tokenizers) | Time |
|---|---|---|
| `Atomic[int]`, `hits.add(i % 7)` | 90.6 | 0.21 s |
| `Mutex[int]`, `with hits as mut x` / `x += i % 7` | 97.0 (+7%) | 0.29 s (+36%) |
| `Mutex[int]`, `lock` / `unlock` | 100.2 (+11%) | 0.41 s (+96%) |

`perfbench`'s `parsum` with its `total += s` accumulator replaced by
`total.add(s)` runs in 0.535 s against 0.529 s: an atomic add costs about
the same as the pool's accumulator.

## Options considered

### Option A — a prelude handle with `load`, `store`, `add`, `cas` (chosen)
`Atomic(v)` or `Atomic[T](v)` with `T` one of `int`, `bool`. The cell is a
real hardware atomic (`AtomicI64`) behind an `Arc`, so it crosses into
pooled tasks and `par for` workers unchanged. `add` returns the old value
and traps on overflow, like `+`.
- **Pros:** fewest tokens; tasks that only use atomics stay pure enough for
  the pool (`conc.par.10`), so they run on four cores; the same type in
  the interpreter and native code.
- **Cons:** `par for` loops that use an atomic can't fall back to running
  in order after a trap (`conc.par.11`), so they report the trap directly.

### Option B — let `par for` write through `ref` cells
- **Pros:** no new type.
- **Cons:** a `ref` write is a read-modify-write, a data race on four
  cores (`conc.race.1`); would need a lock per cell anyway.

### Option C — `Atomic[T]` for any `T`, built on a lock
- **Pros:** general.
- **Cons:** duplicates `Mutex`; hides a lock behind a name that promises a
  hardware atomic; slower.

## Decision

Option A. `add` exists only on `Atomic[int]`. Atomic method calls aren't
handle calls: a function that calls them can still run as a pooled task,
and a `for` loop that reaches one is not proven independent
(`conc.atomic.6`) but may be asserted with `par for`. Every operation is a
switch point under a random schedule (ADR 0322).

## Rationale

Tokens decide (criterion 1): 7–11% fewer than the mutex forms on a shared
counter. Speed agrees (criterion 4): 1.4–2× faster, because the tasks run
on the pool instead of one at a time. AI understanding (criterion 2): one
obvious type for a shared number, named as in Rust, C++ and Go.

## Consequences

- `add` returns the value from before, as hardware fetch-add, Rust and C++
  do. Go's `atomic.Add` returns the new value; the spec rule says which.
- An atomic `par for` that traps leaves the updates of iterations that ran;
  the program can't observe them unless it handles the trap's caller.
- Weaker orderings (`conc.atomic.2`) are OQ-097.

## Rejected alternatives

- `ref` writes in `par for`: a data race on real cores.
- A lock-based `Atomic[T]` for every `T`: `Mutex` already does that, and
  more slowly than the name suggests.
- `add` returning the new value: differs from the hardware operation and
  from Rust and C++, which agents copy most.
