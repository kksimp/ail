# 0577. Bounds checks in counted loops: versioning with guards and hoisted loads

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.bce.1](../spec/23-native.md), native.build.1, type.coll.4
- **Builds on:** [ADR 0391](0391-native-backend-optimisations.md) (MIR passes), [ADR 0384](0384-mir-shape-and-native-subset.md) (MIR)

## Context

After ADR 0391, fannkuch, matmul, binarytrees and fasta were 3–5.6x behind
`gcc -O2` ([native-performance.md](../design/native-performance.md)). Every
`xs[i]` was an inline compare, a branch, a data-pointer load and the
element load, and a row fetched as `a[i]` inside a loop was fetched again
on every iteration. In matmul and the copy and shift loops of fannkuch the
lists are long enough for every check to pass, but the compiler can't see
that: the bound is `n`, a parameter, and the lists are filled by `push`.

## Decision

A new MIR pass, `compiler/mir/bce.ail` (loop analysis in `loops.ail`), runs
last in `inline_module` on every function that isn't in the runtime. It
versions an innermost counted loop (Allen and Cocke's loop versioning):

1. **Counted loops.** The header branches on `p < hi` (or `hi > p`), `p`
   starts at a constant >= 0 and every path back adds 1, `hi` is defined
   outside the loop, and the loop has one exit, from the header. The
   two-pointer form `while i < j` with `i += 1` and `j -= 1` also counts:
   `0 <= i < j <= j0` in the body, `j0` being `j`'s start.
2. **Guards.** Each access `xs[i]`, `xs[i + c]` (1 <= c <= 16) and `xs[k]`
   (`k` fixed in the loop) on a list that is fixed in the loop needs one
   test before the loop: `hi <= len(xs)`, `hi <= len(xs) - c`, or
   `0 <= k < len(xs)`. For the two-pointer form `hi` is `j0`, and `xs[j]`
   needs `j0 <= len(xs) - 1`. The tests are new blocks ahead of the loop.
3. **Two loops.** When the tests pass, a copy of the loop runs with the
   checks of those accesses removed (`site` -1). Otherwise the original loop
   runs, as before: a trap is raised by the same instruction, with the same
   operands, on the same iteration. Values the loop leaves in its header
   parameters become parameters of the exit block, which both loops jump to.
4. **Fixed lists.** A list is fixed in the loop if it is defined outside it
   (or is a hoisted element) and neither it nor a list it was fetched from
   is passed to a call, or stored to with `set` when it holds lists. A
   `set` of an element of an `int` list doesn't change a length, so it is
   allowed for the length test; it does change an element, so a load of that
   list's element is not hoisted.
5. **Hoisted loads.** `get y, c` with `y` and `c` fixed, and `y` not stored
   to in the loop, is done once after its own bounds tests (a row `a[i]`
   in matmul's inner loop). The guard blocks hold the load; the copy of the
   loop uses it. A load is only hoisted into the guard, never ahead of it, so
   an out-of-range index can't trap early or late.

Results at the perfbench sizes (callgrind, instructions): fannkuch
442M → 296M, matmul 223M → 157M, sieve 144M → 136M; at the large sizes
fannkuch 0.84 s → 0.51–0.61 s and matmul 0.20 s → 0.13 s.

## Consequences

- Code size of a versioned loop doubles. Only innermost loops with at least
  one removable check are versioned.
- `b[k][j]` in matmul keeps the check on the row's length: the lengths of the
  rows aren't known.
- Compile time rose by about 0.1 s per program (the passes run in the
  interpreter, [native-performance.md](../design/native-performance.md)).
- The data pointer and the length aren't kept in registers yet; that needs
  new MIR operations and changes to both code generators.

## Rejected alternatives

- **Prove `i < len(xs)` from the loop alone.** Works only for
  `for i in 0..xs.len()`; matmul and fannkuch loop to `n`.
- **Hoist the checks with no copy of the loop.** A hoisted check that fails
  would trap before the work the interpreter did first, so the output and
  the trap's line would differ (native.build.1).
- **Deoptimise at the failing check.** Needs a way back into checked code
  in the middle of a loop; versioning needs none.
- **A range-analysis lattice.** More general, and a far larger pass for a
  compiler that runs in the interpreter; the counted loop covers what the
  corpus writes.
