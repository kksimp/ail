# 0780. `ail fix` for a failure inside an operand, and a hint for `Json.num` in an `int` function

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 8 — Errors; 22 — Tools
- **Spec rules:** err.use.1, err.fix.1, expr.block.2
- **Builds on:** [ADR 0743](0743-operator-traits-and-constant-types.md)

## Context

`err.fix.1` says a fix is attached where one is mechanical. After ADR 0743 an unhandled
`err.use.1` got `?` after a statement, a binding, an argument or a receiver, even with a
trailing comment. A failure that is an operand (`1 + load(s)`) was still reported at the
operator's position and without a fix, because `operator` has no operand positions.

Whole-task t2 (day 2) lost 98.8 tokens to one slip: `doc.num("retries") else 2.0` gives a
float, and the function returns `int`. `expr.block.2` offered `x.to_int()` or `float`, but
the member is read as an int with `doc.int("retries") else 2`, and that was the agent's
second retry.

## Decision

- `binary` reports `err.use.1` at the failing operand, so the diagnostic carries the same `?`
  fix as any other position (rejected: the operator position, which can't name where `?` goes).
  Only the first failing operand is reported, as before. The self-hosted checker does the same.
- The `expr.block.2` help for a float body in an `int` function mentions `j.int(k) else 2` when the
  function calls `num` on a `Json` value; otherwise the text is unchanged. No fix is applied: moving
  to `int`, converting, and changing the return type are each a choice.

## Rejected alternatives

- **A `.to_int()` fix on the body.** `(retries + 1).to_int()` and `retries.to_int() + 1` differ for
  a fractional value, so a fix would guess.
- **A fix that rewrites `num` to `int`.** The default `2.0` must become `2`, and the member may
  be a real float elsewhere. It stays a hint.
- **A for-range bound with a failure** (`0..load(s)`). It reports `expr.loop.3`, not `err.use.1`;
  changing its rule is a spec decision.

## Consequences

`bootstrap/tests/fix/fail_in_operand` covers the fix; a unit test in `check.rs` covers the hint.
