# 0656. Cold-build hot paths in the AIL compiler, found with a per-function profile

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** native.build.1 (output unchanged)
- **Builds on:** [ADR 0655](0655-build-cache.md), [native-compile-speed.md](../design/native-compile-speed.md), [ADR 0393](0393-native-modules.md)

## Context

After ADR 0655 a changed program still builds cold in about 1.4 s, nearly all
of it the AIL compiler running on the bytecode tier. No profile named the
AIL functions responsible, so lane 182 and the cache work could only guess.

## Decision

1. **`AIL_BYTECODE_STATS=3`** prints, with the usual summary, the 40 AIL
   functions that ran the most bytecode instructions of their own, with
   their call counts (`bytecode.rs`, `Stats.by_fn`). Off by default; the
   counters only run when the variable is set.
2. **`mir.dominators`** ran 43% of all instructions in a hello build (30.3 M of
   71.2 M; 238 calls of about 127,000 each): a data-flow fixed point over an
   `n x n` bool matrix. It now computes immediate dominators (Cooper, Harvey
   and Kennedy) and builds each row as its immediate dominator's row plus
   the block. Same matrix, including all-true rows for blocks the entry
   can't reach; 580 K instructions now.
3. **The runtime was parsed with `prim`, `core` and `math` twice** (a list of
   three first, then every file of the directory, which includes them). Each
   file is now parsed once, in the same order, so the output is the same.
4. **The lexer** matched each symbol against a list of 38 by building each
   one's characters. `symbol_at` checks `..=`, then the two-character set,
   then the one-character set (6.2 M instructions to 0.2 M).
5. **The parser's lookahead** (`peek_at`, `sym_at`, `kw_at`) no longer calls
   `min` and `tok_tag`/`tok_text` per peek.

## Measurements

Linux x86-64, `ail run compiler build hello.ail` (the AIL side of a build:
without the Rust check; minimum of 9 runs), bytecode instructions from
`AIL_BYTECODE_STATS=3`:

| Step | Instructions | Wall (min of 9) |
|---|---|---|
| Before (ADR 0655 head) | 71.2 M | 1.446 s |
| `dominators` | 41.4 M | |
| runtime parsed once, `symbol_at` | 35.6 M | |
| lookahead | 23.9 M | **0.845 s** |

Output is byte-identical to the head's compiler for every program in
`examples/` and `tools/perfbench/programs/` (76) on `x86_64-linux` and
`aarch64-macos`. With ADR 0655's front end, a cold `ail build` of hello goes
from 1.82 s to about 1.0 s. What remains (hello, AIL side, 0.55 s): the runtime's
lexing (0.11 s) and parsing (0.18 s), lowering (0.14 s) and code generation
(0.12 s).

## Rejected alternatives

- **Parsing the runtime's files in parallel** (`par` comprehension over the
  token slices, joined with re-indexed `impl` and `trait` methods). Output was
  identical, but the interpreter's `par` ran no faster: four 3 M-iteration
  loops took 300 ms serially and 263 ms with `par` in a probe, because pool
  threads run the walker, not the bytecode tier. Dropped; revisit if the pool
  gets the tier.
- **Per-token symbol and keyword lists built once per parse** (so lookahead
  copies no `Tok`). It cut instructions by 7% and slowed the build, because
  building two lists copies every token.
- **Skipping unreachable runtime functions before parsing.** Needs the list of
  runtime names the lowering calls on its own account (`lower*.ail`, outside
  this lane); a missed name would break one program in a hundred.
- **Rewriting `codegen.liveness` and `allocate`** (2.7 M and 2.3 M
  instructions, about 10% each): left for a lane that can test the
  register allocator against the optimiser's output.
