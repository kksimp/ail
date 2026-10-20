# 0378. The AIL checker skips what it can't type yet, and the comparison skips it too

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 23 — Compiler architecture (Stage 3, ADR 0022); 28 — Testing strategy
- **Spec rules:** the expression and statement rules listed in `compiler/check/coverage.ail`

## Context

Slice 2 of lane 114 ports the typing of function bodies (`check_fn` and
everything it calls) to `compiler/check/`. A body can use any construct, so
until every one is ported some bodies meet a construct the AIL checker has
no counterpart for. ADR 0377 compares diagnostics over covered rules and
lines; slice 2 has to say which lines a partly ported checker covers.

## Options considered

### Option A — gap: give up on the function, and skip its lines (chosen)
`Checker.gap(why)` marks the current function and types the construct as
unknown. The function's lines, from its header to the next declaration,
leave the comparison, as do a gapped constant or struct invariant, and the
deriving `impl` line of generated code with a gapped function. A caller of a
gapped function whose error set is inferred is gapped too, since the set is
unknown. With `AIL_CHECK_GAPS` set, each gap prints `gap REASON` to stderr.
- **Pros:** never reports a guessed diagnostic; agreement stays total while
  coverage grows; the driver prints `bodies CHECKED TOTAL`, a single number
  for progress; the gap tally names the next construct to port.
- **Cons:** a gapped function's other diagnostics aren't compared.

### Option B — compare everything, and list known differences
- **Cons:** a list of expected differences goes stale and hides new ones.

### Option C — port in source order, all at once
- **Cons:** no comparison until the end; differences arrive in thousands.

## Decision

Option A. Two more rules keep the comparison exact while slice 3 is open:

1. Rules that both the typing pass and slice 3's flow passes report
   (`own.move.2`, `own.move.6`, `own.uniq.2`, `own.defer.3`, `conc.par.12`,
   `conc.task.5`) stay out of `COVERED_RULES` until slice 3 ports the flow
   passes. Slice 3 ported them, with the facts typing records for them
   (call targets, handle and atomic calls, standard calls, writes through
   a `ref`).
2. Both sides are filtered by the same rules and lines before comparing.

At the end of slice 2 the AIL checker types 1,819 of the 1,848 function
bodies in the corpus (98.4%), and agrees on 495 of 495 programs. The
remaining gaps are `std.tensor` and `std.ui` (their functions, methods and
operators) and the fields of standard modules.

## Rationale

ADR 0076 criterion 2 (AI understanding): a diagnostic the AIL checker
reports is one `ail check` reports at the same place, with no exceptions to
remember.

## Consequences

- Porting a construct is measured by the bodies count rising with agreement
  still total.
- Slice 4's report counts gapped bodies as not yet agreeing.

## Rejected alternatives

Options B and C, above.
