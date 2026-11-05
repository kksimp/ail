# 0938. Run push-builder loops on the pool: one push per iteration, chunks joined in order

- **Status:** Accepted
- **Date:** 2026-11-04
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); P3 of the speed-edges review (2026-11-04), built at the designer's request; conditions from the integrator: results and traps equal the sequential run, `AIL_PARALLEL=0` runs the loop in order, timings on the Mac and ail-linux
- **Topic:** 11 — Concurrency; 25 — Optimization strategy
- **Spec rules:** conc.par.16 (new), conc.par.8 (amended), conc.par.12 (native)
- **Amends the scope of:** [ADR 0911](0911-default-automatic-parallel-loops.md), [ADR 0914](0914-automatic-fork-join-of-independent-calls.md)

## Context

ADR 0911 runs a proven loop on the pool only when it writes list elements and
`int` accumulators. The most common builder shape in AIL code is excluded:

```ail fragment
mut out := List[int]()
for i in 0..n
    out.push(f(i))
```

`push` writes all of `out`, so the checker called the loop sequential
(`conc.par.2`). The loop is the comprehension `[f(i) for i in 0..n]` written
with `push`: iteration `k` appends element `k`. Matmul's `c.push(row)` (a row
built by an inner loop) is the same shape, and so is `[e par for x in xs]`,
whose native lowering ran in order.

## Decision

1. **Proof (`conc.par.16`).** An outer list `out` is *built* by a loop when the
   body's only mention of it is one `out.push(e)` statement at the top of the
   body (not under `if`, a nested loop or `match`), the `push` is the built-in
   one, and no `continue` outside nested loops can skip it. Nothing else in the
   body reads or writes `out`. A built list doesn't make the loop sequential;
   `ail effects --parallel` says `conc.par.16: pushes one element per iteration to `out``.
   Range and list loops both qualify: chunks keep iteration order.
2. **Lowering (`lower_build.ail`).** Each chunk pushes into a list of its own
   (sized at creation by `native.bce.6`) and leaves it in its task record.
   When every chunk ends without a trap, `rt.par_gather` makes room in `out`
   once and appends the chunks' lists in chunk order. A trap leaves `out`
   untouched and the loop runs again in order (`conc.par.11`). At most four
   built lists per loop; the element type of `out` may come from the push
   itself (`mut out := []`).
3. **Comprehensions.** `[e par for x in xs]` with one clause and no filter is
   lowered as a `par for` that builds its list, so it runs on the pool natively
   as `conc.par.12` says.
4. **Unchanged.** The cost model (`conc.par.14`), the outermost-loop rule, the
   lowering's scan (`ast/parscan.ail`) and `AIL_PARALLEL=0` / `--deterministic`
   (`conc.par.9`) apply as before.

## Rejected alternatives

- **Rewrite the loop into a comprehension in the front end:** the body may hold
  statements, locals and inner loops before the push, which a comprehension
  element can't; the lowering would need the same chunked builder anyway.
- **Pre-size `out` and write `out[len + k]`:** needs the iteration count in
  advance and breaks for list loops whose body the analysis can't size;
  per-chunk lists need neither.
- **Allow a push under `if` (a filter):** chunks would still keep order, but
  sizes differ per chunk; left for a later step, since the gather already
  handles any chunk length. Kept out to keep the proof one rule.
- **Lock-free shared append:** order would depend on scheduling (`conc.par.1`).

## Consequences

- Builder loops whose iterations are heavy (matmul's `c` loop, row builders,
  per-item transforms calling a function) use every core. The gather is one
  copy of the pushed words, O(n), on the calling thread.
- Programs that use the pool pay the allocation lock (`H_FASTLIM`) after the
  loop; that is P1 of the same review.
- A written `par for` that pushes once per iteration is no longer a
  `conc.par.5` error.
