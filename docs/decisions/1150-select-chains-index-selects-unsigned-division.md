# 1150. Select chains, unlooped index selects and unsigned constant division

- **Status:** Accepted
- **Date:** 2026-11-18
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked on 2026-11-17 for lanes on the open slowdowns against C, runtime speed only, no syntax change
- **Topic:** 25 — Optimization strategy
- **Spec rules:** [native.select.1, native.range.3](../spec/23-native.md) (both amended; no new rule)
- **Amends:** [ADR 1141](1141-branch-free-selects-with-guards.md) (which branches become selects)
- **Extends:** [ADR 1063](1063-function-wide-value-ranges.md) (division of a value never negative)
- **Builds on:** [ADR 0816](0816-native-codegen-entry-addressing-constants.md) (`mark_nonneg`), [ADR 0623](0623-branch-and-codegen-peepholes.md)

## Context

fasta is the weakest published benchmark: on the Mac, main `8a636a55`,
100 million bases take 1.252 s against 0.623 s for the C twin (clang -O2),
with 93 instructions a base against 31
(`hotpaths-dis-darwin-8a636a55b.txt` in the project library). The
disassembly shows where:

1. The base is picked by a three-level `if`/`elif` on a random value, three
   data-dependent branches. ADR 1141's pass skipped it twice over: it
   converts one two-way branch, not a chain, and it leaves any branch whose
   picked value feeds a list index (`counts[base]`), a rule added for the
   `vm` benchmark, where the picked value is the next `pc`.
2. `seed % 139968` and `seed * 100 / 139968` use the signed
   multiply-and-shift: `smulh; add; asr; lsr 63; add`, five instructions on
   the loop-carried chain, where clang, knowing the seed is never negative,
   uses `umulh; lsr`.
3. `print(line.join(""))` costs 0.23 s of the 1.25: each character goes
   through `char_put` and reloads the output buffer's length.
4. Each `push` jumps over its inline grow call, and `[0, 0, 0, 0]` tests
   the capacity before every element.

## Decision

1. **Chains.** The select pass runs up to four rounds. A converted level is
   a block of comparisons and selects that jumps to the join, so the next
   round takes it as an arm of the level outside it. A `select` is free to
   run early, and an arm counts its comparisons, not its selects. Only the
   first round adds range guards, so a guard's slow path (the original
   checked branch) is never converted again.
2. **Index selects.** A branch that picks a value an index is computed from
   keeps its branch only when the pick feeds itself: when the branch's
   condition or one of its two values is computed, through any instructions
   and block parameters, from the join's parameter. That is the `vm` case
   (the next `pc` is read through the last one). An index that only reaches
   its element, as in fasta's table of counts, is picked with a select.
   `AIL_OPT_OFF=selchain` restores ADR 1141's rule.
3. **Division.** `mark_nonneg` and the range pass mark `x / c` and `x % c`
   for a constant `c >= 3` that is not a power of two when `x` is never
   negative (`nnm`). Both back ends then emit the unsigned high multiply by
   the same constant (`umulh`, `mul`) and a logical shift. With
   `m = 1 + floor(2^(63+l) / c)` read as signed, `mulhs(m, x) + x` is exactly
   `mulhu(m, x)` for `x >= 0`, and the sign term `x >>> 63` is 0, so the
   quotient is the one ADR 0816's sequence gives.
4. **No rule needed** (code generation only): a full list's grow call is
   emitted after the function's blocks on both back ends; a list literal's
   pushes carry the room-guaranteed form of native.bce.6, since
   `rt.list_new(n)` made room for all `n`; x86-64 uses the two-byte form for
   a jump back within a byte's reach and reads a spilled second operand of a
   comparison, `add`, `sub` or `imul` from its slot; `print(xs.join(""))` of
   characters writes each run of ASCII a byte a character.

ADR 0076 order: no source change and no diagnostic changes, so AI
understanding, readability and tokens tie; speed decides.

## Consequences

- fasta's pick is three comparisons and three selects with no branch on the
  data, and its seed chain is `mul; add; umulh; lsr; msub`.
- A well predicted chain now runs up to four comparisons and selects where
  it ran one predicted branch. That costs a few instructions off any
  loop-carried chain; the `vm` rule still guards the case where it costs
  latency.
- Measurements: see `docs/status.md` (this ADR's line) once the Mac and
  Linux runs are in.
- Follow-ups: `madd` for a multiply that feeds one add (one cycle on
  fasta's chain); the comparison of a quotient through its dividend
  (ADR 1114, unmerged, measured as no gain before selects existed).

## Rejected alternatives

- **Lift the index rule outright** (`AIL_OPT_OFF=selindex`): brings back the
  23% `vm` regression ADR 1141 measured.
- **A profile-driven filter:** needs a profiling run per build; the
  feeds-itself test separates the two measured cases with no profile.
- **A `switch`-like table for constant chains:** a load per pick and a
  table per chain; three selects are shorter and have no memory dependence.
- **A separate unsigned magic constant per divisor** (the shortest shift,
  as clang prints): needs 128-bit arithmetic in the compiler for the same
  two instructions.
