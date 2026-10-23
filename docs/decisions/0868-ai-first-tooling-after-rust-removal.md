# 0868. AI-first tooling after Rust removal: eight native features, zero cost when unused

- **Status:** Accepted
- **Date:** 2026-10-23
- **Decided by:** Designer (2026-10-23, approved all eight items of the AI-first wishlist)
- **Topic:** 22 — Tooling
- **Builds on:** [ADR 0147](0147-typed-holes-todo.md), [ADR 0394](0394-token-budgets-on-every-command.md), [ADR 0398](0398-record-and-replay.md), [ADR 0016](0016-optimized-and-observable-debugging.md)

## Context

AIL's goal is to be the language agents write best, while staying as fast or almost as fast
as C. Several agent tools exist today only in the Rust interpreter (record and replay,
capability denial) or only in part (typed holes list names, not candidates). Once the Rust
bootstrap is gone ([plan](../roadmap.md)), each tool needs a native form.

## Decision

These eight items start after Rust removal, in this order of priority. Each must cost
nothing in a program that doesn't use it: no runtime check, no extra code, no slower build.

1. **`ail ask`: semantic queries.** Questions such as "who writes field `x`?", "what calls
   `f`?" and "which effects reach `main`?" are answered from the compiler's own graph of the
   program, with no grep and no file reading.
2. **Typed holes rank candidates.** `todo` (ADR 0147) keeps its token. The hole warning also
   lists the in-scope values and calls whose type fits, ranked by fit. **`?` is not used**:
   it is already the error-propagation operator and appears inside strings and comments, so
   a second meaning would be ambiguous.
3. **Edit impact.** `ail edit` reports which tests and effects an edit touches, and
   `ail test --affected` runs only those.
4. **Native time travel.** A native build records a run (inputs, scheduling, syscalls) when
   asked to. Replay answers questions such as "when did `x` become 0?" without adding prints
   (native form of ADR 0398).
5. **Contracts checked by tools** (lower priority). `requires` and `ensures` are fuzzed or
   checked statically, so agents get counterexamples rather than traces.
6. **Token-priced diagnostics.** Every diagnostic has a one-line form and an expanded form,
   and `--budget` (ADR 0394) chooses between them.
7. **Native agent sandbox.** Running generated code with capability denial (`eff.cap.1`) plus
   CPU, memory and wall-time caps, enforced by the native runtime.
8. **Live attach.** A running native binary answers state, metrics and heap-summary queries
   over a small local protocol, so `@observe` works for every program. Only programs built
   with attach enabled pay for it.

## Consequences

Each item gets its own ADR, spec rules and owner when work starts. The roadmap lists the
items after the Rust-removal milestone.

## Rejected alternatives

- **`?` as the hole token** (the designer's suggestion): it clashes with the try operator
  (`x?`) and reads as a question mark in prose. `todo` already exists and says what it is.
- **Build these on the interpreter first:** the interpreter is being removed; work done there
  would be thrown away.
- **Always-on recording or attach:** this would cost speed in every program, against the
  near-C goal.
