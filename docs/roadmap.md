# Roadmap

> **Status:** the stage list below comes from the founding brief, as amended
> by ADRs 0010 and 0017–0024.
> Proposed refinements are listed separately and are not yet decided.
> Stages 4, 6 and 7 were adjusted by ADRs [0010](decisions/0010-no-llvm-backend.md), [0017](decisions/0017-no-c-emission.md) and [0018](decisions/0018-optimizing-backend-from-start.md).

## Stages (from the founding brief)

| Stage | Goal | Exit criteria (proposed) | State |
|---|---|---|---|
| 1 | Language specification and design | All 28 design topics have accepted ADRs. The spec chapters are drafted. Token and performance benchmark methodologies are defined. | **In progress** |
| 2 | Lexer, parser, AST, type checker, **first semantic API** (bootstrap in Rust: [ADR 0023](decisions/0023-rust-bootstrap.md); API: [ADR 0024](decisions/0024-semantic-api-from-stage-2.md)) | All spec examples parse and type-check. Diagnostics are structured. Conformance harness running. Semantic API v1 published. | Not started |
| 3 | Interpreter: the **semantic reference implementation** and differential-testing oracle ([ADR 0018](decisions/0018-optimizing-backend-from-start.md)) | Semantics validated by running the conformance suite. | Not started |
| 4 | **Optimising native backend, written in AIL** ([ADR 0022](decisions/0022-staged-self-hosting.md)), runs on the interpreter until it compiles itself. Then the front end is ported to AIL. on AIL's own toolchain ([ADR 0010](decisions/0010-no-llvm-backend.md): no LLVM; [ADR 0017](decisions/0017-no-c-emission.md): no C; [ADR 0018](decisions/0018-optimizing-backend-from-start.md): optimising from the start; [ADR 0015](decisions/0015-always-whole-program-incremental.md)/[0016](decisions/0016-optimized-and-observable-debugging.md): single-mode WPO with full observability) | All 5 Tier 1 targets produce working binaries with AIL's own encoder and linker. Differential tests match the interpreter. First benchmark results against clang, gcc and rustc are published. | Not started |
| 5 | Custom runtime and GC | Runtime and GC pass stress tests on all targets, gated on ARM64 for weak-memory correctness. | Not started |
| 6 | Optimizer maturation | Closes the gap to (then beats) clang/gcc -O2/-O3 on the benchmark suite, using thresholds the designer sets. | Not started |
| 7 | *(merged into Stage 4 by ADR 0018)* | — | — |
| 8 | Complete the AI semantic interface (started in Stage 2) | All brief §12 queries are answerable. | Not started |
| 9 | Advanced optimisation, profiling, debugging, cross-platform tooling | Profile-guided optimisation, debugger, and profilers on all targets. | Not started |

## Proposed refinements (awaiting designer decision)

These are recommendations only. Each will be put to the designer as a question.

1. ~~Start the AI semantic interface in Stage 2.~~ **Accepted** as
   [ADR 0024](decisions/0024-semantic-api-from-stage-2.md).
2. **Run token and performance benchmarks from Stage 2 and Stage 4
   respectively.** "Performance must be measurable" (philosophy #4) should
   apply from the first binary, not only at Stage 9.
3. ~~Keep the bootstrap backend as a permanent performance reference.~~
   **Rejected** by [ADR 0010](decisions/0010-no-llvm-backend.md). Clang, gcc
   and rustc serve only as external comparison baselines.
4. **Write the formatter in Stage 2.** Canonical formatting shapes the grammar,
   and every later example in the docs should already be canonical.

## Documentation roadmap

- `ail doc`: a documentation tool written in AIL that replaces plain Markdown
  browsing (ADR 0003).
- A conformance harness that extracts `ail` examples from `docs/spec/` (ADR 0002).
