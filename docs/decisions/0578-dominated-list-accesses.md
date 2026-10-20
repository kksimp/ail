# 0578. Accesses that an earlier access or a push vouches for

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.bce.2](../spec/23-native.md), type.coll.4, native.build.1
- **Builds on:** [ADR 0391](0391-native-backend-optimisations.md) (MIR passes), [ADR 0384](0384-mir-shape-and-native-subset.md) (MIR)

## Context

`check` in binarytrees reads `left[node]` twice; `make` takes
`node := left.len()`, pushes to `left`, and then stores to `left[node]`. In
both, no check can fail, but each was compiled with one. Lists in the native
subset only grow (`push`); there is no `pop`, `clear` or `remove`.

## Decision

`dominated` in `compiler/mir/bce.ail` clears the trap site of `xs[i]` or
`xs[i] = v` when:

1. another access to `xs[i]` (the same values) is earlier on every path
   (by dominators; in a block, by position), or
2. `i` is `len(xs)` from before a `push` to `xs`, and that `push` is earlier
   on every path than the access.

The argument: lengths never decrease, so an access that passed (rule 1)
keeps `i` in bounds, and after a `push` the length is above `len` taken
earlier (rule 2). The pass is on only when `lists_only_grow` holds: no
function of the module calls an `rt.list_*` function that isn't known to
leave lengths alone. A later `pop` or `clear` adds such a function and turns
the pass off until this ADR is updated.

binarytrees: the second `left[node]` and the `set` after the `push` lose
their checks.

## Consequences

- Cheap (a map from `(list, index)` to places, one dominator matrix).
- Adding a shrinking list operation to the native subset needs a decision
  here: the pass would have to treat it as an end to the facts.

## Rejected alternatives

- **Value numbering of loads.** Would also remove the second load, not
  only its check, but needs alias information for `set` and calls; the check
  was most of the cost.
- **Treat any call as ending the facts.** Safe against a future `pop`, but
  loses `make`: its facts must survive the recursive calls.
