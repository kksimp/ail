# 0571. Key functions are lowered inline, with no closure object

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [expr.key.1-4](../spec/05-expressions.md), [native.subset.1](../spec/23-native.md)
- **Builds on:** [ADR 0250](0250-key-functions-and-take.md), [ADR 0384](0384-mir-shape-and-native-subset.md)

## Context

Lane 174 was briefed to give the native backend function values and
closures with an indirect call. The only lambda in the language is the key
function `by: |x| e` (expr.key.1), and it "isn't a value": it can't be
bound, returned, stored or passed on. The compiler's own source uses it
only in `sorted(by:)` (no capture analysis is needed: `e` sees the
enclosing locals because it stays in the enclosing function).

## Decision

`xs.sorted(by: |x| e)` and `xs.sort(by: |x| e)` are lowered inline
(`compiler/mir/lower_closure.ail`): a counted loop binds `x` to each
element in order, lowers `e` once per element (expr.key.3), and pushes the
`int` key. The keys go to `rt.sort_perm` ([ADR 0572](0572-native-stable-sort.md)); a second loop
builds the result from the indices. `xs.sorted()` and `xs.sort()` on a
`List[int]` use the list as its own keys. A key or element type other than
the supported ones (`float` and `str` keys; `float` and `bool` lists
without a key function) is a native.subset.1 problem, not a silent
difference. `min(by:)`/`max(by:)` return an optional and wait for lane
175.

Captured variables are read where they live, so the by-value capture
question doesn't arise, and a trap in `e` has the interpreter's site and
operands (`expr.arith.1`).

## Rejected alternatives

- **A closure object (kind 6) and `call_indirect` per key.** Costs an
  allocation, a call per element and a new MIR op in both code generators
  for a construct that is not a value. Inlining lets the optimiser fold the
  key, and needs no ABI.
- **A comparison callback in the runtime sort.** Evaluates `e` per
  comparison, which expr.key.3 forbids ("once per element").
