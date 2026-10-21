# 0756. Differential fuzzing of the bytecode tier

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 16 — Testing
- **Spec rules:** fuzz.prop.5, expr.arith.1, type.coll.4, fuzz.policy.1
- **Builds on:** [ADR 0361](0361-bytecode-tier-design.md), [ADR 0269](0269-fuzz-generator-design.md), [ADR 0620](0620-bytecode-element-method-calls.md)

## Context

`fuzz.prop.5` already compares the tier with the walker on the general program generator. That generator rarely makes the shapes the tier compiles to one instruction: nested counted and `while` loops with `break` and `continue`, overflow edges, `xs[i].m()` calls through `mut self`, copies of struct values, `return` inside loops.

## Decision

1. **Generator** (`bootstrap/tests/fuzz/bcprogs.rs`, reusing the harness `Rng`). Each seed makes a program with a struct `Acc` (`step(mut self, d)`, `bump(mut self, d) int`, `total`), a struct `Bag` holding a list, a function `work` with loops and early returns, and `main`. Statements are chosen by depth: arithmetic at the overflow edges (a quarter of the programs are "edgy"; the rest keep values small so most runs finish), counted loops, `while` loops whose counter is bumped before `continue`, `break`, `return` inside a loop, indexes into `xs` that are in range or (in a fifth of the edgy programs) out of range, copies `mut c := xs[i]`, floats with `to_int`. Every third program is `pure`: no print, string, map or `else` inside a loop. The others mix those hand-off constructs in mid-loop. Loops count up to a small constant, so every program ends.
2. **Test** (`bootstrap/tests/fuzz_bytecode.rs`). Each program runs with the tier and with `--no-bytecode`. `fuzz.bc.1`: stdout, exit code and stderr (trap text and trace) are the same bytes. `fuzz.bc.2`: no panic, hang or `bootstrap.*` diagnostic. `fuzz.bc.3`: a `pure` program should make few hand-offs however long it loops; read from `AIL_BYTECODE_STATS=1`. A `fuzz.bc.3` miss costs speed only, so it is counted and reported, not failed. A disagreement is minimised by line deletion and saved in `tools/fuzz/found/bytecode/`. `AIL_FUZZ_N` (default 60, the selftest smoke) and `AIL_FUZZ_SEED` choose the run; `tools/fuzz/bytecode-diff.sh N [SEED]` runs it.
3. **No new flag:** `--no-bytecode` and `AIL_BYTECODE_STATS` already exist (ADR 0361).

## Results

120,300 programs compared, 24% ending in a trap (`expr.arith.1`, `expr.arith.3`, `type.coll.4`, `type.num.9`); about 99% of tier instructions ran natively.

- **Disagreement 1 (open, lane 190):** `xs[3].step(ARG)` with an out-of-range index and an `ARG` that traps. The walker traps on the index (`type.coll.4`); the tier traps on `ARG` first (`expr.arith.1`). The walker is the oracle: a receiver is evaluated before its arguments. Repro: `tools/fuzz/found/bytecode/elem_call_arg_before_bounds.ail`. The generator keeps the two apart until it is fixed, so the other 120,000 programs could run.
- **Missed fast path (info):** `xs[i % xs.len()].step(j)` hands off on every call; `xs[i].step(j)` and `xs[i % 3].step(j)` do not. An index that reads the list being called through leaves the fast path. About 0.6% of the pure programs show it.

## Rejected

- A Python generator in `tools/fuzz/gen/`: ADR 0070 limits Python to measurement, and the harness and its `Rng` are Rust.
- Failing the test on `fuzz.bc.3`: a missed fast path is a speed finding, and a failing selftest would block unrelated lanes.

## Wider grammar (2026-10-21)

`bcprogs::wide` adds maps, strings, function values, enums with `match` and
nested structs (about half the programs). 40,000 programs agreed with the
walker once lane 190's element-order fix landed, and the generator no longer
keeps a trapping argument apart from a bad index. Hand-offs seen by fuzz.bc.3
(speed only): a call through a function value hands off twice, and a
`mut self` method on a nested field (`o.inner.grow(i)`) hands off per call.
