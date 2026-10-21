# 0625. Address folding, and saving callee-saved registers where the calls start

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** native.build.1 (output unchanged)
- **Builds on:** [ADR 0577](0577-bounds-checks-in-counted-loops.md) (`ldx`/`stx`), [ADR 0622](0622-loop-starts-steps-and-index-spans.md)

## Context

Once a loop's checks are gone (ADRs 0577, 0622), an access `s[bi + 3]` is still
a copy, an `add` and a load: nbody did that for most of its 40 element
accesses per step.

## Decision

`compiler/mir/addr.ail` runs after `bce`. An unchecked `ldx`/`stx` whose index
is `y + k` or `y - k` (an `add`/`sub` with no overflow check, constant `k`,
total at most 4095 elements) takes `y` as its index and carries the offset.
The offset is a whole number of elements kept in the instruction's `fimm`,
which no integer instruction uses (`Inst.elem_off()`), so `Inst` keeps its
fields and queued PRs that build one still compile.

- x86-64: the displacement of `[base + y*8 + 8k]`.
- AArch64: no displacement on the indexed form, so one `add`/`sub` into the
  scratch register replaces the one removed (neutral); constant indexes fold
  completely.

## Shrink-wrapping

A function such as `check` (`if left[node] < 0: return 1`, else recurse)
pushed and popped all five callee-saved registers on its leaf path, which is
half of its calls. Now:

- `codegen.split_for_calls` finds the deepest block that dominates every
  block with a call, has no parameters, and whose dominated blocks form a
  region that nothing leaves and no edge re-enters at the block itself (a
  loop header fails; the block before it works). At its start, each value
  defined before it and used in it is copied, and the uses there read the
  copy; the allocator gives the copy a callee-saved register and the
  original, which never crosses a call, a caller-saved one.
- The allocator finds the deepest such block that dominates every block a
  callee-saved register lives in (`Alloc.wrap`, `Alloc.region`). If it is not
  the entry, the registers are saved there (`mov [rbp - 8k]` on x86-64, the
  same frame slots a push would use; `str` on AArch64) and restored only by
  returns in its region. Returns outside it save and restore nothing.
- Cost: `ail build` hello is about 5% slower (a dominator pass per function
  with calls).

binarytrees 14: 340M to 335M instructions; the leaf path of `check` loses
eight memory operations. The rest of binarytrees is list growth.

## Measurements

nbody 100000: 120.2M to 104.7M instructions (163.0M at the start of the lane).

## Rejected alternatives

- **A new `off` field on `Inst`:** every other lane's `Inst(...)` literal
  would stop compiling after the merge.
- **Folding checked accesses (`get`/`set`):** their bounds check needs the
  index with the offset applied.
