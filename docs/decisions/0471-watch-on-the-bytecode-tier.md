# 0471. `--watch` runs on the bytecode tier

- **Status:** Accepted; amends [0364](0364-bytecode-tier-limits-coverage-and-traces.md) (`--watch` no longer keeps a run on the walker)
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076), within the designer's ADR 0359
- **Topic:** interpreter performance; tracing
- **Spec rules:** none changed; `trace.watch.1` and `trace.watch.3` keep their output

## Context

[ADR 0364](0364-bytecode-tier-limits-coverage-and-traces.md) left `--watch`
on the tree walker, because the walker hooks every `let`, assignment and
mutating method and the tier compiled those without the hook. So an agent
that asked "where does this value change" got none of the speed-up.

The walker's hooks are at four places: a `mut` `let` (`trace_watch`), a
plain or field or index assignment (`trace_watch` and `watch_element`), a
call that passes a `mut` argument or a mutating receiver (`watch_call`, in
`invoke`), and the builtin mutators, which show their own watch in
`builtins.rs`. Frame aliases (the callee's name for the watched value, shown as "as `self`")
are kept by `run_body`, which the tier already shares.

## Options considered

#### Option A — the tier hands each watched node to the walker (chosen)
When `--watch` is on, the compiler emits `Stmt` (the walker's `stmt`) for
every assignment and every `mut` `let`, and `MethodOn`/`Expr` for every call
of a mutating method. Calls with a `mut` argument or a `mut` parameter were
already handed to the walker, so they keep their `invoke` hook. Everything
else (arithmetic, loops, reads, non-mutating calls) stays on the tier.

- **Pros:** the walker's hook code runs unchanged, so the output is the walker's
  by construction; the change is a few compile-time checks; with `--watch`
  off the compiler reads one flag per statement and emits nothing new, so the
  watch-off code is the same instructions.
- **Cons:** a watched program still hands its assignments to the walker, so
  the speed-up under `--watch` is smaller than with no watch. Measured below.

#### Option B — call the watch hooks from the tier's own instructions
Add `watch_element` and `trace_watch` calls to `Ins::Assign`-style
instructions and to `Ins::Call` for `mut` arguments.
- **Pros:** full speed-up under `--watch`.
- **Cons:** duplicates the walker's place and alias logic in a second place
  that must match it byte for byte; the frame alias rules of `watch_call` would
  need a second copy. Ruled out on the oracle rule (ADR 0032): the walker must
  stay the only definition of what `--watch` prints.

#### Option C — keep `--watch` on the walker (as before)
- **Cons:** the runs an agent makes to find a changing value never use the tier.

## Decision

Option A. `bytecode::usable` no longer refuses `--watch`. The compiler's
`stmt` and `method` hand off watched nodes as described above. The tier's
own instructions are unchanged.

## Verification

- The five `trace_watch*` cases and the new `trace_watch_tier` case (loops,
  `mut` arguments, a mutating method, a helper that is not watched) give the
  same stdout and stderr on both tiers, and `./ail selftest` runs each `--watch`
  program on both tiers (`bootstrap/tests/bytecode.rs`).
- `AIL_BYTECODE_STATS=1 ail run --watch total` on `trace_watch_tier.ail`
  reports 4 of 4 functions compiled and 10 calls on the tier.

## Measurements

Callgrind instruction counts and wall time are in the PR body and in
[interpreter-performance.md](../design/interpreter-performance.md#watch-on-the-bytecode-tier-lane-142).

## Rejected alternatives

Option B (a second copy of the watch logic) and Option C (no change) are
rejected above.
