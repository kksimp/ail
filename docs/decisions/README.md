# Architecture Decision Records

Each design decision has its own file (see [ADR 0001](0001-record-decisions-as-adrs.md)).
New ADRs copy [0000-template.md](0000-template.md) and take the next free number.
ADRs are never deleted. Replaced decisions are marked `Superseded by NNNN`.

| # | Title | Status | Date |
|---|---|---|---|
| [0001](0001-record-decisions-as-adrs.md) | Record design decisions as numbered ADRs | Accepted | 2026-10-04 |
| [0002](0002-normative-spec-with-rule-ids.md) | Normative specification with stable rule IDs and tested examples | Accepted | 2026-10-04 |
| [0003](0003-documentation-in-repo-markdown.md) | Documentation as Markdown in the repo; AIL docs tool later | Accepted | 2026-10-04 |
| [0004](0004-macos-arm64-only.md) | Support macOS on ARM64 only | Accepted | 2026-10-04 |
| [0005](0005-x86-64-baseline-v3.md) | x86-64 baseline is x86-64-v3, with runtime dispatch | Accepted | 2026-10-04 |
| [0006](0006-arm64-baseline-v8-2.md) | ARM64 baseline is ARMv8.2-A | Accepted | 2026-10-04 |
| [0007](0007-64-bit-little-endian-only.md) | 64-bit little-endian targets only | Accepted | 2026-10-04 |
| [0008](0008-isa-priority-policy.md) | ISA priority policy: ARM64 and x86-64 in strict lockstep | Accepted | 2026-10-04 |
| [0009](0009-custom-tooling-policy.md) | Build custom tooling unless an existing tool matches every requirement | Accepted | 2026-10-04 |
| [0010](0010-no-llvm-backend.md) | No LLVM or other external backend; existing compilers for comparison only | Accepted | 2026-10-04 |
| [0011](0011-ai-familiarity-not-a-constraint.md) | AI training-data familiarity is not a design constraint; open-source intent | Accepted | 2026-10-04 |
| [0012](0012-memory-rd-tracks.md) | Pursue four memory R&D tracks (allocation ladder, task-local heaps, in-place reuse, inferred arenas) | Accepted | 2026-10-04 |
| [0013](0013-compiler-chosen-layout.md) | Data layout compiler-chosen by default, fixed layout on request | Accepted | 2026-10-04 |
| [0014](0014-compressed-managed-references.md) | Managed references may be compressed; raw pointers stay 64-bit | Accepted | 2026-10-04 |
| [0015](0015-always-whole-program-incremental.md) | Always whole-program optimisation, with fine-grained incremental caching | Accepted | 2026-10-04 |
| [0016](0016-optimized-and-observable-debugging.md) | Debug builds are fully optimised and fully observable | Accepted | 2026-10-04 |
| [0017](0017-no-c-emission.md) | Never emit C as a path to native code | Accepted | 2026-10-04 |
| [0018](0018-optimizing-backend-from-start.md) | Build an optimising native backend from the start | Accepted | 2026-10-04 |
| [0019](0019-miscompilation-defence.md) | Four-layer defence against miscompilation | Accepted | 2026-10-04 |
| [0020](0020-linux-no-libc.md) | Linux runtime uses raw syscalls, no libc | Accepted | 2026-10-04 |
| [0021](0021-debug-info-formats.md) | Emit AIL's own debug format plus DWARF and PDB | Accepted | 2026-10-04 |
| [0022](0022-staged-self-hosting.md) | Staged self-hosting: backend written in AIL from the start | Accepted | 2026-10-04 |
| [0023](0023-rust-bootstrap.md) | Bootstrap front end and interpreter written in Rust | Accepted | 2026-10-04 |
| [0024](0024-semantic-api-from-stage-2.md) | AI semantic API starts in Stage 2 and grows each stage | Accepted | 2026-10-04 |
| [0025](0025-memory-model-atomics.md) | Abstract weak memory model; data-race-free safe code; SeqCst-default atomics | Accepted | 2026-10-04 |
| [0026](0026-hardware-feature-acceleration.md) | Portable runtime core; ISA-specific features only as optional accelerators | Accepted | 2026-10-04 |
| [0027](0027-ci-gating-all-targets.md) | All five Tier 1 targets gate every merge | Accepted; amended (interim) by 0085 | 2026-10-04 |
| [0028](0028-git-workflow.md) | Git workflow: `main` is the only long-lived branch | Accepted | 2026-10-04 |
| [0029](0029-token-efficiency-benchmarks.md) | Token-efficiency benchmarks: corpus, multiple tokenizers, AI task runs | Accepted | 2026-10-04 |
| [0030](0030-performance-benchmarks.md) | Performance benchmarks: standard suites, real apps, automated drag races | Accepted | 2026-10-04 |
| [0031](0031-hosts-and-cross-compilation.md) | Compiler runs on all five platforms and builds for all of them | Accepted | 2026-10-04 |
| [0032](0032-keep-rust-interpreter-as-oracle.md) | Keep the Rust interpreter permanently as an independent oracle | Accepted | 2026-10-04 |
| [0033](0033-indentation-blocks.md) | Blocks are delimited by indentation | Accepted | 2026-10-04 |
| [0034](0034-newline-terminated-statements.md) | Newlines end statements; no semicolons | Accepted | 2026-10-04 |
| [0035](0035-integer-overflow-traps.md) | Integer overflow traps; wrapping/saturating/checked ops are explicit | Accepted | 2026-10-04 |
| [0036](0036-drag-race-c-styles.md) | Drag races compare against both idiomatic and tuned C | Accepted | 2026-10-04 |
| [0037](0037-signature-syntax.md) | Signatures: `fn name(param Type) ReturnType`, no colons or arrows | Accepted | 2026-10-04 |
| [0038](0038-immutable-bindings-by-default.md) | Local bindings are immutable by default | Accepted | 2026-10-04 |
| [0039](0039-declaration-syntax.md) | Declarations: `x := v`, `mut x := v`; reassignment `x = v` | Accepted | 2026-10-04 |
| [0040](0040-four-space-indentation.md) | Indentation unit is four spaces; no tabs in indentation | Accepted | 2026-10-04 |
| [0041](0041-square-bracket-generics.md) | Generic arguments use `[ ]`; indexing also uses `[ ]` | Accepted | 2026-10-04 |
| [0042](0042-fn-keyword.md) | Functions are introduced by `fn` | Accepted | 2026-10-04 |
| [0043](0043-hash-comments.md) | Comments `#`, doc comments `##`, no block comments | Accepted | 2026-10-04 |
| [0044](0044-no-shadowing.md) | Names may not shadow other visible local names | Accepted | 2026-10-04 |
| [0045](0045-error-model.md) | Error model: values + `?`; bugs trap; no exceptions | Accepted | 2026-10-04 |
| [0046](0046-inferred-error-sets.md) | Error sets inferred inside a package, explicit at APIs; `T ! E`, `!T` | Accepted | 2026-10-04 |
| [0047](0047-traps-end-the-task.md) | A trap ends the current task; the parent observes it | Accepted | 2026-10-04 |
| [0048](0048-optional-shorthand.md) | No null; optional values are `T?` | Accepted | 2026-10-04 |
| [0049](0049-else-for-fallback-and-early-exit.md) | `else` handles failure: fallback values and early exit | Accepted | 2026-10-04 |
| [0050](0050-error-return-traces.md) | Automatic error return traces; optional context message | Accepted | 2026-10-04 |
| [0051](0051-no-implicit-optional-to-error.md) | No `?` on optionals in error-returning functions | Accepted | 2026-10-04 |
| [0052](0052-struct-declarations.md) | Structs: `struct Name` with indented `field Type` lines | Accepted | 2026-10-04 |
| [0053](0053-enum-declarations.md) | Enums: `enum Name` with one indented variant per line | Accepted | 2026-10-04 |
| [0054](0054-match-arm-syntax.md) | Match arms: `pattern => result` | Accepted | 2026-10-04 |
| [0055](0055-numeric-type-names.md) | Numeric types by bit width; `int`/`float` default aliases | Accepted | 2026-10-04 |
| [0056](0056-exhaustive-matching.md) | Matches must be exhaustive; `_` hiding variants is warned | Accepted | 2026-10-04 |
| [0057](0057-variant-references.md) | Variants bare where the type is known, qualified otherwise | Accepted | 2026-10-04 |
| [0058](0058-struct-literal-syntax.md) | Struct literals `Type(field: value)`; `name: value` for named args | Accepted | 2026-10-04 |
| [0059](0059-default-numeric-spelling.md) | `int`/`i64` and `float`/`f64` both canonical | Accepted | 2026-10-04 |
| [0060](0060-mutable-value-semantics.md) | AIL uses mutable value semantics | Accepted | 2026-10-04 |
| [0061](0061-call-site-mut.md) | `mut` parameters are marked `mut` at the call site | Accepted | 2026-10-04 |
| [0062](0062-second-class-borrows.md) | Second-class borrows; no lifetime annotations | Accepted | 2026-10-04 |
| [0063](0063-use-site-ref.md) | Shared mutable identity via use-site `ref T` | Accepted | 2026-10-04 |
| [0064](0064-exclusivity.md) | Exclusivity: overlap is a compile error; runtime check when undecidable | Accepted | 2026-10-04 |
| [0065](0065-copy-reporting.md) | Copies implicit in source, reported by the compiler | Accepted | 2026-10-04 |
| [0066](0066-unique-resource-types.md) | Unique (non-copyable) resource types, scope-end cleanup, `defer` | Accepted | 2026-10-04 |
| [0067](0067-ref-access.md) | `ref` cells: automatic access, mutable through any copy | Accepted | 2026-10-04 |
| [0068](0068-move-parameters.md) | Ownership-taking parameters and arguments are marked `move` | Accepted | 2026-10-04 |
| [0069](0069-unique-keyword.md) | Non-copyable types are declared `unique` | Accepted | 2026-10-04 |
| [0070](0070-token-bench-tool.md) | Token-measurement tooling in `tools/tokbench/` (Python) | Accepted | 2026-10-04 |
| [0071](0071-one-command-scripts.md) | Everything a person runs is a one-command, self-setting-up script per OS | Accepted | 2026-10-04 |
| [0072](0072-dot-named-methods.md) | Methods are dot-named functions: `fn Type.name(self, ...)` | Accepted | 2026-10-04 |
| [0073](0073-explicit-trait-impls.md) | Explicit trait implementation: `impl Trait for Type` | Accepted | 2026-10-04 |
| [0074](0074-inline-generic-bounds.md) | Inline generic constraints `[T Bound]`, joined with `+` | Accepted | 2026-10-04 |
| [0075](0075-unmarked-receiver-mutation.md) | Receiver-mutating calls are not marked at the call site | Accepted | 2026-10-04 |
| [0076](0076-delegated-decisions.md) | AI decides routine questions by priority order; serious ones escalate | Accepted | 2026-10-04 |
| [0077](0077-operator-traits.md) | Fixed operator set implementable via standard traits *(delegated)* | Accepted | 2026-10-04 |
| [0078](0078-trait-typed-parameters.md) | Trait as a parameter type = anonymous static generic *(delegated)* | Accepted | 2026-10-04 |
| [0079](0079-any-dynamic-dispatch.md) | Dynamic dispatch is explicit: `any Trait` *(delegated)* | Accepted | 2026-10-04 |
| [0080](0080-generic-compilation-strategy.md) | Compiler chooses specialisation vs sharing per generic instance *(delegated)* | Accepted | 2026-10-04 |
| [0081](0081-default-methods.md) | Traits may provide default methods *(delegated)* | Accepted | 2026-10-04 |
| [0082](0082-method-placement-coherence.md) | Associated functions; placement; coherence; no ambiguous calls *(delegated)* | Accepted | 2026-10-04 |
| [0083](0083-attribute-syntax.md) | Attributes `@name(args)`, same line as the declaration *(delegated)* | Accepted | 2026-10-04 |
| [0084](0084-no-shadowing-module-names.md) | Locals can't reuse module-level, imported or prelude names *(delegated)* | Accepted | 2026-10-04 |
| [0085](0085-interim-local-testing-gate.md) | Interim gate: local Linux (AI) and macOS (designer) runs; no hosted CI | Accepted (interim) | 2026-10-04 |
| [0086](0086-finish-core-design-before-stage-2.md) | Finish the core design before Stage 2; Hello World is the first milestone | Accepted | 2026-10-04 |
| [0087](0087-private-rust-toolchain.md) | Pinned Rust toolchain installed privately inside the repository | Accepted | 2026-10-04 |
