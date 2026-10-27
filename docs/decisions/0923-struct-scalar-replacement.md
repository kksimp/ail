# 0923. Native: scalar replacement of struct objects that stay in one function

- **Status:** Accepted
- **Date:** 2026-10-26
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.sroa.1](../spec/23-native.md)
- **Builds on:** [ADR 0391](0391-native-backend-optimisations.md) (inlining), [ADR 0392](0392-native-structs-and-methods.md) (struct objects), [ADR 0890](0890-optimizer-roadmap-to-match-c.md) (optimizer roadmap)

## Context

The big-program race put `raytrace` (N = 4000) at 9.7 s against 1.75 s for the
C twin built with gcc -O2 (5.5x, the worst of the four programs). perf on
ail-linux: 55% of cycles in `rt.alloc_z`, `rt.gc_sweep`, `rt.alloc` and
`rt.obj_new`; the rest in `hit`, `Vec.sub`, `Vec.scale` and `trace`. Every
`Vec` result of `add`, `sub`, `scale` and `unit` is a heap object of three
slots, and a constructor is never inlined because it calls `rt.obj_new`
(ADR 0391 inlines only functions that call nothing).

## Decision

1. **Inline allocating constructors.** Before the per-function passes,
   `inline_module` runs two rounds that inline small (at most
   `INLINE_LIMIT`) functions whose only call is `rt.obj_new` into their
   callers, then merges the join blocks. Two rounds reach `a.sub(b).unit()`.
   Where a callee returns `T` for a declared `T?`, the inliner retypes the
   value with a `copy` so the join block's parameter type matches.
2. **Scalar replacement,** `compiler/mir/sroa.ail`, per non-runtime function
   after inlining. An `rt.obj_new` with constant size and kind 4 (struct)
   qualifies when every use of the object and of its `copy`s is a slot `load`
   or `store`; no use is a call argument, a return, a jump argument or a stored
   value; every slot read is stored exactly once, in the allocating block and
   before the read, with a value of the load's type. The object, its stores
   and copies are removed and each load is replaced by the stored value.
   `AIL_OPT_OFF=sroa` turns both parts off.

The pass is a pure removal of unobservable work: no trap site, trace frame or
checked arithmetic changes.

## Measured (ail-linux, main plus this change, x86-64)

| | before | after |
|---|---|---|
| raytrace N=4000 | 9.71 s | 3.6 s (gcc -O2 1.75 s) |
| instructions | 81.6 G | 31.2 G |
| compile time (self-host) | | +4.6% |

Checksum identical; self-host reaches its fixed point. `hit` is fully
scalarized; `trace` still allocates the hit point (passed to `hit` and the
recursive call), the reflected direction, the `base` colour joined across
branches, and its result.

## Rejected alternatives

- **Value-type structs in the language** (copy in registers by definition).
  Changes the ABI of every struct and an ADR the designer made (0392); the
  escape analysis gets the same win for code that never lets a struct leave.
- **Inlining only, no replacement.** The 3.6% measured with the first draft
  (before join blocks were merged) shows the allocation, not the call, is the
  cost.
- **Scalarizing at lowering time.** Would add code to `lower.ail`'s queue
  (ADR 0823); a MIR pass sees the objects the inliner creates.
- **Partial escape analysis** (materialize the object only on the path that
  escapes) is the next step, not part of this change: it needs proof that no
  call mutates the object through a `mut` parameter before a later read.

## Consequences

Programs that build small structs and read them back run without allocation.
Code size grows by the inlined constructors (bounded by `INLINE_BUDGET`).
Follow-ups: interprocedural scalar arguments (`hit(s, p, light)`), joins of
objects through block parameters, and `mut self` field replacement (Optimizer
lane).

## Amendment 2026-10-27: joins

Rule native.sroa.2: objects that meet at a block parameter (a `mut` struct
variable assigned in a branch or a loop) form one group with the parameter.
When every member qualifies and the parameter is only read by field access, it
becomes one parameter per slot and each jump passes the slots of its object.
Found by the raytrace profile: `mut base := s.col` and loop-carried vectors
kept allocating after the first step.

## Amendment 2026-10-27: late inlining and unrolling (native.sroa.3)

After every function has been through scalar replacement, `late_inline`
(compiler/mir/inline.ail) inlines a call that is given an object the caller
just built when the callee calls nothing, takes a struct and has at most
`LATE_LIMIT` (100) instructions (`hit(sp, p, light)`), and unrolls a
self-recursive function that passes a fresh object to itself (`opt_unroll.ail`,
`UNROLL_FRESH` 400 instructions; every callee of such a function is inlined
first because unrolling refuses bodies with other user calls). Scalar
replacement runs again on the result. Raytrace N=4000 on ail-linux: 3.04 s →
2.33 s (gcc -O2 1.8 s), instructions 24.95 G → 18.43 G; `trace` doubles in
size (358 → 800 objdump lines); stage-2 +0.1%, self-build +0.7%.

Rejected: a smaller GC trigger (`MIN_THRESH` 512 KB: raytrace −6%, matmul
+8% to +52%, L1d misses unchanged, so the cache-window theory was wrong);
a filter on which constructors to inline (saved 1.5% size, cost raytrace 6%).

## Amendment 2026-10-27: stack allocation (native.stack.1)

Objects that scalar replacement cannot remove but that are only lent to calls
(`trace(..., p, r, depth + 1)`, `main`'s per-pixel direction) go in the frame.
`compiler/mir/stack.ail` computes which struct parameters each function only
reads or passes on to such parameters (optimistic fixpoint, recursion
included); an allocation of at most 128 bytes whose uses are field accesses
and such arguments becomes a new MIR op `salloc` (the address of
`imm / 8 + 1` frame words, the first holding the header `rt.alloc` would
write) plus a header store. The register allocator reserves the slots
(`Alloc.sallocs`) and both generators lower `salloc` to an address from
`rbp` / `sp`. At most 24 words per function. `AIL_OPT_OFF=stack` turns it off.
Rejected: a per-site static buffer (recursion and tasks clobber it), hoisting
the allocation out of the loop (a callee that is the same function reads the
object while the next level overwrites it), passing the fields as extra
arguments (the ABI has no float arguments beyond the eighth, and Windows has
four float registers).
