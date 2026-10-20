# 0575. Native results: the error slot and a return trace

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.subset.1](../spec/23-native.md), err.type.1, err.prop.1, err.prop.5, err.else.3, err.else.5, err.ctx.1, err.ctx.3
- **Builds on:** [ADR 0574](0574-native-optional-layout.md), [ADR 0385](0385-x86-64-code-generation.md)

## Context

A function `T ! E` returns a success value or an error. Returning both in
registers needs a second return register in MIR and both generators.

## Decision

A fallible function returns its **success value** (the unit for `! E`), and
reports failure through a runtime **error slot** (global 8): the error's
display text as a string pointer, or 0. Invariant: the slot is 0 whenever
no failure is travelling.

- `return E()` (the error constructor) stores the error and returns zero.
- A call is followed by `rt.failed()` (a load and a compare, inlined). `e?`
  on failure adds a trace line and returns zero, leaving the slot set.
  `e else ...` clears the slot first (`rt.err_clear`) or, for
  `else err`, takes the text (`rt.err_take`) and binds it as a `str`.
- The **return trace** (err.ctx.1) is a list of ready-made `  at PATH:LINE:COL`
  lines: global 9. `return E()` starts it, each `?` appends its own line.
- `rt.exit` calls `rt.err_report` first: a failure that reached `main` prints
  `error: TEXT` and the trace to standard error and exits 1, as `ail run`
  does (err.text.1, err.ctx.3). `?` on an optional in a function with
  inferred errors fails with `a value is absent` (err.prop.5).
- `globals` in the image grows from 8 to 16 slots.

## Consequences

- A success path costs one load and branch per call; a failure path does a
  store and a list push. No second register, so lane 120's files change only
  by dispatch lines.
- The slot is process-global: `spawn` (last on the critical path) needs it
  per task, in the task's own block, when tasks arrive.
- `? "context"` is outside the subset for now (native.subset.1).

## Rejected alternatives

- **Second return register** (rdx / x1 plus a flag): the fastest, and the
  right end state; it needs a pair in MIR and the allocator first.
- **Boxed results** (kind 10): an allocation per call.
- **Unwinding tables**: nothing else in the runtime needs them.
