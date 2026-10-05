# Project status

_Last updated: 2026-10-05_

**Current stage:** 2 — Bootstrap front end and interpreter. The Stage 1 core
design is complete ([ADR 0086](decisions/0086-finish-core-design-before-stage-2.md)).
**Implementation:** milestone 1 complete (2026-10-04): `./ail run examples/hello.ail`
prints `Hello, world!`. Milestone 2 (2026-10-05): `examples/gradebook.ail`
runs, with structs, enums, methods, `match`, loops, lists, `mut` and
interpolation. Verified on Linux x86-64; awaiting macOS ARM64 (ADR 0085).
See [bootstrap/README.md](../bootstrap/README.md) for the supported subset.

## Design topics

The 28 topics that must be resolved before implementation (founding brief §22).

Legend: **—** not started · **Q** questions open · **D** decided (ADR) ·
**S** specified (normative rules) · **I** implemented · **T** tested

| # | Topic | State | ADRs | Spec |
|---|---|---|---|---|
| 1 | Language name | D | — (founding brief: **AIL**, by Voltare, `.ail`, `ail` CLI) | — |
| 2 | Core syntax | D (partial) + Q; I+T (bootstrap subset) | [0011](decisions/0011-ai-familiarity-not-a-constraint.md), [0033](decisions/0033-indentation-blocks.md), [0034](decisions/0034-newline-terminated-statements.md), [0037](decisions/0037-signature-syntax.md), [0039](decisions/0039-declaration-syntax.md), [0040](decisions/0040-four-space-indentation.md), [0091](decisions/0091-control-flow.md), [0092](decisions/0092-operators.md), [0110](decisions/0110-bool.md), [0111](decisions/0111-if-expression-layout.md), [0112](decisions/0112-operator-precedence.md), [0114](decisions/0114-reserved-keywords.md), [0115](decisions/0115-integer-division.md), [0117](decisions/0117-typing-rules.md), [0118](decisions/0118-trailing-commas.md) | `lex.*`, `expr.*` |
| 3 | Grammar | D (partial) + Q | [0033](decisions/0033-indentation-blocks.md), [0034](decisions/0034-newline-terminated-statements.md), [0041](decisions/0041-square-bracket-generics.md), [0042](decisions/0042-fn-keyword.md), [0043](decisions/0043-hash-comments.md), [0054](decisions/0054-match-arm-syntax.md) | `lex.*`, `gram.*`, `expr.match.1` |
| 4 | Type system | D (partial) + Q; I+T (bootstrap subset) | [0007](decisions/0007-64-bit-little-endian-only.md) (64-bit sizes), [0035](decisions/0035-integer-overflow-traps.md) (overflow), [0052](decisions/0052-struct-declarations.md), [0053](decisions/0053-enum-declarations.md), [0055](decisions/0055-numeric-type-names.md), [0056](decisions/0056-exhaustive-matching.md), [0057](decisions/0057-variant-references.md), [0058](decisions/0058-struct-literal-syntax.md), [0059](decisions/0059-default-numeric-spelling.md), [0113](decisions/0113-struct-literal-field-order.md), [0116](decisions/0116-core-library-basics.md), [0117](decisions/0117-typing-rules.md) | `plat.ptr.*`, `expr.arith.*`, `expr.match.*`, `type.*` |
| 5 | Memory model | D (partial) + Q | [0007](decisions/0007-64-bit-little-endian-only.md), [0012](decisions/0012-memory-rd-tracks.md), [0013](decisions/0013-compiler-chosen-layout.md), [0014](decisions/0014-compressed-managed-references.md) | `plat.endian.1`, `plat.ptr.3` |
| 6 | GC design | D (direction) + Q | [0012](decisions/0012-memory-rd-tracks.md), [0014](decisions/0014-compressed-managed-references.md) | |
| 7 | Ownership/lifetime semantics | D (core model) + Q; I+T (values, `mut`, exclusivity) | [0038](decisions/0038-immutable-bindings-by-default.md), [0044](decisions/0044-no-shadowing.md), [0060](decisions/0060-mutable-value-semantics.md), [0061](decisions/0061-call-site-mut.md), [0062](decisions/0062-second-class-borrows.md), [0063](decisions/0063-use-site-ref.md), [0064](decisions/0064-exclusivity.md), [0065](decisions/0065-copy-reporting.md), [0066](decisions/0066-unique-resource-types.md), [0067](decisions/0067-ref-access.md), [0068](decisions/0068-move-parameters.md), [0069](decisions/0069-unique-keyword.md) | `expr.bind.*`, `own.*` |
| 8 | Error handling | D (core) + Q | [0045](decisions/0045-error-model.md), [0046](decisions/0046-inferred-error-sets.md), [0047](decisions/0047-traps-end-the-task.md), [0048](decisions/0048-optional-shorthand.md), [0049](decisions/0049-else-for-fallback-and-early-exit.md), [0050](decisions/0050-error-return-traces.md), [0051](decisions/0051-no-implicit-optional-to-error.md) | `err.*`, `type.opt.*` |
| 9 | Generics | D (core) + Q | [0041](decisions/0041-square-bracket-generics.md), [0074](decisions/0074-inline-generic-bounds.md), [0078](decisions/0078-trait-typed-parameters.md), [0080](decisions/0080-generic-compilation-strategy.md) | `gram.generic.1`, `gen.*` |
| 10 | Traits/interfaces | D (core) + Q | [0072](decisions/0072-dot-named-methods.md), [0073](decisions/0073-explicit-trait-impls.md), [0075](decisions/0075-unmarked-receiver-mutation.md), [0077](decisions/0077-operator-traits.md), [0079](decisions/0079-any-dynamic-dispatch.md), [0081](decisions/0081-default-methods.md), [0082](decisions/0082-method-placement-coherence.md) | `trait.*` |
| 11 | Concurrency | D (core) + Q | [0006](decisions/0006-arm64-baseline-v8-2.md) (LSE atomics), [0025](decisions/0025-memory-model-atomics.md), [0095](decisions/0095-colorless-tasks.md), [0096](decisions/0096-scope-bound-tasks.md), [0100](decisions/0100-task-communication.md) | `conc.*` |
| 12 | Modules | D (core) | [0088](decisions/0088-entry-point-and-print.md), [0097](decisions/0097-directory-modules.md), [0098](decisions/0098-pub-visibility.md), [0099](decisions/0099-qualified-imports.md), [0101](decisions/0101-single-file-programs.md) | `mod.*` |
| 13 | Package management | — | | |
| 14 | FFI | D (core) + Q | [0103](decisions/0103-c-header-import.md), [0104](decisions/0104-c-calls-unsafe.md), [0105](decisions/0105-libc-only-with-c.md) | `ffi.*` |
| 15 | Unsafe semantics | D (core) + Q | [0102](decisions/0102-unsafe-blocks.md), [0106](decisions/0106-unsafe-operations.md) | `unsafe.*` |
| 16 | Reflection | — | | |
| 17 | Metaprogramming | — | | |
| 18 | Compile-time execution | — | | |
| 19 | AST/IR architecture | — | | |
| 20 | AI semantic API | D (timing) + Q | [0024](decisions/0024-semantic-api-from-stage-2.md) | |
| 21 | Standard library architecture | D (collections, strings, print) + Q | [0088](decisions/0088-entry-point-and-print.md), [0089](decisions/0089-strings.md), [0090](decisions/0090-collections.md), [0116](decisions/0116-core-library-basics.md) | `type.str.*`, `type.coll.*` |
| 22 | Runtime architecture | D (partial) + Q | [0009](decisions/0009-custom-tooling-policy.md), [0020](decisions/0020-linux-no-libc.md), [0026](decisions/0026-hardware-feature-acceleration.md) | |
| 23 | Compiler architecture | D (partial) + Q | [0009](decisions/0009-custom-tooling-policy.md), [0010](decisions/0010-no-llvm-backend.md), [0015](decisions/0015-always-whole-program-incremental.md), [0017](decisions/0017-no-c-emission.md), [0018](decisions/0018-optimizing-backend-from-start.md), [0022](decisions/0022-staged-self-hosting.md), [0023](decisions/0023-rust-bootstrap.md) | |
| 24 | Cross-platform ABI strategy | D (partial) + Q | [0004](decisions/0004-macos-arm64-only.md), [0005](decisions/0005-x86-64-baseline-v3.md), [0006](decisions/0006-arm64-baseline-v8-2.md), [0007](decisions/0007-64-bit-little-endian-only.md), [0008](decisions/0008-isa-priority-policy.md), [0027](decisions/0027-ci-gating-all-targets.md) | `plat.*` |
| 25 | Optimization strategy | D (partial) + Q | [0005](decisions/0005-x86-64-baseline-v3.md), [0006](decisions/0006-arm64-baseline-v8-2.md), [0012](decisions/0012-memory-rd-tracks.md), [0013](decisions/0013-compiler-chosen-layout.md) | `plat.isa.*` |
| 26 | Debugging/profiling architecture | D (partial) + Q | [0016](decisions/0016-optimized-and-observable-debugging.md), [0021](decisions/0021-debug-info-formats.md) | |
| 27 | Build system | D (partial) + Q | [0015](decisions/0015-always-whole-program-incremental.md) (incremental cache), [0031](decisions/0031-hosts-and-cross-compilation.md) (hosts and cross-compilation) | |
| 28 | Testing strategy | D (partial) + Q | [0002](decisions/0002-normative-spec-with-rule-ids.md) (spec examples as conformance tests), [0019](decisions/0019-miscompilation-defence.md) (miscompilation defence), [0027](decisions/0027-ci-gating-all-targets.md) (CI gating), [0029](decisions/0029-token-efficiency-benchmarks.md) and [0030](decisions/0030-performance-benchmarks.md), [0036](decisions/0036-drag-race-c-styles.md) (benchmarks), [0032](decisions/0032-keep-rust-interpreter-as-oracle.md) (oracle) | |

## Project process

| Item | State | ADR |
|---|---|---|
| Decision records | D | [0001](decisions/0001-record-decisions-as-adrs.md) |
| Spec format and rule IDs | D | [0002](decisions/0002-normative-spec-with-rule-ids.md) |
| Documentation system | D | [0003](decisions/0003-documentation-in-repo-markdown.md) |
| Custom tooling policy | D | [0009](decisions/0009-custom-tooling-policy.md) |
| No external backend | D | [0010](decisions/0010-no-llvm-backend.md) |
| Open-source intent | D | [0011](decisions/0011-ai-familiarity-not-a-constraint.md) |
| Git workflow: `main` only long-lived branch | D | [0028](decisions/0028-git-workflow.md) |
| Token measurement tool (`tools/tokbench/`) | D, implemented | [0070](decisions/0070-token-bench-tool.md) |
| Delegated decisions by priority order | D | [0076](decisions/0076-delegated-decisions.md) |
| One-command scripts for everything people run | D, implemented for tokbench | [0071](decisions/0071-one-command-scripts.md) |
| Interim testing gate: local Linux and macOS, no hosted CI | D (interim) | [0085](decisions/0085-interim-local-testing-gate.md) |
| Merge via pull requests; GitHub deletes branches on merge | D | [0109](decisions/0109-merge-via-pull-requests.md) |
| Stage 2 entry criteria; Hello World milestone | D | [0086](decisions/0086-finish-core-design-before-stage-2.md) |
| Private, pinned Rust toolchain for the bootstrap | D (takes effect in Stage 2) | [0087](decisions/0087-private-rust-toolchain.md) |

## Open questions

See [open-questions.md](open-questions.md) (OQ-001 to OQ-037).
