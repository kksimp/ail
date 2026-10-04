# 0025. Abstract weakly-ordered memory model; atomics default to sequential consistency

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 11 — Concurrency; 5 — Memory model
- **Spec rules:** [conc.mm.1, conc.race.1, conc.atomic.1, conc.atomic.2](../spec/11-concurrency.md)
- **Resolves:** OQ-002

## Context

ARM64 is weakly ordered, while x86-64 is TSO (total store order). If the
language's memory model were based on either chip, it would be distorted:
x86-based semantics force barriers on ARM, and ARM-based semantics leak
hardware details. This was guardrail 1 in [ADR 0008](0008-isa-priority-policy.md).

## Options considered

### Option A — Abstract weak model; data-race-free safe code; SeqCst default, weaker opt-in (chosen)
- **Pros:** Correct by default. Maps efficiently onto both ISAs (ARMv8 has
  load-acquire and store-release instructions). Experts keep full control.
- **Cons:** SeqCst stores cost slightly more on x86. Defaults can hide
  over-synchronisation, though the performance report can flag it.

### Option B — Ordering always explicit
- **Pros:** Fully transparent.
- **Cons:** More tokens, and easy to choose an ordering that is too weak.
  Conflicts with "safe by default".

### Option C — TSO guaranteed
- **Pros:** Simple intuition.
- **Cons:** Barriers everywhere on ARM. This is exactly how x86 would hold
  AIL back.

## Decision

1. The language memory model is defined on an **abstract, weakly-ordered
   machine**, not on any real hardware.
2. **Safe code cannot cause data races.** The mechanism (type system,
   ownership, task confinement) is decided under the concurrency and
   ownership topics.
3. Atomic operations default to **sequentially consistent** ordering.
4. Weaker orderings (at least acquire, release, acquire-release and relaxed)
   are available by explicit request.

## Consequences

- The exact set of orderings, the formal model, and the syntax are left to
  the concurrency spec chapter.
- The performance report should flag SeqCst atomics where a weaker ordering
  can be proven sufficient. It suggests, never silently weakens.
- Data-race freedom is now a **requirement** on the type and ownership
  design. It also supplies facts that the task-local heaps (ADR 0012) need.

## Rejected alternatives

- **Always explicit:** verbose and error-prone. Contradicts safe-by-default.
- **TSO:** penalises ARM64. Contradicts lockstep parity (ADR 0008).
