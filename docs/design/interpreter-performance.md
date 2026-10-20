# Interpreter performance

Non-normative. Measurements for [ADR 0175](../decisions/0175-interpreter-fast-paths.md)
and [ADR 0176](../decisions/0176-keep-name-based-locals.md). The interpreter is
the permanent oracle ([ADR 0032](../decisions/0032-keep-rust-interpreter-as-oracle.md)),
so every change had to keep it simple and leave output byte-identical.

## How to measure

`tools/perfbench/run.sh [RUNS]` builds the bootstrap and prints the min and
median wall time of `examples/interp.ail`, `json.ail`, `sheet.ail` and
`tools/perfbench/programs/loops.ail` (integer loops, list pushes, map updates,
string building). It also times the matching Python program for reference.
Wall time on a shared machine is noisy, so each change was judged by
instruction counts from `valgrind --tool=callgrind` on a release build with
debug info, which are stable to a fraction of a percent.

## Results (Linux x86-64)

| Program | Before: min s | After: min s | Before: instr | After: instr |
|---|---|---|---|---|
| interp | 0.225 | 0.149 | 1,687M | 1,346M |
| json | 0.009 | 0.009 | not measured | not measured |
| sheet | 0.020 | 0.016 | not measured | not measured |
| loops | 0.912 | 0.587 | not measured | not measured |
| collatz only (3,000 starts) | n/a | n/a | 516M | 393M |

`json` and `sheet` finish in under 20 ms, so process start-up dominates them.
Python (reference only): interp 0.10 s, json 0.04 s, sheet 0.05 s.

## Where the time went

Profile of the baseline (callgrind):

- `eval` had a stack frame over 1.6 KB, and most evaluations are names,
  literals and operators. About 35 instructions of every call were frame setup.
- Position-keyed and name-keyed tables (`float_literals`, `variants`, `calls`,
  `fns`, `methods`) were hashed with SipHash on every integer literal and
  call: about 10% of `interp.ail`.
- Every call allocated a clone of each argument's place for the overlap check
  and a vector of final values even with no `mut` parameter.
- Name lookup (`lookup`, a reverse linear scan) was about 3% including its
  string compares. Values were already `Rc`-shared (strings, lists, maps,
  structs), so there was no deep clone to remove.

## Changes kept, in order

1. `eval` split into a small hot function and `eval_general` (collatz −4%).
2. Fast hasher for the lookup tables, and traps boxed inside `Exit`
   (interp −10%, collatz −4%).
3. Overlap check on the prepared operands, results vector only for `mut`
   parameters, preallocated environment (interp −6%).
4. Inline int arithmetic and comparisons; plain-local assignment without a
   `Place` (collatz −17%).

## Left alone

Slot-resolved locals ([ADR 0176](../decisions/0176-keep-name-based-locals.md)),
and in-place string appends for `s = "{s}x"` (quadratic, but only visible for
strings of tens of thousands of characters). Next steps if more speed is wanted
are caching call targets per call site and a bytecode form, which belongs in
Stage 3 rather than in the oracle.
