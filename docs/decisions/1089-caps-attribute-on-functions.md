# 1089. `@caps(LIST)` limits a function to the listed capabilities

- **Status:** Accepted
- **Date:** 2026-11-12
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** Effects and capabilities ([spec 18](../spec/18-effects.md))
- **Spec rules:** `eff.cap.6`, `eff.cap.7`

## Context

`--deny` and a manifest's `caps LIST` limit capabilities for a whole program or package
(`eff.cap.1-5`). `@pure` limits one function to no `io`, `unsafe`, `spawn` or capability, but
nothing says "this function may touch `fs` and nothing else". An agent that edits a function
cannot state, and the compiler cannot enforce, the capability budget of that one function. The
AI-first review (item 16) asked for it.

## Decision

`@caps(fs, net)` or `@caps(none)` on a function is an error when the function's inferred
effects (`eff.infer.*`, so everything it calls) include a capability outside the list
(`eff.cap.6`). The message names the shortest chain of calls, in the wording of `eff.pure.1`.
Entries are the names `--deny` takes; `none` stands alone; anything else is `eff.cap.7`.
Only capabilities are limited: `@pure` stays for `io`, `unsafe` and `spawn`.

## Options considered

### Option A: a separate `@caps` attribute (chosen)
- **Pros:** one small attribute, the `@pure` diagnostic shape, no change to inference.
- **Cons:** two attributes about effects.

### Option B: extend `@pure(fs)`
- **Pros:** one attribute.
- **Cons:** `@pure` means no capability; a list in it reads as the opposite. Rejected for AI
  understanding.

### Option C: also limit `io`, `spawn`, `unsafe` in `@caps`
- **Pros:** one list covers all effects.
- **Cons:** `io` is not a capability (`eff.cap.3`); mixing the two lists breaks that rule. Rejected.

## Consequences

`@caps` is accepted by the native backend and ignored at run time. A caller of the function
sees its inferred set unchanged.
