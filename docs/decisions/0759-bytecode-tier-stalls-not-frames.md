# 0759. Bytecode tier: the cost left is stalls, not the frame layout

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 20 — Interpreter
- **Builds on:** [ADR 0619](0619-bytecode-calls-loops-and-fusion.md), [ADR 0621](0621-bytecode-bounds-checks-stay.md)
- **Decision card:** the designer chose "No unsafe (keep safe Rust)" on 2026-10-21

## Context

[ADR 0621](0621-bytecode-bounds-checks-stay.md) left day 3 a plan: put locals in the register file and make calls
non-recursive, because binarytrees (1.5x CPython), sieve (1.1-1.3x) and matmul (1.1-1.4x) were still above
1.0x. The designer kept the tier in safe Rust, so there is no unchecked access.

## What was built and measured

Each experiment is a branch, measured with callgrind (instructions, small arguments, `tools/perfbench/programs`)
and wall time (minimum of 5-9 runs on the shared box, which is noisy: a program's wall time moves by
10% or more between runs, so only differences above that are claimed from wall time).

| Experiment | Result | Kept |
|---|---|---|
| Locals and temporaries in one array (a window of the environment after the parameters; two-pass compile, nameless slots) | Instructions +1% to +7% on every program (binarytrees 368 M to 393 M), geometric mean over CPython 0.955x to 1.004x. Each call writes and drops the temporaries as environment entries; the saving per operand did not cover it. | No (`lane/190-uf-wip`) |
| Calls as frames in one driver loop (`exec` returns a call, the driver pushes a frame; one register stack) | Binarytrees 368 M to 400 M instructions, wall time the same. The driver costs about 600 instructions per call (frame push and pop, `Rc` of the code, argument and result moves, truncation); the nested `exec` it saves was about 230. | No (`lane/190-driver-wip`) |
| Hot/cold split of `exec`, per-operand-kind variants | Slower or under 2%; a cold call costs 40+ instructions, so only truly cold arms can leave the loop. | No |
| `Value::clone` with the scalar cases first | Slower on three of five micro-benchmarks. | No |
| Scalar write into a list element for `xs[i] = false` | Sieve 0.60 s to 0.73 s. | No |
| Operator-specific instructions (`Add`, `Lt`, `BrLt`, `AddA`, and the `K` forms), no runtime switch on the operator | 1% to 4% fewer instructions. | Yes |
| The end of a `while` turn (`i += step`, then the test) as one instruction, with the plain pair kept for the first test and `continue` | Sieve -11% instructions. | Yes |
| Conditional jumps kept as branches (`std::hint::black_box` on the taken target) | Collatz 0.39 s back to 0.33 s: the compiler had turned the specialised jumps into selects, so the next dispatch waited for the comparison instead of being predicted. | Yes |
| Scalars built in place: `Push`, `take`, `Move`, element reads (`Index`, `IndexOff`, `Index2`) | See below. | Yes |
| The frame's base fixed at 0 (`run_code` asserts it; callers always build the frame's own environment) | Frees a register in `exec`; counts -1% to -2%. | Yes |

The finding behind the last four rows: `exec` moved whole `Value`s (32 bytes) through the stack, and the
compiler merged the plain-data variants of the clone into byte, word and dword copies, followed by 16-byte
loads of the same bytes. Those loads cannot be forwarded from the earlier smaller stores and stall for
about a dozen cycles each. Building the common scalars (`Int`, `Float`, `Bool`) in registers and writing
them straight to their slot avoids it. This does not show in instruction counts (the counts barely move), only in
wall time, and it is a property of the generated code, so it changes with unrelated edits to the function.

## Result

Callgrind instructions, batch head 079c6d8 to this branch (millions, small arguments), and wall time
(minimum of 9 runs, seconds; CPython 3 in the last column):

| Program | Instructions before | after | Change | Wall before | after | Change | After, vs CPython |
|---|---:|---:|---:|---:|---:|---:|---:|
| collatz | 943.4 | 909.1 | -3.6% | 0.312 | 0.284 | -9% | 0.35x |
| nbody | 228.1 | 218.8 | -4.1% | 0.257 | 0.238 | -7% | 0.82x |
| spectralnorm | 127.0 | 121.9 | -4.0% | 0.356 | 0.329 | -8% | 0.83x |
| mandelbrot | 325.0 | 300.5 | -7.5% | 0.206 | 0.165 | -20% | 0.75x |
| fannkuch | 83.2 | 81.5 | -2.0% | 0.654 | 0.589 | -10% | 0.81x |
| binarytrees | 368.3 | 369.2 | +0.2% | 0.795 | 0.805 | +1% | 1.48x |
| matmul | 131.1 | 128.8 | -1.8% | 0.406 | 0.358 | -12% | 0.78x |
| sieve | 320.4 | 289.5 | -9.6% | 0.595 | 0.582 | -2% | 1.18x |

The last column is the wall ratio from one `bench.py` run (minimum of 3), so it carries the box's noise. The differential fuzz (`tools/fuzz/bytecode-diff.sh`, ADR 0756) ran 3,000 and then 20,000 programs on the final commit with no disagreement.

Collatz, nbody and mandelbrot moved by more than the box's noise. The other wall differences are inside it and
are backed by the instruction counts. Binarytrees and sieve stay above CPython.

## Decision

Keep the five changes marked "Yes" and drop the rest. The tier's structure stays: separate registers, a
`Vec<(&str, Value)>` environment, recursive `exec` for calls. Lane 236 builds on this frame layout.

What is left is not the interpreter loop:

- **Sieve** at its default size is memory-bound: a `List[bool]` is 32 bytes per element (96 MB for 3,000,000),
  CPython's is 8. Its instruction count fell 10% and its wall time did not move. The fix is typed list storage
  (a flat vector of `bool` or `int` behind `List[T]`), a design of its own that touches every list operation.
- **Binarytrees** costs about 1,400 instructions per call-iteration: 11 bytecode instructions at about 60 each and
  about 700 for the call. The driver experiment shows the call cost is spread over many small steps with no
  single large one, so it needs fewer steps (arguments built directly in the callee's frame, no pooled
  environment), not a different loop.
- Unchecked access (about 12% fewer instructions, [ADR 0621](0621-bytecode-bounds-checks-stay.md)) is still the
  designer's call.

## Rejected alternatives

- **The unified frame and the call driver**: built, measured and dropped (table above). Both keep the stack
  traces and the walker hand-off working, which is why they were worth a real trial; neither is faster.
- **Reordering `Value::clone` for every caller**: the walker shares it, and the effect depended on the caller's
  code layout; the fix is local to the tier's moves.
- **Patching a stall per benchmark**: the changes above are per kind of move (a scalar into a slot), not per program.
