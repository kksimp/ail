# 0211. Contracts are checked at every call and trap with the values

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 8 — Error handling
- **Spec rules:** [contract.req.1, contract.ens.1, contract.run.1–2](../spec/20-contracts.md)

## Context

A contract that fails must say what went wrong, precisely enough for an AI
to repair the right side: the caller (a broken `requires`) or the function
(a broken `ensures`). [ADR 0050](0050-error-return-traces.md) set the style:
name the place and the values, not just the rule.

## Options considered

### Option A — blame-directed traps with arguments (chosen)
A failed `requires` traps `contract.req.1` **at the call**; a failed
`ensures` traps `contract.ens.1` **at the `ensures` line**. Both name the
function, the condition as written, every argument as `name = value`, and,
for `ensures`, the returned value and where the function returned.
- **Pros:** the position is where the fix goes; the values make the failure
  reproducible without a debugger.
- **Cons:** rendering arguments costs time, but only on failure.

### Option B — a returned error (`ContractError`)
- **Pros:** recoverable.
- **Cons:** every function with a contract would become fallible, which
  changes its type and costs tokens at every call; a broken contract is a
  bug, which ADR 0047 makes a trap.

### Option C — both traps at the function
- **Cons:** a failed `requires` then points into correct code.

## Decision

Option A. `ensures` is checked only when the function returns a value, not
a failure. Calls made while evaluating a condition don't check their own
contracts, so a condition may call the function it describes without
infinite regress (Eiffel's rule).

## Rationale

AI understanding (criterion 2): the trap's position and message carry the
diagnosis. Speed is unaffected for functions without contracts (one length
test per call).

## Consequences

- Contracts on a trait method with a body apply to that method only; whether
  implementations inherit a trait's contracts is OQ-060.
- A static prover can later remove a check it proves; the trap rules stay.

## Rejected alternatives

- **Returned errors:** wrong category (a bug, not a condition to handle).
- **Trapping at the function for `requires`:** blames the wrong code.
- **Checking contracts inside conditions:** `ensures result == f(n - 1) * n`
  would recurse without end.
