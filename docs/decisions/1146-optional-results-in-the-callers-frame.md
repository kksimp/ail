# 1146. Optional results of numbers are stored in the caller's frame

- **Status:** Accepted
- **Date:** 2026-11-18
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked on 2026-11-17 for lanes on the open slowdowns, runtime speed only, no syntax change
- **Topic:** 25 — Optimization strategy
- **Spec rules:** [native.stack.3](../spec/23-native.md)
- **Builds on:** [ADR 0574](0574-native-optional-layout.md) (`T?` is one pointer-or-0 register), [ADR 0923](0923-struct-scalar-replacement.md) (`salloc`, native.stack.1-2)

## Context

`T?` is one register: a pointer, or 0 for `none`; a number is boxed in 8
bytes. `k := pick(i) else -1` therefore allocates for every present result
and reads the box once. The round 2 runtime review measured it: `optbox`
2.6 times gcc, `optbig` 5.2 on ail-linux. The shared boxes for -1 to 1023
(#1340) brought `optbox` to 1.6 on the Mac and left `optbig` at 4.2, since
large values and floats still allocate.

## Options considered

### Option A — the caller lends a frame cell (chosen)

A MIR pass before inlining (`compiler/mir/optret.ail`). A function with a
`return` of a fresh box gets a second entry `NAME$out` with one more
parameter, the address of a cell; where the original boxes and returns, the
copy stores the number in the cell and returns the cell's address. A caller
that only tests the result against `none` and unwraps it passes a `salloc`
cell and calls the copy. Every other caller, and every function value,
keeps the original.

- **Pros:** no change to the representation of `T?`, to the calling
  convention, to register allocation or to either backend: the same code on
  arm64 and x86-64 by construction (ADR 0008). The unwrap is the load it
  already was. A small copy is a leaf, so the inliner takes it. The C twin
  of `optbig` is this shape (`int pick(long i, long *out)`).
- **Cons:** one store and one load through the frame where two registers
  would carry the pair; a second copy of each such function in the program.

### Option B — a second result register

`T?` of a number returned as (value, present) in two registers.

- **Pros:** no memory traffic at all.
- **Cons:** a call with two results in MIR, in the verifier, in every pass
  that reads calls, in the allocator and in both backends, and a second
  representation of `T?` wherever a value crosses a block edge. Weeks of
  work on the least testable layer, for a store and a load that sit off the
  loop-carried chain. To be reopened only if a measurement after Option A
  shows the gap.

### Option C — a value that marks `none`

One `int` or `float` bit pattern stands for `none`.

- **Cons:** every pattern is a value of `int` and of `float`.

### Option D — escape analysis of the box after inlining

- **Cons:** only helps a callee that is inlined; `first_at` (a loop) and
  recursive functions are not.

## Decision

1. A function qualifies when it returns an optional and has a result that is
   a call of `rt.box`, `rt.boxf` or `rt.box_bool`, or of a qualifying
   function, read nowhere but by the `return` of its own block or by a jump
   to a block that returns that parameter and reads it nowhere else. Other
   results of the same function (a parameter, a field, a kept box) are
   returned unchanged.
2. A call goes to `NAME$out` when every use of its result, through copies,
   is `== none`, `!= none`, or an unwrap. It gets its own cell; a function's
   frame holds at most 12.
3. `main`, `--values` and `--trace` builds, and callers inside the runtime
   are left alone. `AIL_OPT_OFF=optret` (or `stack`) turns the pass off.
4. Copies nothing calls are dropped.

## Soundness

- The copy's result is 0 or the address of a word holding the number, as the
  original's is; the caller's uses read nothing else of a box.
- The cell is written only by the call it was made for (the callee stores
  into it or hands it to the call whose result it returns at once, nothing
  else). The call defines the result and dominates each use, so a use reads
  the cell after the latest run of the call and before the next. A result
  that could outlive that (an edge argument, a stored or passed or returned
  value) keeps the boxed entry.
- A store into the cell is placed where the box was made, and the block then
  returns, or jumps to a block that returns; no other store into the cell
  can come between.
- The cell holds a number, never a pointer, so the collector misses nothing;
  the frame's words are scanned conservatively as before.
- Nothing is added that can trap, and no trap moves.
- Test: `tests/native/run/opt_frame_results.ail` (every scalar type, kept and
  passed and returned results, an earlier result held past a later call,
  recursion, `defer`, `?`, more than 12 calls in one function, a function
  value).

## Rationale

ADR 0076 order: no source change, so AI understanding, readability and
tokens tie; speed decides, then risk. Option A removes the allocation,
which is the measured cost, with one MIR file and no backend change.

## Consequences

- `optbig` and `optbox` should reach the C twin's shape; to be measured on
  the Mac and ail-linux before any number is quoted.
- Follow-ups: the runtime's own optional results consumed by user code
  (`to_float`, `char_at`, `find`) when their callers are user functions
  (covered where the runtime function returns a fresh box at once); results
  carried round a loop (`mut best := none`), which need a cell per variable.

## Rejected alternatives

- **Option B:** cost and risk out of proportion to a store and a load.
- **Option C:** no spare bit pattern.
- **Option D:** misses every callee that is not inlined.
