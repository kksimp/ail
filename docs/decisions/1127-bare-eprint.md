# 1127. A bare `eprint(value)` is `io.eprint(value)`

- **Status:** Accepted
- **Date:** 2026-11-15
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** Std; AI understanding
- **Spec rules:** std.io.3
- **Extends:** [ADR 0157](0157-standard-os-modules.md), [ADR 1126](1126-std-modules-in-ail-without-use.md)

## Context

`print(value)` is a built-in with no module name; standard error needed
`io.eprint(value)`. In the E13 agent benchmark first drafts wrote a bare
`eprint(...)` or `eprintln(...)` next to `print(...)`; the call failed with
`expr.name.1` and cost a whole fix iteration. The measure is iterations until
the code first passes, so a first-draft error that is one word away is costly.

## Options considered

### Option A: `eprint` is a built-in like `print` (chosen)
`eprint(value)` checks, runs and counts as an effect exactly as `io.eprint(value)`.
`io.eprint` stays valid. Like `print`, the name can't be redeclared.
- **Pros:** the guess every agent makes works; one name for one thing.
- **Cons:** a second spelling of one function; a program that declared its own
  `eprint` now gets `mod.unit.1`-style "built-in can't be redeclared".

### Option B: keep the error, improve the fix
- **Pros:** one spelling.
- **Cons:** the first build still fails.

### Option C: also accept `eprintln`
- **Cons:** `println` is not accepted either; two spellings for one function
  breaks `print`'s own rule. `eprintln` keeps its fix to `io.eprint`.

## Decision

Option A. `eprintln` and `io.write_err` keep `std.use.2`'s fix.

## Rationale

AI understanding (criterion 1): the guess works on the first build. Readability:
equal. Speed and tokens: `io.eprint` becomes `eprint`, four characters less.
