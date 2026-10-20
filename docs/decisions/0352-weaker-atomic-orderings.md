# 0352. Weaker atomic orderings as a trailing `order:` argument

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 11 Concurrency
- **Spec rules:** `conc.atomic.7` (provides `conc.atomic.2`)
- **Resolves:** OQ-097 part (1); extends ADR 0320

## Context

`conc.atomic.2` (ADR 0025) requires acquire, release, acquire-release and
relaxed orderings. ADR 0320's `Atomic[int]` and `Atomic[bool]` had only
SeqCst. SeqCst stays the default (`conc.atomic.1`); the question is how to
spell the rare weaker case.

## Decision

A trailing named argument: `a.load(order: Acquire)`,
`a.store(v, order: Release)`, `a.add(n, order: Relaxed)`,
`a.cas(old, new, order: AcqRel)`. The prelude declares
`enum AtomicOrder` with `SeqCst`, `Acquire`, `Release`, `AcqRel` and
`Relaxed`, so the bare names work like `Less` and `Greater`.

The ordering must be written as one of those names, never computed, so the
checker always knows it and rejects one that makes no sense for the method
(`conc.atomic.7`): a load can't release and a store can't acquire. A failed
`cas` reads with the read half of its ordering. The interpreter maps the
name to the hardware ordering, so the bootstrap's `par for` counter really
runs relaxed.

Decided by criterion 1 (tokens): code that doesn't use a weaker ordering
pays nothing, and the cost lands only on the rare call that does. Criterion
2 agrees: one method name per operation, and the checker's diagnostic names
the orderings the method allows.

## Options considered

### (a) Trailing named argument (chosen)
- **Pros:** zero cost for SeqCst code; four method names stay four; the
  ordering reads as what it is, a modifier.
- **Cons:** the only named argument on a built-in method so far.

### (b) Separate methods, `load_relaxed`, `add_acquire` (rejected)
About the same tokens per use, but 4 methods × up to 5 orderings is up to
20 names an agent must recall, and the invalid pairs simply don't exist,
so a typo gets "no method" instead of an explanation.

### (c) Wait for the native backend (rejected)
The interpreter already runs real hardware atomics shared across threads
(ADR 0320), so orderings are meaningful today, and leaving `conc.atomic.2`
unprovided keeps a MUST unmet.

### Ordering as any expression of type `AtomicOrder` (rejected)
Would allow choosing at run time, which no realistic code needs, and would
move the method check from the checker to a trap.

## Consequences

- Speed: a relaxed `add` costs the same in the interpreter as SeqCst on
  x86-64 (both are `lock xadd`), and saves a barrier on ARM64; the
  interpreter's per-call overhead dominates either way (see the PR's
  perfbench numbers).
- The SHOULD in `conc.atomic.2`'s note (report orderings that could be
  weakened) remains future work.
