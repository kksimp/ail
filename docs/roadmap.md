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
| 3 | Interpreter: the **semantic reference implementation** and differential-testing oracle ([ADR 0018](decisions/0018-optimizing-backend-from-start.md)); the front end ported to AIL (`compiler/`, run by the bootstrap and differentially tested against it) | Semantics validated by running the conformance suite. The AIL lexer, parser and checker agree with the Rust ones on the whole corpus. | **Started 2026-10-20** (designer's decision). Done: the syntax tree (ADR 0376), lexer and parser in AIL (ADRs 0371–0375; byte-identical to `ail api --ast` on all repo files); the checker's declaration passes (ADR 0377). In progress: the rest of the checker (its gate is ADR 0380; ADRs 0378 and 0379 were reserved and never written); a bytecode tier in the bootstrap (ADR 0359, slices 1–2 merged). |
| 4 | **Optimising native backend, written in AIL** ([ADR 0022](decisions/0022-staged-self-hosting.md)), run on the interpreter until it compiles itself; then the front end is ported to AIL. Built on AIL's own toolchain ([ADR 0010](decisions/0010-no-llvm-backend.md): no LLVM; [ADR 0017](decisions/0017-no-c-emission.md): no C; [ADR 0018](decisions/0018-optimizing-backend-from-start.md): optimising from the start; [ADR 0015](decisions/0015-always-whole-program-incremental.md)/[0016](decisions/0016-optimized-and-observable-debugging.md): single-mode WPO with full observability) | All 5 Tier 1 targets produce working binaries with AIL's own encoder and linker. Differential tests match the interpreter. Every host can build for every target ([ADR 0031](decisions/0031-hosts-and-cross-compilation.md)). The drag-race harness runs, and first results against clang and gcc are published ([ADR 0030](decisions/0030-performance-benchmarks.md)). | Not started |
| 5 | Custom runtime and GC | Runtime and GC pass stress tests on all targets, gated on ARM64 for weak-memory correctness. | Not started |
| 6 | Optimizer maturation | Closes the gap to (then beats) clang/gcc -O2/-O3 on the benchmark suite, using thresholds the designer sets. | Not started |
| 7 | *(merged into Stage 4 by ADR 0018)* | — | — |
| 8 | Complete the AI semantic interface (started in Stage 2) | All brief §12 queries are answerable. | Not started |
| 9 | Advanced optimisation, profiling, debugging, cross-platform tooling | Profile-guided optimisation, debugger, and profilers on all targets. | Not started |

## Proposed refinements (history: decided long ago)

History, dated before the Rust removal; each was resolved in an ADR. These were recommendations only. Each will be put to the designer as a question.

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

## Rust bootstrap deleted ([ADR 0904](decisions/0904-rust-bootstrap-deleted.md))

`bootstrap/`, the interpreter, `cargo` and the private Rust toolchain
([ADR 0087](decisions/0087-private-rust-toolchain.md)) are gone; `./ail` builds from
the published seed ([ADR 0855](decisions/0855-retire-the-rust-bootstrap.md)). Notes
below that name the bootstrap, the interpreter, `tools/verify.sh` selftests or the
bytecode tier are history. The remaining plan is "After Rust removal" and the
backlog queue below.

## Next milestone (history: Stage 2, milestone 8; drafted 2026-10-20)

History: written before the Rust removal and kept for the record. For what comes next, see "Toward 0.2" below.

Drawn from what is still open after the overnight buildout; the order is a
proposal, not a decision.

Day 1 of batch 2 reached `main` on 2026-10-21 as PR #223 (106 lane pull requests; Linux 360/360 and macOS 359/359 selftests). What landed is listed in [status.md](status.md).

1. **Verify on macOS ARM64:** run `tools/verify.sh` on the whole 2026-10-20
   batch (the strict-lockstep rule, ADR 0008).
2. **Stage 3 started 2026-10-20 (designer's decision):** the lexer, parser and checker in AIL under `compiler/`, run by the bootstrap and differentially tested against it (lanes 113–114). **Stage 3 and 4 groundwork:** build the tiers designed in
   [ir-architecture.md](design/ir-architecture.md) (AST, HIR, MIR, machine IR)
   and the first native code for the hot loops the interpreter shows
   (`tools/perfbench/run.sh`); meanwhile a bytecode tier in the bootstrap
   (ADR 0359; slices 1–4 are ADRs 0361–0364).
   Native runtime and ABI (lane 122, ADRs 0404–0409): [native-abi.md](design/native-abi.md); `compiler/rt/rt.ail` has number text, strings, lists and traps on a machine model; syscalls and entry wait for the encoders.
3. **Dependencies:** git dependencies and `ail.lock` (no registry, ADR 0383);
   in-house TLS for `std.net` (ADR 0360, done: ADRs 0366–0369); hardware
   AES-GCM (AES-NI and PCLMULQDQ on x86-64, the crypto extension on ARM64,
   detected through AT_HWCAP on Linux) and proxy support
   ([ADR 0869](decisions/0869-http-proxy-from-the-environment.md)) are done.
4. **Rust-free check (designer, 2026-10-20):** now that the shipped toolchain
   no longer needs Rust (ADR 0904), rebuild ail-edit (its own repository) with the
   self-hosted toolchain and compare it with the Rust-bootstrap baseline in
   editor-baseline.md (in kksimp/ail-edit, `docs/`) (start-up, `ail check`
   time, test time, tokens, memory, and keystroke/redraw latency).
5. **Hardware:** accelerator backends for `std.tensor` (allowed by ADR 0381; CPU slice gaps first, then Vulkan and Metal, then NPUs) and weaker
   atomic orderings (OQ-097).
6. **Post-1.0 (after milestone 8 and the native toolchain):** a Python
   on-ramp, `ail port app.py` from typed Python to AIL, or a Python-syntax
   front end (OQ-119, designer's question of 2026-10-20).
7. **Open language questions:** safe member access and matching on error sets
   (OQ-029), custom hashing (OQ-042), unique values in generic code (OQ-050),
   loop invariants and static proof (OQ-060), downcasting `any Trait`
   (OQ-094).

## After Rust removal: AI-first tooling (designer, 2026-10-23)

Order per [ADR 0868](decisions/0868-ai-first-tooling-after-rust-removal.md); each is native and costs nothing when unused:
`ail ask` semantic queries; `uhwhatnow?`, a hole that ranks the candidates that fit; edit impact
(`ail test --affected`); native time-travel record and replay; token-priced diagnostics
(one line or expanded, chosen by `--budget`); a native sandbox (capabilities plus CPU,
memory and time caps); live attach to running binaries; contracts checked or fuzzed by tools
(lower priority).

## Backlog queue after Rust removal (designer, 2026-10-23)

Starts when the interpreter is gone. Lanes are grouped by the code they touch, so
two lanes never edit the same area: the lanes run in parallel, and the items in
one lane run in the order shown. A feature that changes the language or a public
std API still needs its own ADR first.

1. **Checker and diagnostics** (`compiler/check/`, `compiler/explain/`):
   `uhwhatnow?` (ranked typed holes); diagnostics in one-line and expanded forms chosen by `--budget`;
   then the open language questions OQ-029, OQ-042, OQ-050, OQ-094.
2. **Compiler graph tools** (`compiler/api/`, `compiler/edit/`): `ail ask`; edit
   impact and `ail test --affected`; contracts checked or fuzzed by tools, with
   OQ-060 (lower priority).
   Then the AI-first tools the designer chose on 2026-10-24, in this order, each
   with its own ADR (0891-0897) and no cost when unused (ADR 0868):
   `ail context SYM --budget N`; `@tool` functions served by `ail serve --mcp`;
   `ail try` (a patch checked and tested in memory); effect changes in `ail diff`;
   `ail infer` (suggested `requires`/`ensures`); `ail test --mutate`; `ail check --perf`
   (shares facts with the optimizer).
3. **Native runtime** (`compiler/rt/`): time-travel record and replay; the
   agent sandbox (capabilities plus CPU, memory and time caps); live attach.
4. **std.app and its host** (`std/app`, `tools/app-host/`): the screen model
   ([ADR 0878](decisions/0878-screen-model-for-std-app.md)), Garlic first, then every
   GUI program; `@observe` and live attach ([ADR 0879](decisions/0879-observe-and-live-attach.md)).
5. **Garlic interface** (kksimp/garlic window and chrome): closing tabs, and
   a redesign that looks as polished as Chrome or Firefox but has its own look.
6. **Garlic agent features** (kksimp/garlic page model and agent server):
   the agent server and attach channel; watches and scheduled actions; trust
   rules; grounding; forms as schema; extraction; quiet pages; waiting; intent
   maps; page memory; opt-in action replay; form diff before submit; site
   memory. Parity fixes for real sites are interleaved.
7. **Packages and networking** (`std/net`, packages): done (hardware
   AES-GCM, proxy support, git dependencies and `ail.lock` in the native
   loader).
8. **Tensors and hardware** (`std/tensor`, backends): CPU gaps, then Vulkan
   and Metal, then NPUs; weaker atomic orderings (OQ-097).
9. **Optimizer** (`compiler/mir` passes): purity-driven speed passes of
   [ADR 0874](decisions/0874-purity-driven-speed-optimizations.md): memoization by a cost
   model, pure-call CSE and hoisting, merging only where it is faster.
   Then two showcase programs the designer chose (2026-10-24), each with its own ADR:
   a database engine with its shell (file format, index, query language; an
   optimizer benchmark against C using contracts, `ail test --mutate` and
   `ail check --perf`; ADR 0898), and a media player or image editor on `std.app`
   (SIMD, multi-core and tensor speed; ADR 0899).
10. **Last:** the Rust-free rebuild of ail-edit compared with
   editor-baseline.md (in kksimp/ail-edit, `docs/`); a drag race of Garlic
   against Chrome and Firefox on a fixed set of real sites (page load, first
   paint, scrolling, memory and tokens per agent task); then the Python
   on-ramp (OQ-119).

## Toward 0.2 (2026-11-09)

v0.1.0 is the first public tag: the compiler builds itself from a published seed, runs on macOS ARM64, Linux x86-64 and Windows x86-64, and carries a launch set of docs. Until the tag, only launch changes and pull requests already green on the gate land; everything else targets 0.2.

Planned for 0.2 (none of it is a promise; status.md says what exists):

- Gate the ungated tools (`ail context`, `ail infer`, probes, `ail lsp` rename and code actions, `ail test --fuzz` and `--mutate`) and add a Linux ARM64 and a Windows ARM64 machine to the gate.
- The open feature pull requests: const generics (ADR 0954), generators (ADR 0965), `@tool` functions (ADR 0892), `ail merge`, `ail review`, `perf --explain`, variadic C and `export c`, epoll and signals, TLS server (needs constant-time signing first).
- Speed: the optimizer leftovers and the SIMD slices.
- Apps in their own repositories: Garlic (cross-platform look, video, canvas), Clove, ail-edit, Garlic Press.

## Documentation roadmap

- `ail doc`: a documentation tool written in AIL that replaces plain Markdown
  browsing (ADR 0003).
- ~~A conformance harness that extracts `ail` examples from `docs/spec/` (ADR 0002).~~ **Done** (2026-10-20, ADR 0172); it also checks the quick reference and the language card.
