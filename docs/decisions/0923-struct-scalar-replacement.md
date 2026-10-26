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
