# 0468. A `none` local takes its type from its first assignment; an `if` with a `none` branch is `T?`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 4 — Types; 5 — Expressions
- **Spec rules:** `type.opt.9` (new), `type.opt.10` (new); `type.opt.4` and
  `type.opt.5` unchanged
- **Resolves:** OQ-114, OQ-115
- **Amended by:** [ADR 1198](1198-none-local-typed-by-its-other-operand.md) (an operand takes its type from the other operand)
- **Criterion:** AI understanding (ADR 0076, criterion 2). Token effect under
  the 2% tie threshold (see Consequences).

## Context

Writing the self-hosted parser ([ADR 0377](0377-checker-in-ail-ported-pass-for-pass.md),
lane 113) hit two gaps. `mut owner := none` has no type, so 21 locals in
`compiler/` started as a generic helper `absent[T]()`. And
`if c then x else none` with a plain `T` for `x` reported `type.opt.4`, so
a helper such as `char_text` was needed for each such case.

## Options considered

### OQ-114
- **A (chosen): type the local by its first assignment.** `mut x := none` is
  a pending local, as `[]` and `{}` are (`type.coll.13`). `x = e` gives it
  `T?` with `T` the type of `e` (`type.opt.5`). A read before that is an error.
  - **Pros:** no new syntax; the type is still found in one function; the
    local reads like the code a reader expects.
  - **Cons:** a read before the first assignment is an error even where a
    reader might expect `none`; the checker keeps one more pending kind.
- **B: a typed literal `none[T]`.** Explicit, but a new form for one case.
- **C: keep `absent[T]()`.** No change; costs a helper and a type in each use.

### OQ-115
- **A (chosen): when a branch is a bare `none` and no type is expected, the
  other branches give `T`, and the `if` has type `T?`.** Each `none` branch
  is then `none` of type `T?`.
  - **Pros:** what a reader expects; removes `char_text`-style helpers.
  - **Cons:** the checker types the other branches before the `none` ones.
- **B: keep `type.opt.4`.** One rule fewer; a helper or an `if` statement per use.

## Decision

OQ-114 option A; OQ-115 option A. A `none` branch is typed after the others
in `Agree` (`bootstrap/src/check.rs`); a `mut` local from `none` is a
`Pending` entry with `opt` set.

Rules:
- **type.opt.9:** a `mut` local declared with `:=` from `none` has no type
  until its first assignment `x = e`, whose type is `T?`. An error if it is
  used before, or never assigned.
- **type.opt.10:** an `if` with no expected type, with a bare `none` branch and
  other branches of type `T`, has type `T?`. If every branch is `none`, the
  error is `type.opt.4`.

## Measurements

`tools/tokbench/run.sh --json` on the three AIL files that changed
(`compiler/syntax/parse.ail`, `recover.ail`, `compiler/check/checker.ail`),
before and after the helpers were removed (mean of five tokenizers):

| File | Before | After | Change |
|---|---|---|---|
| `compiler/check/checker.ail` | 8,806 | 8,793 | −0.15% |
| `compiler/syntax/parse.ail` | 17,926 | 17,905 | −0.12% |
| `compiler/syntax/recover.ail` | 5,654 | 5,650 | −0.08% |
| Total | 32,386 | 32,348 | −0.12% |

Under the 2% tie threshold (ADR 0076), so the choice rests on criterion 2.
Only the locals that are assigned before any read lost their helper: 4 locals
read before assignment keep `absent[T]()`.

## Consequences

- OQ-114 and OQ-115 are closed; the rejected options are recorded above.
- The checker's body pass isn't in AIL yet (`compiler/check/coverage.ail`
  covers declarations only). Its port is part of checker slice 2
  ([self-hosting status](../design/self-hosting-status.md)).
- Tests: `bootstrap/tests/cases/optional_none.ail` (positive) and
  `optional_none_errors.ail` (`type.opt.9`, `type.opt.4`).

## Rejected alternatives

- OQ-114, option B: a typed literal `none[T]`. Rejected because it is a new form for one case.
- OQ-114, option C: keep `absent[T]()`. Rejected because it costs a helper and a type in each use.
- OQ-115, option B: keep `type.opt.4`. Rejected because the helper or the `if` statement per use is what the chosen rule removes.
