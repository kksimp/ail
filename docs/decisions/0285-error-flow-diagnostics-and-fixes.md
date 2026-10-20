# 0285. Error-flow diagnostics carry applicable helps and fixes

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** AI-first tooling
- **Spec rules:** `err.ret.1`–`err.ret.3`, `err.fix.1`, `err.prop.1`, `err.prop.3`, `err.use.1`, `err.else.3`

## Context

The checker rejected `return OopsVariant` in a function that can't fail with "expected `int`, found `Oops`", rejected `return opt` where a result is expected with a bare mismatch, and gave `?` with nowhere to go a help but no edit. An agent had to work out the signature change itself.

## Options considered

### Option A — dedicated rules with the signature change as a fix (chosen)
- **Pros:** tokens: one fix replaces a read of the spec and a retry; AI understanding: a rule ID per mistake (`err.ret.1-3`) says which one it was.
- **Cons:** `ail fix` can now change a function's return type, and a caller of it then needs `?` in the next round.

### Option B — help text only
- **Pros:** nothing is edited without an agent choosing it.
- **Cons:** the agent writes the same edit every time.

## Decision

Add `err.ret.1` (error value returned where the function can't fail), `err.ret.2` (optional returned where a result is expected) and `err.ret.3` (error type missing from the stated set). Each help names the return type to write (`T ! A | B`). Where the fix is mechanical, the diagnostic carries it (`err.fix.1`): `! E` appended to a signature that has no failing type (for `err.use.1` together with `?`, for `err.prop.1` and `err.ret.1`), and the name dropped from `else name` on an optional. A function with an explicit error type or an optional return type is never edited; its help says what to write.

## Rationale

Tokens first: a fix that applies saves the round trip an agent spends reading the rule. The signature edit is always valid by `err.type.7`, and `ail fix` re-checks, so a wrong guess can't leave a broken file.

## Rejected alternatives

- Help text only (Option B), as above.
- Rewriting the callers of a function made fallible: unbounded, and `ail fix`'s rounds already do it one step at a time.
