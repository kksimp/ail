# 0600. Native atomics: one cell, sequentially consistent, LSE on ARM64

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 11 — Concurrency
- **Spec rules:** [conc.atomic.1–7](../spec/11-concurrency.md), [native.task.1](../spec/23-native.md)
- **Builds on:** [ADR 0025](0025-memory-model-atomics.md), [ADR 0320](0320-atomic-int-and-bool.md), [ADR 0352](0352-weaker-atomic-orderings.md), [ADR 0599](0599-native-threads-and-task-pool.md)

## Decision

1. **`Atomic[int]` and `Atomic[bool]`** are a pointer to an 8-byte cell
   (`Ty.Atomic` in MIR). A copy of the handle is the same cell
   (conc.atomic.3), so it passes to `spawn` and `par for` as an argument.
2. **Six primitives**, usable by the runtime and by lowered code: `atomic_load`,
   `atomic_store`, `atomic_xchg`, `atomic_add`, `atomic_cas`, `sp`. On x86-64
   a load is `mov`, a store and an exchange are `xchg`, `add` is `lock xadd`
   and `cas` is `lock cmpxchg`. On ARM64 they are `ldar`, `stlr`, `swpal`,
   `ldaddal` and `casal` (ARMv8.1 LSE, inside the ARMv8.2-A baseline of ADR 0006).
3. **Every operation is sequentially consistent.** An `order:` argument is
   accepted and ignored: a stronger ordering than asked for is correct
   (conc.atomic.2), and an implementation must not weaken one on its own
   (conc.atomic.2's note), so `Relaxed` costs what `SeqCst` costs until a
   measurement asks otherwise.
4. **`add` is a compare-and-swap loop**, not `xadd`, because an add that would
   leave `int` must trap and leave the cell unchanged (conc.atomic.5): the
   checked sum is made before the swap, and a failed swap reads again. The
   trap is `conc.atomic.5` with the interpreter's text, `atomic add overflows:
   A + B is outside int`.
5. **`print` of an atomic** writes `Atomic(7)` or `Atomic(true)` as the interpreter does.

## Rejected alternatives

- **`lock xadd` and undo on overflow:** another thread could read the
  overflowed value, which breaks "indivisible".
- **Per-ordering instructions:** the interpreter can't observe a weaker
  ordering, so the differential tests couldn't catch a mistake in one.
- **`Atomic` as an `int` in MIR:** a handle would print as a number and take
  `int` methods; a variant keeps the type visible to every pass.

## Consequences

- An `add` under contention retries; a program that hammers one cell from
  four threads is slower than with `xadd`. Measured when it matters.
