# 0394. List comprehensions and slices in the native subset

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.subset.1](../spec/23-native.md), native.build.1, type.coll.4, own.val.1
- **Builds on:** [ADR 0392](0392-native-structs-and-methods.md), [ADR 0393](0393-native-modules.md)

## Context

After modules, the compiler's own code still had 63 list comprehensions
and a few slices (`args[1..]`) outside the native subset, and no lane owned
them.

## Decision

1. **List comprehensions** lower to the loops they mean: a new list, then
   a `for` per clause (an `if` for each filter), pushing the element. They
   get the `for` loop's code and `push`'s fast path; lane 176 may
   pre-size the list. Set and map comprehensions stay with lane 173.
2. **Slices** `xs[a..b]`, `xs[a..=b]`, `xs[a..]`, `xs[..b]` check
   `0 <= a <= end <= len` (and that `b + 1` doesn't overflow) in MIR and
   then copy with `rt.list_slice`. A failed check traps with the
   interpreter's message (`slice A..B is out of bounds for a list of
   length N`), through a new MIR operation `trap` that jumps to a trap stub
   with the runtime's message kind 7. A slice of lists or structs is
   copied deeply, so the result is independent (own.val.1).

## Consequences

- `trap` is the general way for lowering to end a block in a trap with
  site texts; the list length for kind 7 goes through a runtime slot,
  because a trap stub passes two numbers.

## Rejected alternatives

- **Comprehensions as a runtime call taking a function.** Needs closures
  (lane 174) and an indirect call per element.
- **Slices as views sharing the data.** Cheaper to take, but every later
  write would need copy-on-write checks to keep values independent.
