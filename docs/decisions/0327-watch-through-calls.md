# 0327. `--watch` follows a variable through `mut` calls

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Semantic API (topic 12); modules (topic 5); testing
- **Spec rules:** `trace.watch.2–3`

## Context

`--watch NAME` (ADR 0268) matched only assignments in the frame where `NAME`
is a local, so it went quiet when the variable was passed `mut` to a function
or method, which is where state usually changes.

## Options considered

### Option A — alias the callee's name for the watched variable (chosen)
- **Pros:** exact: only `mut` receivers and arguments rooted in the watched
  name (or in an alias) are followed; events name the frame that wrote.
- **Cons:** a small amount of per-call work, only while `--watch` is on.

### Option B — match the name in every frame
- **Pros:** nothing to track.
- **Cons:** reports unrelated locals with the same name and misses the
  callee's own name for the value (`self`, a parameter).

### Option C — report every write to the value by identity
- **Pros:** complete, including builtin mutating methods.
- **Cons:** values are copied on call (mutable value semantics), so identity
  does not survive; would need a per-value id on the hot path.

## Decision

While `--watch` is on, a call site that passes the watched variable (or an
alias) as a `mut` receiver or `mut` argument makes the callee's name for it an
alias for the callee's frame. An assignment rooted in an alias shows the new
whole value, the frame's function and, when it differs, the alias (`via`).
Builtin mutating methods (`push`) remain unshown (trace.watch.2).

## Rationale

AI understanding decided (criterion 2): "who changed this" is what an agent
needs. Speed: when watch is off, a call tests one byte (trace.run.5);
perfbench is unchanged.

## Consequences

- The text form adds ` in FRAME` after the first frame, so existing output is
  unchanged for programs that write in one frame.
- Writes through `ref` cells and tasks are not followed.

## Rejected alternatives

- Name matching in every frame and identity tracking, as above.
