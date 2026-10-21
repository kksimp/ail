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

After merging the plan branch (ownership, `ref` cells, tracing, tasks, thread pool), wall time was interp 0.093 s vs 0.088 s (1.06) and loops 0.244 s vs 0.232 s (1.05); the plan-branch binary took 0.178 s and 0.66 s on the same two. The fast paths step aside when a program has unique types or runs with `--watch`.

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

## Round 3: where the time goes now (lane 61)

Profile of `examples/interp.ail` (callgrind, 783M instructions, plan branch
at ecd6d75), self cost of the top ten, as a share of the run:

| # | Cost | Share | What it is |
|---|---|---|---|
| 1 | `eval` | 9.1% | the hot expression arms and the leaf reads |
| 2 | `method_call` | 5.7% | site-cache lookup, receiver kind, `ref` cell and handle checks |
| 3 | malloc, free | 5.9% | struct and enum construction allocates twice (`Rc` and its `Vec`) |
| 4 | `stmt` | 5.1% | statement dispatch, `let`, watch and `else` checks |
| 5 | `eval_general` | 4.4% | construction, `match`, interpolation |
| 6 | `invoke` | 3.2% | operand preparation, env pool |
| 7 | `eval_binary` | 3.0% | operators that are not int-on-int |
| 8 | `block` | 2.8% | scope marks, task and unique checks |
| 9 | `Value` drop glue | 2.3% | releasing `Rc` fields when an environment is cleared |
| 10 | `run_body`, `matches` | 3.4% | frame set-up, enum pattern tests |

No entry is above 10%. The next step per cost is not another local fast
path: a name scan is about 5% inside `eval`, one hash lookup per site is
under 3%, and removing the float-literal probes from the literal arms saved
0.4% on `interp` and 3% on a collatz loop. To get from parity to the
1.5x-faster target the work has to remove whole layers (name search, `Flow`
results, `Rc` indirection and per-node dispatch together), which is what a
compiled form does.

Measured and rejected this round: fat LTO with one codegen unit (collatz 12%
faster by wall time, `interp` 8% slower, so no net win).

Kept: a one-word mask over the lines that hold sized or float-typed literals,
so the common int literal skips two table probes ([ADR 0297](../decisions/0297-no-bytecode-tier-in-the-oracle.md)).

## Front end (lane 81)

Agents and the editor run `ail check`, `ail api` and `ail effects` after
nearly every edit, so their latency matters more than one long run
([ADR 0332](../decisions/0332-front-end-allocator.md)–[0334](../decisions/0334-fast-hashing-and-lexing-in-the-front-end.md)).
Wall time: best of 15, base and new binaries run alternately, directly (not
through `./ail`), Linux x86-64, plan branch at c9799b5. Instructions:
callgrind on a release build with debug info.

| Command | Before ms | After ms | Speed-up |
|---|---|---|---|
| `ail check tools/edit/edit.ail` (4,800 lines) | 33.0 | 26.4 | 1.25x |
| `ail api --symbols tools/edit/edit.ail` | 40.4 | 33.5 | 1.20x |
| `ail api tools/edit/edit.ail` | 38.6 | 29.8 | 1.29x |
| `ail effects tools/edit/edit.ail` | 35.4 | 27.3 | 1.29x |
| `ail check examples/sheet.ail` | 5.0 | 4.6 | 1.08x |
| `ail check examples/json.ail` | 3.6 | 3.3 | 1.08x |
| `ail check examples/hello.ail` | 1.7 | 1.8 | 0.95x |
| `ail version` (process start) | 1.3 | 1.3 | 1.00x |
| `ail test tools/edit/edit.ail` (system allocator) | 86.6 | 82.7 | 1.05x (noise) |

Where `ail check tools/edit/edit.ail` spent its instructions (inclusive):

| Phase | Before | After |
|---|---|---|
| Total | 163.3M | 106.3M (-35%) |
| Lex (edit.ail, `std.ui.widgets`, `themes`) | 45.1M | about 31M |
| Lex the prelude, std declarations | 4.6M | about 3M |
| Parse | 37.0M | 24.6M |
| Resolve modules | 26.9M | 12.7M |
| Check | 43.8M | 32.2M |
| Process start (`ail version`) | about 0.4M | same |

Before the change, `malloc` and `free` took 25%, SipHash 8%, and the lexer's
keyword and symbol search (a `String` per word, compared with every keyword)
about 6%. Wall time was a third kernel: about 3,350 page faults on fresh heap.
After: page faults about 150 (huge pages), no SipHash in the front end.

The prelude and the standard modules cost about 0.3 ms of a 1.7 ms `ail check
hello`, so an on-disk cache of them would save little
([ADR 0333](../decisions/0333-no-front-end-disk-cache-yet.md)). The target
was 2x on the editor check; this round reached 1.25x wall and 1.54x
instructions. Next costs, in order: the lexer turns each line into a
`Vec<char>` (lexing is a third of what remains), the checker clones `Ty`
values, and every name is a `String`; each needs a change across `lexer.rs`,
`parser.rs` and `check.rs`.

Run programs (`ail run`, system allocator) are unchanged within noise:
interp 120 → 116 ms, json 6.6 → 6.6, sheet 13.7 → 14.2, loops 277 → 288.
`bootstrap/tests/front_end_alloc.rs` checks that `ail check` and `ail api
--symbols` print the same bytes with and without the front-end allocator
(`AIL_SYSTEM_ALLOC=1`) on every example and test case.

## Bytecode tier, slice 1 (lane 111)

[ADR 0359](../decisions/0359-bytecode-tier-in-the-bootstrap.md) added a
second evaluator; [ADR 0361](../decisions/0361-bytecode-tier-design.md)
records its design. A function's body compiles on its first call to register
code over the walker's own environment; anything outside the compiled subset
is handed to the walker. Slice 1 compiles locals, int and comparison
operators (any other operand types go through the walker's `binop`), `and`,
`or`, `not`, unary `-`, `if` with `if name := e` branches, `while`, `for`
over ranges, lists, sets and channels, `break`, `continue`, `return`, calls
of functions without `mut` parameters, `print`, field reads, list and map
reads, and `local[i] = v` and `local[i] op= v`.

Stack and register code were compared first on the collatz section of
`loops` in a standalone prototype (instructions, callgrind): walker 2,765M,
stack code 2,707M, register code 1,062M, register code with immediates 900M.
The table is in ADR 0361.

Results (Linux x86-64, 4 cores, shared machine). Wall: min of 11 runs of
the release binary; walker is `--no-bytecode`. Instructions: callgrind, stable.
`collatz` is `loops`'s first section alone; its Python twin is the same loop.

| Program | Walker s | Tier s | CPython s | Walker / CPython | Tier / CPython | Walker instr | Tier instr | Gain |
|---|---|---|---|---|---|---|---|---|
| interp | 0.125 | 0.128 | 0.109 | 1.15 | 1.17 | 812.8M | 764.3M | 1.06x |
| json | 0.009 | 0.010 | 0.048 | 0.19 | 0.21 | 30.9M | 30.9M | 1.00x |
| sheet | 0.017 | 0.019 | 0.069 | 0.25 | 0.28 | 83.8M | 83.2M | 1.01x |
| loops | 0.303 | 0.180 | 0.291 | 1.04 | 0.62 | 3,947.9M | 2,204.3M | 1.79x |
| collatz | 0.235 | 0.107 | 0.452 | 0.52 | 0.24 | 2,765.4M | 1,083.8M | 2.55x |

What runs on the tier (`AIL_BYTECODE_STATS=1`): all of `loops` (30,000 calls,
320,000 hand-offs for its method calls, `else` and interpolation); `interp`
89,263 calls on the tier and 135,591 on the walker, with 290,000 hand-offs.
`interp`, `json` and `sheet` are mostly method calls, struct construction,
`match` and interpolation, which slice 3 compiles. In the collatz loop the
remaining cost is operand decoding and bounds checks (about 20% each).

Byte-identical check: every example, test case, corpus program, perfbench
program and package (715 runs of `ail run` and `ail test --json`) prints the
same stdout and stderr and exits the same with the tier as `main` did
without it, except the two programs that print the clock and unseeded random
numbers.

## Bytecode tier, slice 3 (lane 111)

[ADR 0363](../decisions/0363-bytecode-tier-methods-and-data.md) compiles
method calls (a guard on the receiver's value, then the user or built-in
method directly), calls through a trait (the walker's `dispatch`, cached by
type), struct and variant construction, list, map and set literals, `match`,
interpolation, `?` and `else`, and `local.field = v`. String `==` skips the
trait dispatch when no `Eq` implementation applies to `str`.

Results, measured as for slice 1:

| Program | Walker s | Tier s | CPython s | Walker / CPython | Tier / CPython | Walker instr | Tier instr | Gain |
|---|---|---|---|---|---|---|---|---|
| interp | 0.140 | 0.091 | 0.113 | 1.24 | **0.81** | 818.2M | 600.0M | **1.36x** |
| json | 0.010 | 0.009 | 0.052 | 0.19 | 0.17 | 31.2M | 27.4M | 1.14x |
| sheet | 0.019 | 0.019 | 0.065 | 0.29 | 0.29 | 84.3M | 75.6M | 1.11x |
| loops | 0.323 | 0.163 | 0.312 | 1.04 | **0.52** | 3,952.7M | 1,874.5M | **2.11x** |
| collatz | 0.232 | 0.103 | 0.438 | 0.53 | 0.24 | 2,765.4M | 1,058.5M | 2.61x |

The self-hosted front end (lane 114), on `tools/edit/edit.ail`. It has no
Python twin. Wall time is the min of 5 runs.

| Run | Walker s | Tier s | Walker instr | Tier instr | Gain |
|---|---|---|---|---|---|
| `ail run compiler parse` | 2.70 | 1.92 | 16,359M | 12,438M | 1.32x |
| `ail run compiler check` (its `ail api --ast --program` dump) | 4.14 | 3.05 | 27,835M | 20,783M | 1.34x |

The walker is 0.7% slower on `interp` than in slice 1 (818.2M against
812.8M instructions), from splitting its method call so the tier can enter
it after the receiver. `json` and `sheet` are mostly start-up: the front end
is about 25M of their instructions.

What runs on the tier (`AIL_BYTECODE_STATS=1`; instructions the tier ran,
hand-offs counted as one each):

| Program | Functions compiled | Calls on the tier | Instructions | Hand-offs | Native |
|---|---|---|---|---|---|
| interp | 50 of 50 | 224,854 of 224,854 | 3,465,174 | 22,722 | 99.3% |
| json | 30 of 30 | 5,242 of 5,242 | 94,251 | 228 | 99.8% |
| sheet | 49 of 50 | 10,282 of 10,449 | 185,927 | 1,638 | 99.1% |
| agent corpus (10 programs) | 76 of 79 | 3,922 of 3,950 | 66,096 | 309 | 99.5% |
| compiler parse | 172 of 172 | 2,843,288 of 2,843,288 | 68,375,350 | 71,821 | 99.9% |

The agent corpus is the ten agent-tool programs of ADR 0217, run from
`examples/` on their data. The hand-offs that remain are field and element
assignments deeper than one step (`env.scopes[k][name] = v`, 16,620 in
`interp`), comprehensions, slices, `else` blocks that bind the error, and the
functions the tier leaves whole to the walker.

On the tier no single cost dominates any more. On `interp`, the
interpreter loop's own work is 25%, bounds checks 6%, value drops 5%,
allocation 7%, and call set-up (`run_body`, register files) 10%. The parser
adds the walker's `to_str` for interpolation (5%). A further large step needs
frames without names (ADR 0361's Option B).

## Bytecode tier, slice 4 (lane 111)

[ADR 0364](../decisions/0364-bytecode-tier-limits-coverage-and-traces.md)
lets the tier run under `--timeout`, `--max-memory`, `--max-steps`,
`--coverage` and `--trace`. A `Hit` instruction before each statement, and at
each `while` turn, calls the walker's `hit`. Call and effect traces come from
code both tiers share. Only `--watch` still keeps a run on the walker.
Before this slice these runs used the walker alone.

Callgrind instructions, walker then tier:

| Run | Walker | Tier | Gain |
|---|---|---|---|
| `loops` | 3,952.7M | 1,874.5M | 2.11x |
| `loops --max-steps 1000000000` | 4,784.6M | 2,595.5M | 1.84x |
| `loops --trace=calls` | 3,961.5M | 1,883.1M | 2.10x |
| `interp` | 818.2M | 600.0M | 1.36x |
| `interp --max-steps 1000000000` | 876.0M | 648.2M | 1.35x |
| `interp --trace=calls` | 864.1M | 646.1M | 1.34x |

A step limit costs `loops` 38% on the tier and 21% on the walker. `loops` is
nearly all one-line statements, and `hit` is the same call on both tiers.


## Watch on the bytecode tier (lane 142)

[ADR 0471](../decisions/0471-watch-on-the-bytecode-tier.md) lets `--watch`
run on the tier. Assignments, `mut` bindings and mutating method calls hand
off to the walker, which shows each watched value; the rest of the function
stays on the tier. Watch-off runs cost the same instructions as before
(callgrind; before = the batch branch at 037253b, after = this change):

| Program | Before (instr) | After (instr) | Change |
|---|---:|---:|---:|
| loops, no watch | 1,874,659,459 | 1,874,657,257 | 0.0% |
| parsum, no watch | 4,467,660,568 | 4,467,737,128 | +0.002% |
| interp, no watch | 600,063,418 | 600,065,188 | +0.0003% |

With `--watch total` (before = walker, the only option before this ADR):

| Program | Walker (instr) | Tier (instr) | Tier / walker |
|---|---:|---:|---:|
| loops | 7,537,076,827 | 6,846,562,238 | 0.91 |
| parsum (`par for`; the watch order varies run to run, so output is not compared) | 21,646,829,236 | 18,913,573,217 | 0.87 |

The gain under `--watch` is smaller than without it, because the watched
assignments and mutating calls are the walker's own code.


## Typed fast paths (lane 190)

[ADR 0618](../decisions/0618-bytecode-typed-fast-paths.md): float operators,
list element updates, `push`/`len`, `list[i] op= expr` and calls with `mut`
arguments no longer take generic paths or hand off to the walker. Geometric
mean over the eight compute programs: 1.85x of CPython before, 1.33x after
(nbody 2.59x to 1.31x, mandelbrot 3.07x to 1.08x, binarytrees 3.80x to 2.70x,
sieve 2.56x to 1.94x); the table is in the ADR. Agent programs are unchanged.

[ADR 0619](../decisions/0619-bytecode-calls-loops-and-fusion.md) cuts the
per-instruction and per-call overhead (lean calls, loop shapes, fused
`push`/`len` and `a[i][k]`): geometric mean 1.88x of CPython at the batch head,
1.08x now; fannkuch 0.91x, nbody 1.06x, mandelbrot 1.07x, matmul 1.15x, sieve 1.24x,
spectralnorm 1.33x, binarytrees 2.15x (calls and `mut`
arguments). What is left is the cost of reading a local: the walker's
environment is a vector of (name, value) pairs behind a pointer, so every
local read reloads its length and bounds-checks it. A frame with locals in the
register file would remove that; it is the next step.
