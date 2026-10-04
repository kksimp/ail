# Project status

_Last updated: 2026-10-04_

**Current stage:** 1 — Language specification and design.
**Implementation:** none yet (by design; see [AGENTS.md](../AGENTS.md)).

## Design topics

The 28 topics that must be resolved before implementation (founding brief §22).

Legend: **—** not started · **Q** questions open · **D** decided (ADR) ·
**S** specified (normative rules) · **I** implemented · **T** tested

| # | Topic | State | ADRs | Spec |
|---|---|---|---|---|
| 1 | Language name | D | — (founding brief: **AIL**, by Voltare, `.ail`, `ail` CLI) | — |
| 2 | Core syntax | D (principle only) | [0011](decisions/0011-ai-familiarity-not-a-constraint.md) | |
| 3 | Grammar | — | | |
| 4 | Type system | Q | [0007](decisions/0007-64-bit-little-endian-only.md) (64-bit sizes) | `plat.ptr.*` |
| 5 | Memory model | D (partial) + Q | [0007](decisions/0007-64-bit-little-endian-only.md), [0012](decisions/0012-memory-rd-tracks.md), [0013](decisions/0013-compiler-chosen-layout.md), [0014](decisions/0014-compressed-managed-references.md) | `plat.endian.1`, `plat.ptr.3` |
| 6 | GC design | D (direction) + Q | [0012](decisions/0012-memory-rd-tracks.md), [0014](decisions/0014-compressed-managed-references.md) | |
| 7 | Ownership/lifetime semantics | — | | |
| 8 | Error handling | — | | |
| 9 | Generics | — | | |
| 10 | Traits/interfaces | — | | |
| 11 | Concurrency | D (partial) + Q | [0006](decisions/0006-arm64-baseline-v8-2.md) (LSE atomics), [0025](decisions/0025-memory-model-atomics.md) | `conc.mm.1`, `conc.race.1`, `conc.atomic.*` |
| 12 | Modules | — | | |
| 13 | Package management | — | | |
| 14 | FFI | — | | |
| 15 | Unsafe semantics | — | | |
| 16 | Reflection | — | | |
| 17 | Metaprogramming | — | | |
| 18 | Compile-time execution | — | | |
| 19 | AST/IR architecture | — | | |
| 20 | AI semantic API | D (timing) + Q | [0024](decisions/0024-semantic-api-from-stage-2.md) | |
| 21 | Standard library architecture | — | | |
| 22 | Runtime architecture | D (partial) + Q | [0009](decisions/0009-custom-tooling-policy.md), [0020](decisions/0020-linux-no-libc.md), [0026](decisions/0026-hardware-feature-acceleration.md) | |
| 23 | Compiler architecture | D (partial) + Q | [0009](decisions/0009-custom-tooling-policy.md), [0010](decisions/0010-no-llvm-backend.md), [0015](decisions/0015-always-whole-program-incremental.md), [0017](decisions/0017-no-c-emission.md), [0018](decisions/0018-optimizing-backend-from-start.md), [0022](decisions/0022-staged-self-hosting.md), [0023](decisions/0023-rust-bootstrap.md) | |
| 24 | Cross-platform ABI strategy | D (partial) + Q | [0004](decisions/0004-macos-arm64-only.md), [0005](decisions/0005-x86-64-baseline-v3.md), [0006](decisions/0006-arm64-baseline-v8-2.md), [0007](decisions/0007-64-bit-little-endian-only.md), [0008](decisions/0008-isa-priority-policy.md), [0027](decisions/0027-ci-gating-all-targets.md) | `plat.*` |
| 25 | Optimization strategy | D (partial) + Q | [0005](decisions/0005-x86-64-baseline-v3.md), [0006](decisions/0006-arm64-baseline-v8-2.md), [0012](decisions/0012-memory-rd-tracks.md), [0013](decisions/0013-compiler-chosen-layout.md) | `plat.isa.*` |
| 26 | Debugging/profiling architecture | D (partial) + Q | [0016](decisions/0016-optimized-and-observable-debugging.md), [0021](decisions/0021-debug-info-formats.md) | |
| 27 | Build system | D (partial) + Q | [0015](decisions/0015-always-whole-program-incremental.md) (incremental cache) | |
| 28 | Testing strategy | D (partial) + Q | [0002](decisions/0002-normative-spec-with-rule-ids.md) (spec examples as conformance tests), [0019](decisions/0019-miscompilation-defence.md) (miscompilation defence), [0027](decisions/0027-ci-gating-all-targets.md) (CI gating) | |

## Project process

| Item | State | ADR |
|---|---|---|
| Decision records | D | [0001](decisions/0001-record-decisions-as-adrs.md) |
| Spec format and rule IDs | D | [0002](decisions/0002-normative-spec-with-rule-ids.md) |
| Documentation system | D | [0003](decisions/0003-documentation-in-repo-markdown.md) |
| Custom tooling policy | D | [0009](decisions/0009-custom-tooling-policy.md) |
| No external backend | D | [0010](decisions/0010-no-llvm-backend.md) |
| Open-source intent | D | [0011](decisions/0011-ai-familiarity-not-a-constraint.md) |
| Git workflow: work on `main` | D (instruction) | — |

## Open questions

See [open-questions.md](open-questions.md) (OQ-001 to OQ-024).
