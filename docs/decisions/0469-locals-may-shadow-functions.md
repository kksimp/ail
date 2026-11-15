# 0469. A local may shadow a function; a call of it in the local's block is `expr.call.1`

- **Status:** Superseded by [1121](1121-locals-may-shadow-functions-and-modules.md) (which keeps this rule and extends it to modules, imports and fields)
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 5 — Expressions (names and bindings)
- **Spec rules:** `expr.bind.5` (amended: a function name is no longer an
  error for a local); `expr.call.1` (unchanged, now also covers a call of a
  shadowed function)
- **Resolves:** OQ-116
- **Criterion:** AI understanding (ADR 0076, criterion 2)

## Context

`expr.bind.5` forbids a local with the name of any module item, prelude
function included. A module is a directory, so a private helper `word` in one
file forbids a local `word` in every other file of that module. The parser
hit it for `at`, `word`, `operand` and `float`. [ADR 0044](0044-no-shadowing.md)
forbids a local shadowing another local, which this does not change.

## Options considered

### A (chosen): a local may shadow a function
- **Rule:** a local named like a function (module-level or prelude) is legal
  for the rest of its block. A call of that name in the block is `expr.call.1`
  ("`word` is a value, not a function"), which the checker already reports for
  any local called like a function.
- **Pros:** locals stay free to name; the one confusing use, a call, is still
  an error; no change to other files' code.
- **Cons:** a reader must notice the shadow when they see a name that is both.

### B: limit the rule to items declared in the same file
- **Pros:** local reasoning per file.
- **Cons:** moving a function between files changes which locals are legal.

### C: keep the rule
- **Pros:** no ambiguity at all.
- **Cons:** renaming, at a cost in the parser's own names.

Types, constants and imported names stay errors (option A only relaxes
functions): a type shadowed by a local would make `Point(...)` ambiguous in a
way a function call is not.

## Decision

Option A.

## Measurements

No new syntax, so no tokbench table. The change only makes more names legal
for locals. `ail check compiler` passes after it, with the parser's own
`word` and `at` locals in place.

## Consequences

- OQ-116 is closed.
- Test: `bootstrap/tests/cases/local_shadows_fn.ail` (positive) and
  `local_shadows_fn_errors.ail` (`expr.call.1`); the unit test `no_shadowing`
  in `bootstrap/src/check.rs`.
- The checker's rule is in Rust only; the AIL checker's body pass is slice 2
  ([self-hosting status](../design/self-hosting-status.md)).

## Rejected alternatives

- Limiting the rule to items declared in the same file (option B): rejected because moving a function between files would change which locals are legal.
- Keeping the rule (option C): rejected because renaming costs something in the parser's own names.
