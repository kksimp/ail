# 0618. Typed fast paths in the bytecode tier

- **Status:** Superseded by [ADR 0873](0873-rust-only-features-after-rust-removal.md)
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 20 — Interpreter
- **Spec rules:** type.num.6, type.coll.4-6, expr.op.2, own.param.2, own.excl.3
- **Builds on:** [ADR 0359](0359-bytecode-tier-in-the-bootstrap.md), [ADR 0361](0361-bytecode-tier-design.md)

## Context

On `tools/perfbench/programs` the tier ran 1.5x to 3.8x slower than
CPython on nbody, mandelbrot, binarytrees and sieve (geometric mean 1.85x
over eight programs). The profile showed work the tier handed away or ran
through generic paths: float operators went through `binop` with two moved
values, `xs[i] = v` and `xs[i] op= v` walked a `Step` path (`slot_in`),
`xs.push(v)` built an argument vector and went through `builtin_with`,
`s[i] += expr` with any non-trivial right side was handed to the walker, and
every call with a `mut` argument was handed to the walker (3.1M hand-offs in
binarytrees, 3.75M in nbody).

## Decision

1. **Floats**: `Bin` and `BrCmp` take a float-float case in place, with
   `put_float` overwriting a float slot (type.num.6: IEEE, no traps).
2. **List elements**: `SetIndex` on an in-bounds list element writes it
   through `Rc::make_mut` directly, applying `op=` for ints inline. Out of
   bounds, maps and other values take the old path, so traps are unchanged.
3. **`push` and `len`** on a list local run inline in `MethodRun` (not under
   `--watch`).
4. **`list[i] op= rhs` with a computing right side** compiles on the tier
   for lists: `CheckIndex` traps first, as the walker reads the old value
   before the right side runs (expr.op.2), then the right side, then
   `SetIndex`. Maps and right sides that may write locals still hand off.
5. **`mut` arguments that are plain locals** compile to `CallMut`: the
   locals move into the callee's frame and back, with the overlap check
   (own.excl.3), as `invoke_inner` does for the walker. Under `--watch`
   these calls still hand off (trace.watch.3).

No `unsafe` was added and no language behaviour changed; the selftest's tier
check keeps every example, case and perfbench program byte-identical on both
tiers.

## Measurements

Linux x86-64, min of 3 wall-clock runs, `ail run` against CPython 3
(`tools/perfbench/programs`; ratio below 1 beats CPython):

| Program | CPython s | Before s (ratio) | After s (ratio) |
|---|---:|---:|---:|
| collatz | 0.822 | 0.341 (0.42x) | 0.366 (0.45x) |
| nbody | 0.312 | 0.807 (2.59x) | 0.407 (1.31x) |
| spectralnorm | 0.370 | 0.681 (1.84x) | 0.538 (1.45x) |
| mandelbrot | 0.210 | 0.645 (3.07x) | 0.228 (1.08x) |
| fannkuch | 0.670 | 0.969 (1.45x) | 0.759 (1.13x) |
| binarytrees | 0.437 | 1.662 (3.80x) | 1.178 (2.70x) |
| matmul | 0.438 | 0.704 (1.61x) | 0.768 (1.75x) |
| sieve | 0.423 | 1.083 (2.56x) | 0.822 (1.94x) |
| geometric mean | | 1.85x | 1.33x |

## Rejected alternatives

- **`lto = "fat"` with one codegen unit**: no measurable gain on sieve
  (0.826 s to 0.841 s) for a 2.5 minute build; not adopted.
- **Compiling `list[i] op= rhs` with the right side first**: changes which
  trap fires when both the index and the right side trap; kept the walker's
  order with `CheckIndex`.
