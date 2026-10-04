# 0045. Error model: errors are values propagated with `?`; bugs trap; no exceptions

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 8 — Error handling
- **Spec rules:** [err.model.1–2, err.prop.1](../spec/08-errors.md)

## Context

Brief §7 asks for explicit, safe, concise error handling, with no exceptions
for ordinary control flow and a concise propagation form such as
`data := read(path)?`. [ADR 0035](0035-integer-overflow-traps.md) already
makes overflow trap.

## Options considered

### Option A — Error values + `?` + traps for bugs (chosen)
- **Pros:** Every way a function can fail is visible in its signature, which
  matters most to AI and reviewers. No hidden control flow. Zero cost on the
  success path.
- **Cons:** Error information in signatures, and a `?` at each propagating call.

### Option B — Exceptions
- **Pros:** No `?` noise, and familiar.
- **Cons:** Hidden control flow, failure modes invisible to readers, and
  unwinding cost. Contradicts the brief.

### Option C — Error values only, no traps
- **Pros:** Nothing stops unexpectedly.
- **Cons:** Every index and arithmetic operation becomes fallible, which buries
  real errors. Contradicts ADR 0035.

## Decision

1. **Recoverable failures** (expected conditions such as a missing file or
   bad input) are **values** returned by functions. Their types appear in
   signatures (ADR 0046).
2. **Bugs** (overflow, out-of-bounds access, failed assertions, explicit
   unreachable code) **trap** (ADR 0047).
3. A postfix `?` on an expression that may fail returns the failure from the
   enclosing function. Otherwise it yields the success value.
4. AIL has no exceptions.

## Consequences

- Handling forms other than `?` (matching on errors, fallbacks, adding
  context to errors) are still to be designed (OQ-029).
- The AI semantic API can answer "what can this function fail with?" exactly.
- Diagnostics for an unhandled fallible value offer the fixes "propagate with
  `?`", "handle", or "provide fallback". The brief §13 example fits directly.

## Rejected alternatives

- **Exceptions:** hidden control flow.
- **No traps:** turns every operation into a fallible one.
