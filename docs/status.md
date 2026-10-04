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
| 2 | Core syntax | — | | |
| 3 | Grammar | — | | |
| 4 | Type system | Q | [0007](decisions/0007-64-bit-little-endian-only.md) (64-bit sizes) | `plat.ptr.*` |
| 5 | Memory model | Q | [0007](decisions/0007-64-bit-little-endian-only.md) | `plat.endian.1` |
| 6 | GC design | Q | | |
| 7 | Ownership/lifetime semantics | — | | |
| 8 | Error handling | — | | |
| 9 | Generics | — | | |
| 10 | Traits/interfaces | — | | |
| 11 | Concurrency | Q | [0006](decisions/0006-arm64-baseline-v8-2.md) (LSE atomics) | |
| 12 | Modules | — | | |
| 13 | Package management | — | | |
| 14 | FFI | — | | |
| 15 | Unsafe semantics | — | | |
| 16 | Reflection | — | | |
| 17 | Metaprogramming | — | | |
| 18 | Compile-time execution | — | | |
| 19 | AST/IR architecture | — | | |
| 20 | AI semantic API | — | | |
| 21 | Standard library architecture | — | | |
| 22 | Runtime architecture | Q | | |
| 23 | Compiler architecture | Q | | |
| 24 | Cross-platform ABI strategy | D (partial) + Q | [0004](decisions/0004-macos-arm64-only.md), [0005](decisions/0005-x86-64-baseline-v3.md), [0006](decisions/0006-arm64-baseline-v8-2.md), [0007](decisions/0007-64-bit-little-endian-only.md), [0008](decisions/0008-isa-priority-policy.md) (proposed) | `plat.*` |
| 25 | Optimization strategy | D (partial) | [0005](decisions/0005-x86-64-baseline-v3.md), [0006](decisions/0006-arm64-baseline-v8-2.md) | `plat.isa.*` |
| 26 | Debugging/profiling architecture | — | | |
| 27 | Build system | Q | | |
| 28 | Testing strategy | Q | [0002](decisions/0002-normative-spec-with-rule-ids.md) (spec examples as conformance tests) | |

## Project process

| Item | State | ADR |
|---|---|---|
| Decision records | D | [0001](decisions/0001-record-decisions-as-adrs.md) |
| Spec format and rule IDs | D | [0002](decisions/0002-normative-spec-with-rule-ids.md) |
| Documentation system | D | [0003](decisions/0003-documentation-in-repo-markdown.md) |

## Open questions

See [open-questions.md](open-questions.md) (OQ-001 to OQ-011).
