# Roadmap

> **Status:** the stage list below comes from the founding brief, as amended
> by ADRs 0010, 0017–0024 and 0029–0032.
> Stages 4, 6 and 7 were adjusted by ADRs [0010](decisions/0010-no-llvm-backend.md), [0017](decisions/0017-no-c-emission.md) and [0018](decisions/0018-optimizing-backend-from-start.md).
> Refinement #4 (formatter in Stage 2) is done: `ail fmt` ([ADR 0169](decisions/0169-canonical-formatting-style.md), [ADR 0170](decisions/0170-ail-fmt-command.md)).

## Stages (from the founding brief)

| Stage | Goal | Exit criteria (proposed) | State |
|---|---|---|---|
| 1 | Language specification and design | All 28 design topics have accepted ADRs. The spec chapters are drafted. The token benchmark corpus (layers 1–2) exists and is used for syntax decisions. | **Core done** (2026-10-04, ADR 0086). Remaining topics continue alongside Stage 2. |
| 2 | **Entry criteria ([ADR 0086](decisions/0086-finish-core-design-before-stage-2.md)):** concurrency model, modules and packages core, strings/collections/loops/literals, entry point and printing, and the unsafe/FFI core model are decided. **First milestone:** `./ail run examples/hello.ail` on macOS. Then: lexer, parser, AST, type checker, **first semantic API** (bootstrap in Rust: [ADR 0023](decisions/0023-rust-bootstrap.md); API: [ADR 0024](decisions/0024-semantic-api-from-stage-2.md)) | All spec examples parse and type-check. Diagnostics are structured. Conformance harness running. Semantic API v1 published. | **In progress** (started 2026-10-04). Milestone 1 (Hello World) done: Linux x86-64 and macOS ARM64 pass. Milestone 2 (gradebook: structs, enums, methods, `match`, loops, lists, `mut`, interpolation) done: Linux x86-64 and macOS ARM64 pass (2026-10-05). Milestone 3 (command parser: error types, `?`, `else`, optionals, variants with data, `for mut`) done: Linux x86-64 and macOS ARM64 pass (2026-10-05). If-binding conditions (ADR 0125) done and verified on both (2026-10-05). Milestone 4 (traits and generics, ADRs 0126–0129) done: Linux x86-64 and macOS ARM64 pass (2026-10-05). Milestone 5 (three 500–800 line programs: interpreter, JSON tool, spreadsheet; ADRs 0130–0140) done: Linux x86-64 and macOS ARM64 pass (2026-10-05). Milestone 6 (Python parity: implicit `self`, positional constructor arguments, empty literals typed by use; ADRs 0141–0143) done: Linux x86-64 and macOS ARM64 pass (2026-10-05). **Milestone 7 (AI-first buildout, 2026-10-20) done on Linux; macOS verification pending:** the semantic API (`ail api`, `explain`, `fix`, `effects`, schema v1, typed holes), effects and capabilities, standard modules (`fs`, `env`, `io`, `proc`, `time`, `random`, `json`, `ui`), user modules and packages, `ail test` with property and doc tests, `ail fmt`, contracts, tasks and channels, the spec conformance harness, an editor written in AIL, and (rounds 4–9) ownership (`unique`, `move`, `ref`, `defer`), `par for` and a thread pool, derive generators and compile-time reflection, `std.tensor`, tracing and coverage, and a ten-program agent corpus under Python in tokens. See [status.md](status.md). |
| 3 | Interpreter: the **semantic reference implementation** and differential-testing oracle ([ADR 0018](decisions/0018-optimizing-backend-from-start.md)) | Semantics validated by running the conformance suite. | Not started |
| 4 | **Optimising native backend, written in AIL** ([ADR 0022](decisions/0022-staged-self-hosting.md)), run on the interpreter until it compiles itself; then the front end is ported to AIL. Built on AIL's own toolchain ([ADR 0010](decisions/0010-no-llvm-backend.md): no LLVM; [ADR 0017](decisions/0017-no-c-emission.md): no C; [ADR 0018](decisions/0018-optimizing-backend-from-start.md): optimising from the start; [ADR 0015](decisions/0015-always-whole-program-incremental.md)/[0016](decisions/0016-optimized-and-observable-debugging.md): single-mode WPO with full observability) | All 5 Tier 1 targets produce working binaries with AIL's own encoder and linker. Differential tests match the interpreter. Every host can build for every target ([ADR 0031](decisions/0031-hosts-and-cross-compilation.md)). The drag-race harness runs, and first results against clang and gcc are published ([ADR 0030](decisions/0030-performance-benchmarks.md)). | Not started |
| 5 | Custom runtime and GC | Runtime and GC pass stress tests on all targets, gated on ARM64 for weak-memory correctness. | Not started |
| 6 | Optimizer maturation | Closes the gap to (then beats) clang/gcc -O2/-O3 on the benchmark suite, using thresholds the designer sets. | Not started |
| 7 | *(merged into Stage 4 by ADR 0018)* | — | — |
| 8 | Complete the AI semantic interface (started in Stage 2) | All brief §12 queries are answerable. | Not started |
| 9 | Advanced optimisation, profiling, debugging, cross-platform tooling | Profile-guided optimisation, debugger, and profilers on all targets. | Not started |

## Proposed refinements (awaiting designer decision)

These are recommendations only. Each will be put to the designer as a question.

1. ~~Start the AI semantic interface in Stage 2.~~ **Accepted** as
   [ADR 0024](decisions/0024-semantic-api-from-stage-2.md).
2. ~~Run token and performance benchmarks early.~~ **Accepted:** token
   benchmarks start in Stage 1 ([ADR 0029](decisions/0029-token-efficiency-benchmarks.md)).
   The performance drag-race harness ships with Stage 4
   ([ADR 0030](decisions/0030-performance-benchmarks.md)).
3. ~~Keep the bootstrap backend as a permanent performance reference.~~
   **Rejected** by [ADR 0010](decisions/0010-no-llvm-backend.md). Clang, gcc
   and rustc serve only as external comparison baselines.
4. **Write the formatter in Stage 2.** Canonical formatting shapes the grammar,
   and every later example in the docs should already be canonical.
   **Done** (2026-10-20): `ail fmt`, ADRs 0169–0170, style in
   [design/formatting.md](design/formatting.md).

## Next milestone (Stage 2, milestone 8; drafted 2026-10-20)

Drawn from what is still open after the overnight buildout; the order is a
proposal, not a decision.

1. **Verify on macOS ARM64:** run `tools/verify.sh` on the whole 2026-10-20
   batch (the strict-lockstep rule, ADR 0008).
2. **Stage 3 started 2026-10-20 (designer's decision):** the lexer, parser and checker in AIL under `compiler/`, run by the bootstrap and differentially tested against it (lanes 113–114). **Stage 3 and 4 groundwork:** build the tiers designed in
   [ir-architecture.md](design/ir-architecture.md) (AST, HIR, MIR, machine IR)
   and the first native code for the hot loops the interpreter shows
   (`tools/perfbench/run.sh`); meanwhile a bytecode tier in the bootstrap
   (ADR 0359, in progress).
3. **Registry and lockfile:** git dependencies, `ail.lock`, then a package
   registry (OQ-049, the designer's call); in-house TLS for `std.net`
   (ADR 0360, in progress).
4. **Hardware:** accelerator backends for `std.tensor` (allowed by ADR 0381; CPU slice gaps first, then Vulkan and Metal, then NPUs) and weaker
   atomic orderings (OQ-097).
5. **Open language questions:** safe member access and matching on error sets
   (OQ-029), custom hashing (OQ-042), unique values in generic code (OQ-050),
   loop invariants and static proof (OQ-060), downcasting `any Trait`
   (OQ-094).

## Documentation roadmap

- `ail doc`: a documentation tool written in AIL that replaces plain Markdown
  browsing (ADR 0003).
- ~~A conformance harness that extracts `ail` examples from `docs/spec/` (ADR 0002).~~ **Done** (2026-10-20, ADR 0172); it also checks the quick reference and the language card.
