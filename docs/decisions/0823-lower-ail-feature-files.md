# 0823. `lower.ail` is a core plus feature files with one-line hooks

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Builds on:** [ADR 0392](0392-native-structs-and-methods.md), [ADR 0085](0085-interim-local-testing-gate.md)

## Context

Every native feature touched `compiler/mir/lower.ail`, so every native PR
needed a slot in the integrator's one-at-a-time lower.ail queue. In 82
commits since 2026-10-20 12:00, the diff hunks fell mostly in two
dispatchers: `lower_expr` (22) and `lower_method` (21). Both did real work
inline, so a new method on `str` and a new tensor operator edited the same
113-line function. Next came `declare_fn` and `lower_fn` (26 together), the
`Lower` struct and its constructor (17), and `lower_call` (9). The file was
1,853 lines beside 3,278 lines in thirteen `lower_*.ail` files.

## Decision

1. **Dispatchers only dispatch.** In `lower_expr` each arm is one call, and
   in `lower_method` each `match r.ty` arm is one call. The work lives in a
   function in the feature's file: `lower_binary_expr`, `lower_index`,
   `lower_member_expr`, `lower_or_else`, `int_method`, `float_method`,
   `str_method`, `list_method` and so on. A dispatcher changes only when the
   AST or `Ty` gains a variant.
2. **The rule.** A native feature lives in its own `lower_*.ail` file, new
   or the type's own. It may add at most one dispatch line and one state
   field to `lower.ail` (or, after half 2, `lower_fn.ail`). A PR that
   follows the rule needs no lower.ail queue slot. Only a PR that cannot
   follow it does. The rule is in AGENTS.md's git workflow section.
3. **Operators and number methods** move to `lower_ops.ail`.
4. **Half 1 now, half 2 later.** Half 1 (this ADR's PR) is the dispatcher
   change above. Half 2 moves function framing and declarations to
   `lower_fn.ail`, types to `lower_types.ail`, calls to `lower_call.ail`,
   and per-feature state into small structs. It takes its own slot after the
   queue drains.
5. **The move is a script, not a hand edit.** `tools/lower-split/split.py`
   rewrites the current `lower.ail`, and `tools/lower-split/check.sh`
   proves it moved only code, with no behaviour change: the MIR dump of
   every native test and perfbench program, and the x86-64 and aarch64
   executables of the perfbench programs, are byte-identical before and
   after, apart from the line numbers of warnings about the compiler's own
   source. The script is rerun on the head at the split's slot rather than
   rebased.

## Consequences

- Most native features stop conflicting: two PRs add a method on different
  types to different files.
- Reading `lower_expr` and `lower_method` now shows what exists. The
  details are one jump away.
- The script is a one-off. After half 2 it can be deleted.

## Rejected alternatives

- **Leave `lower.ail` whole.** The queue stays serial and each slot
  re-merges the batch head.
- **Split by compiler phase** (declare, lower, generate). The conflicts
  follow features, not phases, so they would remain.
- **A table of function values for dispatch.** It needs native closures
  in the compiler's own subset, and adds an indirect call to every
  expression.
