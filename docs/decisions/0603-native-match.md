# 0603. `match` in native code

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated the native self-compile lanes on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.subset.1](../spec/23-native.md), expr.match.1-12
- **Builds on:** [ADR 0602](0602-native-enum-layout.md) (enum layout), [ADR 0384](0384-mir-shape-and-native-subset.md) (MIR)

## Context

Lane 181's survey of `ail build compiler` counted `match` and the
functions ending in a `match` value among the top blockers. The compiler
matches on enums (with bindings, `|`, bare variants with data, guards),
on strings (`match args[0]`, `"check" =>`), and on ints.

## Decision

1. **A compare chain, no new MIR op.** The scrutinee is evaluated once
   and, for an enum with fields, its variant index loaded once (slot 0).
   Arms are tried in order; each pattern ends the current block with a
   branch to the arm's body or to the next arm's test. A variant pattern
   compares the index; an `int` or `bool` literal compares the value; a
   `str` literal calls `rt.str_eq` (lane 172's); a range `lo..hi` or
   `lo..=hi` is two comparisons; `p1 | p2` chains its alternatives to the
   same body; `_` jumps.
2. **Bindings** (expr.match.6) load the variant's fields into new
   immutable locals in the arm's scope; they count as borrowed, so
   returning one copies it (own.val.1). A **guard** (expr.match.11) is
   evaluated after the bindings; when false it branches to the next arm.
3. **Statement or value**, like `if` (expr.if.4): a value `match` passes
   each arm's value to a join block; an arm that leaves (`return`,
   `break`, `continue`) passes nothing. The arms' last expression is
   lowered expecting the type of the earlier arms or of the context, so a
   bare variant in an arm takes its enum (type.enum.3). A function may end
   with a `match` value.
4. **Falling out of the last arm is `unreachable`.** The checker makes
   every `match` exhaustive (expr.match.2) and the interpreter treats the
   fall-through as unreachable too.

## Rejected alternatives

- **A `switch` MIR op with a jump table.** Faster for a `match` over
  `ast.Expr`'s 27 variants, but a new op in every pass and both code
  generators; it belongs in lane 120's optimisation work once the
  self-compile works and a profile shows the chains. The chain needs no
  backend change.
- **Lowering `match` to nested `if` syntax trees.** It would lose the
  single index load and need a synthetic variable for the scrutinee.

## Consequences

- `compiler/mir/lower_match.ail` holds the lowering.
- `char` patterns wait for lane 172's `char` values; nested patterns don't
  exist in the language yet (expr.match.6).
- Until lane 172's `str.ail` lands, `compiler/rt/native/match.ail` carries
  the same `rt.str_eq`; it is deleted when that file merges.
