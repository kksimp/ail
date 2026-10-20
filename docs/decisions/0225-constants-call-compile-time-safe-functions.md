# 0225. Constants may call compile-time-safe functions, computed with the interpreter's semantics

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 18 — Compile-time execution; 12 — Modules
- **Spec rules:** [ct.const.1–4](../spec/15-compile-time.md), [mod.const.2](../spec/12-modules.md) (amended)
- **Amends:** [ADR 0132](0132-module-constants.md) (designer's decision that a constant's value is a constant expression)
- **Design note:** [compile-time-execution.md](../design/compile-time-execution.md)

## Context

ADR 0132 limits a constant to literals, operators and other constants, so a
computed table (`SQUARES`, `CRC_TABLE`, a keyword list split from a string)
must be built in `main` and passed down. The plan asks for `const fn`-style
evaluation using the interpreter, limited to pure code by the effect system
(ADR 0163). **Tokens decide** between the syntaxes, measured with
`tools/tokbench` (mean of five tokenizers, two constants): plain
`NAME := f()` 17.4, `comptime f()` at the use 21.4 (+23%), and a `const fn`
marker on each callee +1 token per function. Then **AI understanding**: one
colour of function, and the diagnostic names the chain when a callee isn't
compile-time-safe.

## Options considered

### Option A — no marker; the effect set decides (chosen)
- **Pros:** zero tokens; every pure function already qualifies, including
  library ones nobody annotated; "compile-time-safe" is exactly what `@pure`
  means (ADR 0164), so there is one concept.
- **Cons:** an edit deep in a callee can invalidate a constant far away.
  Mitigated by the call-chain diagnostic and by `@pure` for a stated promise.

### Option B — `const fn` marker (Rust)
- **Pros:** the promise is visible at the function.
- **Cons:** tokens on every callee; a second colour that libraries forget to
  apply, so useful functions can't be used.

### Option C — `comptime` at the use (Zig)
- **Pros:** explicit at the constant.
- **Cons:** +23% tokens on a construct whose meaning is already fixed.

## Decision

1. A constant's value may be any expression (`ct.const.1`).
2. Everything it may call must be compile-time-safe: its effect set has none
   of `io`, `unsafe`, `spawn` or a capability (`ct.const.2`). The diagnostic
   names the shortest chain to the offending effect.
3. A constant can't hold a failure: `?` makes a failure a compile error at
   the constant, `else` supplies a fallback (`ct.const.3`).
4. A constant's type may not contain `ref` or a `unique` type (`ct.const.4`).
5. The value is computed with the interpreter's semantics (ADR 0032); it is
   target-independent because every target is 64-bit little-endian with
   defined arithmetic (ADR 0007, ADR 0035).

`mod.const.2` is amended to admit these expressions; the rest of ADR 0132
(no order, no mutable globals, a failure is reported at the constant)
stands.

## Consequences

- **Designer, look first at:** this widens your ADR 0132. The key risk is
  "distant edit breaks a constant"; the alternative is a `const fn` marker
  (Option B).
- The bootstrap implements points 1–4: constants are still computed before
  `main` (its compile time), and the checker enforces the effect rule with
  `check/effects.rs`. Evaluation inside `ail check` waits for the query
  compiler.
- `ail api` can show a constant's computed value (an additive `value` key,
  follow-up).
- The same rule governs derive generators (ADR 0227).

## Rejected alternatives

- **`const fn` (B):** tokens and a second function colour.
- **`comptime` (C):** +23% tokens for no extra meaning.
- **Compile-time parameters / types as values (Zig):** signatures would no
  longer say what a function accepts; generics already cover type parameters.
- **Build scripts with capabilities:** the supply-chain hole of wishlist 5.1.
