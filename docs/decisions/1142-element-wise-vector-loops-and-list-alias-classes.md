# 1142. Native: element-wise loops in two lanes, and one alias class per list

- **Status:** Accepted
- **Date:** 2026-11-17
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked for the changes the
  speed reviews found, without syntax changes (2026-11-17)
- **Topic:** 25 — Optimization strategy
- **Spec rules:** [native.vec.4, native.fwd.4](../spec/23-native.md)
- **Extends:** [ADR 0890](0890-optimizer-roadmap-to-match-c.md) (its passes 4 and 6), [ADR 1066](1066-pair-adjacent-float-operations.md),
  [ADR 1050](1050-dominated-checks-and-field-alias-classes.md)

## Context

The round-2 optimizer review (`perf-review/round2-optimizer.md`, items 3 and 4) found that AIL proves something C
compilers can't assume, and does not spend it: two lists never share part of their elements. A list's elements are a
block of its own; `push` may move the block but never shares it, and value semantics copy a list before a second name
can write it. For `c[i] = a[i] * k + b[i]` through three pointers, gcc -O2 must prove the pointers apart or add a
run-time overlap test, and its -O2 cost model takes neither. In AIL the loop needs no test.

On main 855a6a78 nothing used this. Load forwarding (`forward.ail`) forgot every list's elements at any element
store, a loop header started with nothing known, nothing moved a load out of a loop except the three special cases of
`hoist.ail`, and the only vector passes were for one shape each (escape loops, reduction nests, pairs inside one
block, fill).

## Decision

1. **Which lists are apart (`alias.ail`).** Two list values, with copies and `data` taken off, are different lists
   when they are different values, each a parameter or a list the function made (`rt.list_new`, `rt.list_new_for`,
   `rt.list_copy`), and at least one was made here or both are parameters. Two parameters can only both be written if
   both are `mut`, which own.excl.1 forbids to overlap; a read-only one that overlaps a `mut` one is passed as a copy
   (own.excl.4). A list read from a field, an element or a call's result could be any list, and the runtime's own
   functions are called outside own.excl, so neither is ever apart.
2. **One alias class per list in load forwarding (native.fwd.4).** An element store forgets the elements of every list
   not known apart from the stored one, instead of all of them.
3. **Loop headers keep what the loop can't write.** A header with one preheader starts with what the preheader ended
   with, less what any instruction of the loop may write, so a value read before the loop serves the loop.
4. **Loads a loop repeats move to the preheader (`licm.ail`).** A field read of a struct or tuple, and a list's `len`,
   `cap` or `data`, whose operand is defined before the loop and is an object on every path (a parameter, a call's
   result, a value read from a field or element), and whose memory nothing in the loop may write. Such a read can't
   trap, so it may run when the loop body would not have. An operand unwrapped from an optional (a copy or a block
   parameter) may be 0 there and stays put; enum values, which may be plain numbers, too.
5. **Element-wise loops in two lanes (native.vec.4, `vecmap.ail`).** A check-free counted loop whose one-block body
   only loads `float` elements, does `fadd fsub fmul fdiv`, and stores `float` elements, every access at a counter
   that steps by 1 with no offset, gets a loop in front of it that does two iterations per pass with the `Vec2`
   operations of ADR 1066; the original loop does the rest. Counters other than the loop's own are the lengths
   `push` writes at once `pushes.ail` keeps them in registers, so comprehensions qualify. Accesses at two different
   counters need the lists apart (item 1). Accesses at one counter need nothing: the same counter value and offset 0
   give the same element or elements of different lists.
6. **Exact results.** Each lane does its scalar operation with its operands in order, never fused. A loop that
   carries a `float` (a sum) is not taken, so no float reduction is reassociated. Nothing in the body can trap, and
   the counters can't overflow (`i + 2` runs only after `i + 1 < bound`).
7. **Knobs.** `AIL_OPT_OFF=vmap`, `fwdalias`, `fwdloop`, `fwdhoist`.

## Rejected alternatives

- **Unroll by two and let `slp.ail` pair the copies** (the review's first slice). Pairing must move the second
  iteration's loads above the first one's store, which `slp_legal` refuses unless base and index are equal; teaching
  it item 1 and adding an unroller is more code than building the vector body directly for the one shape.
- **A run-time overlap test, as gcc -O3 adds.** Never needed: the language rules it out.
- **Four lanes on x86-64-v3 now.** Needs a 256-bit value type, its register allocation and VEX encodings on one ISA
  only; ADR 0008 asks for both ISAs in step. Kept for later, with `int` lanes (overflow mask before the store) and
  `u8` lanes.
- **Hoisting element loads (`get`, `ldx`) out of loops.** An element read may be out of bounds when the loop runs
  zero times; it moves only with its check and a proof the loop runs, which is a later step.
- **Treating call results as apart.** A function may hand back the list it was given.

## Result

To be filled from the runner's timing job (`perf-review/round2/axpy.ail`, a comprehension twin, C at -O2 and -O3).
