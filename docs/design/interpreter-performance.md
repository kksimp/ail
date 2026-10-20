# Interpreter performance

Non-normative. Measurements for [ADR 0175](../decisions/0175-interpreter-fast-paths.md),
[ADR 0176](../decisions/0176-keep-name-based-locals.md) (round 1) and
[ADR 0219](../decisions/0219-interpreter-speed-round-two.md)–[0220](../decisions/0220-interpreter-site-caches.md)
(round 2, at the end). The interpreter is
the permanent oracle ([ADR 0032](../decisions/0032-keep-rust-interpreter-as-oracle.md)),
so every change had to keep it simple and leave output byte-identical.

## How to measure

`tools/perfbench/run.sh [RUNS]` builds the bootstrap and prints the min and
median wall time of `examples/interp.ail`, `json.ail`, `sheet.ail` and
`tools/perfbench/programs/loops.ail` (integer loops, list pushes, map updates,
string building). It also times the matching Python program (the tokbench
corpus twin, or `programs/NAME.py`) and prints `ratio`: AIL's min over Python's
min, so below 1 beats CPython. It times the binary directly: the `./ail`
wrapper's toolchain probe used to add about 0.1 s ([ADR 0221](../decisions/0221-ail-skips-toolchain-probe.md)).
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

## Round 2: beating CPython (lane 36)

Goal: AIL at or below CPython's wall time on the benchmark programs. Python
3 on the same machine (min of 15 runs, process start-up included on both sides;
CPython's is about 17 ms).

### Where the time went (before round 2)

callgrind on `interp.ail` (1,383M instructions) and a 3,000-start collatz
loop (397M):

- A call to `eval` for every operand, even names and literals, and a `Value`
  built and dropped for every int operation (collatz: 1,700 instructions per
  loop iteration).
- 1.46 million allocations in `interp.ail`: a call environment, an operand
  vector and an argument vector per call; a path vector per place; a field
  vector per struct; an empty list per unit variant; a copy of every string
  literal; a pair vector per struct construction to work out which field each
  argument sets. About 17% of the instructions were `malloc` and `free`.
- Hashing: three string-keyed lookups per call, three per method call, the
  checker's call table per method call; SipHash on every `Map` key.
- Trait dispatch (`dispatch`, a table probe and name scan) on every `==`,
  ordering and text conversion of a char, string or enum, though the program
  implemented none of the traits.
- Locals: counters showed 1.7 million lookups scanning 2.7 entries on average,
  about 5% of the run. Below ADR 0176's threshold, so still name-based.

### Changes kept

Each was kept only with a measured drop in instructions on `interp.ail` or the
collatz loop, and byte-identical output (stdout, stderr and exit code) on every
example and test case against the build before it. Percentages are of the
instruction count at that step.

| Change | Effect |
|---|---|
| Int locals and literals read in place; `n % 2`, `3 * n`, conditions as `bool`; `/ %` inline | collatz −45% |
| Call environments pooled; no-`mut` calls skip places and overlap check | interp −12% |
| Inline `Value::clone`; string literals shared | interp −3% |
| `mut` operands move into the callee environment: no operand or value vectors | interp −7% |
| `return` value kept in the interpreter (`Flow` 48 to 32 bytes); field and index reads in own frames | collatz −4% |
| Int locals overwritten in place; `if` statements skip `eval` | collatz −8% |
| Place roots without a path vector; read-only receivers and arguments by value | interp −7% |
| No trait dispatch when the program defines none of `Eq`, `Ord`, `ToStr` | interp −2% |
| Per-site caches: call target, method resolution, field slots (ADR 0220) | interp −10% |
| Field and index reads borrowed; shared empty field lists | interp −3% |
| `local.field = e` without a place | interp −2% |
| `mut` operands that are plain locals moved without places | interp −8% |
| Fast integer hashing for map keys; pooled builtin argument lists | interp −2%, list push loop −12% |

### Tried and dropped

Two inline steps in `Place` (+1.5% on `interp.ail`), fat LTO with one codegen
unit (no change in instructions, build 60 s instead of 25 s), `#[inline(never)]`
on the cold statement arms and assignment inside `block` (more instructions on
collatz), reading int operands two operators deep (no gain), a hand-written
allocator (allocation is already down to 0.37 million calls).

### Results (Linux x86-64)

| Program | Instructions before → after | Wall, AIL vs CPython (min of 15) | Ratio before → after |
|---|---|---|---|
| interp | 1,383M → 739M (−46.5%) | 0.097 s vs 0.096 s | 1.95 → 1.01 |
| json | 42.2M → 28.5M (−32%) | 0.008 s vs 0.041 s | → 0.20 |
| sheet | 130.5M → 82.7M (−37%) | 0.014 s vs 0.055 s | → 0.25 |
| loops | 6,736M → 3,629M (−46%) | 0.249 s vs 0.244 s | 2.85 → 1.02 |

Ratio is AIL min / CPython min; below 1.0 means AIL is faster. Instruction counts come from callgrind and are stable; wall time varies a few percent.

### Which programs beat CPython, and why not the rest

json and sheet beat CPython clearly. interp and loops are at parity (within noise), not ahead.

Where loops still loses: the map-update section (AIL about 0.027 s against CPython's 0.011 s) and the string-build section, where `s = "{s}x"` copies twice and CPython appends in place when it holds the only reference. The collatz section beats CPython (0.19 s against about 0.21 s) and list push/sum ties. The remaining profile is flat: name lookup is about 5% of instructions, so the name-based environment of ADR 0176 stays. Closing the last gap needs resolved local slots or a bytecode tier, which is a separate decision.

### Left alone

Slot-resolved locals (about 5% of a run, [ADR 0176](../decisions/0176-keep-name-based-locals.md)),
`Rc<[Value]>` struct fields (halves struct allocations but touches every
builtin), a 24-byte `Value`, in-place string appends (`s = "{s}x"` copies the
string, which CPython avoids by reusing its buffer when the count is one), and
the bytecode form, which belongs in Stage 3 and not in the oracle.
