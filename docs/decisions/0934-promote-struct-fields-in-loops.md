# 0934. Native: struct fields kept in registers across loops

- **Status:** Rejected (2026-11-05, measured no gain)
- **Date:** 2026-11-04
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** none (`native.promote.1` was never added)
- **Builds on:** [ADR 0890](0890-optimizer-roadmap-to-match-c.md) (optimizer roadmap), [ADR 0923](0923-struct-scalar-replacement.md) (scalar replacement), [ADR 0816](0816-native-codegen-entry-addressing-constants.md) (load forwarding)

## Context

Range coders, decoders and interpreters keep their state in a struct and
change it in `mut self` methods (`Dec.bit`, `Enc.direct` in the lzr race
program, Clove's decoders). Every `rng = rng.shl(8)` is a load and a store of
the object's slot, on every pass round the loop; perf annotate of lzr shows
`mov 0x8(%rdi)` / `mov %rcx,0x8(%rdi)` pairs at the top. SROA (ADR 0923)
removes objects that never leave a function; these objects live long, so it
does not apply. Load forwarding (ADR 0816) stops at a loop's back edge.

## Decision

A MIR pass, `compiler/mir/promote.ail`, runs on every non-runtime function
after inlining, before bounds-check elimination. For each loop, outermost
first, a slot `(object, offset)` qualifies when the object (followed through
`copy`) is a struct, a `ref` of a struct or a tuple; it is defined outside the
loop and used inside only as the object of `load`/`store`; the slot holds a
number, `bool` or `char` (never a pointer, so the collector needs nothing);
no other object of the same or unknown type is accessed at that offset; and
the slot is accessed more often than the loop has barriers. A barrier is a
call other than a list push, an indirect call, or a primitive with effects.

Up to four slots per loop (the busiest), in loops of at most 24 blocks, are
then loaded in the preheader and carried round as a parameter of every loop
block. Reads become the carried value; writes replace it. Written slots are
stored on a new block on each edge that leaves the loop, before a `return`
in it, and before each barrier; every promoted slot is loaded again after a
barrier. A trap ends the program, so it needs no store. `simplify` then drops
the parameters that never change.

`AIL_OPT_OFF=promote` turns the pass off, to bisect.

## Measured

Pending (runner, ADR 0890 protocol: ail-linux and the Mac, min of 7, stage-2
compiler, before/after on lzr, vm, Clove).

## Rejected alternatives

- **Promotion in the register allocator** (treat a slot as a spillable
  value). The allocator sees one location per value and no memory; doing it
  in MIR lets every later pass (bounds checks, folding) see the scalar.
- **Only loops with no calls.** lzr's `Enc.direct` calls `shift_low` on a
  rare path; storing and reloading around the call costs only when it runs.
- **Pointer-holding slots.** A list or string field only in a register would
  hide the new object from the collector's view of the heap (the old slot
  still holds the old one); the gain is small since list headers are already
  hoisted (native.bce.11).
- **Enum payload slots.** A slot's offset means different fields in
  different variants, so a store back on exit could write past a smaller
  variant.
- **Whole-function promotion** (keep a field in a register across the
  function). A method without a loop runs its loads once per call; the gain
  then needs the method inlined into its caller's loop, which is the inliner's
  job (proposal P8).

## Consequences

Loops over `mut self` state run on registers. Each promoted slot holds one
register across its loop, capped at four per loop to limit spills. Code grows
by one load per slot before the loop and one block per exit edge that writes.

## Rejection (2026-11-05)

Not merged. Measured on ail-linux at the lane's final head (09db8282,
promote and split together, interleaved, passes on against off): every
benchmark within ±5%, lzr 0.95 (slower), and the stage-2 self-build 794 ms
(2.8%) slower. Self-host fixed point and the pass goldens were green, so the
change was correct but bought nothing. Likely cause: register holes round
loops (ADR 0848), load forwarding (ADR 0816) and struct scalar replacement
(ADR 0923) already remove most of the loads these passes target, and the
remaining ones hit in L1. Under ADR 0076 (speed is the fourth criterion,
compile time after runtime) a pass with no runtime gain and a compile-time
cost is not kept. Revisit only with a profile showing slot reloads dominating
a hot loop.
