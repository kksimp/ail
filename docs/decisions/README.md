# Architecture Decision Records

Each design decision has its own file (see [ADR 0001](0001-record-decisions-as-adrs.md)).
New ADRs copy [0000-template.md](0000-template.md) and take the next free number.
ADRs are never deleted. Replaced decisions are marked `Superseded by NNNN`.

Status legend: **Accepted** is decided; the spec and code may still be catching up (see [status.md](../status.md)). **Proposed** is not decided or not built: *planned, not implemented* means the direction is chosen but no code exists, so do not read it as shipped behaviour. **Deferred**: direction chosen, not built, not in 0.1.0. **Superseded** and **Rejected** are kept for history.

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
| [0010](0010-no-llvm-backend.md) | No LLVM or other external backend; existing compilers for comparison only | Accepted (amended by 0381) | 2026-10-04 |
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
| [0058](0058-struct-literal-syntax.md) | Struct literals `Type(field: value)`; `name: value` for named args | Accepted; amended by 0142 | 2026-10-04 |
| [0059](0059-default-numeric-spelling.md) | `int`/`i64` and `float`/`f64` both canonical | Accepted | 2026-10-04 |
| [0060](0060-mutable-value-semantics.md) | AIL uses mutable value semantics | Accepted | 2026-10-04 |
| [0061](0061-call-site-mut.md) | `mut` parameters are marked `mut` at the call site | Accepted | 2026-10-04 |
| [0062](0062-second-class-borrows.md) | Second-class borrows; no lifetime annotations | Accepted | 2026-10-04 |
| [0063](0063-use-site-ref.md) | Shared mutable identity via use-site `ref T` | Accepted | 2026-10-04 |
| [0064](0064-exclusivity.md) | Exclusivity: overlap is a compile error; runtime check when undecidable | Accepted; partly superseded by 0119 | 2026-10-04 |
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
| [0082](0082-method-placement-coherence.md) | Associated functions; placement; coherence; no ambiguous calls *(delegated)* | Accepted; amended by 0137 | 2026-10-04 |
| [0083](0083-attribute-syntax.md) | Attributes `@name(args)`, same line as the declaration *(delegated)* | Accepted | 2026-10-04 |
| [0084](0084-no-shadowing-module-names.md) | Locals can't reuse module-level, imported or prelude names *(delegated)* | Accepted | 2026-10-04 |
| [0085](0085-interim-local-testing-gate.md) | Interim gate: local Linux (AI) and macOS (designer) runs; no hosted CI | Accepted (interim) | 2026-10-04 |
| [0086](0086-finish-core-design-before-stage-2.md) | Finish the core design before Stage 2; Hello World is the first milestone | Accepted | 2026-10-04 |
| [0087](0087-private-rust-toolchain.md) | Pinned Rust toolchain installed privately inside the repository | Accepted | 2026-10-04 |
| [0088](0088-entry-point-and-print.md) | `fn main()` entry point; prelude `print` ends the line *(delegated)* | Accepted | 2026-10-04 |
| [0089](0089-strings.md) | Strings: UTF-8, `{expr}` interpolation, no direct indexing *(delegated)* | Accepted; amended by 0135 | 2026-10-04 |
| [0090](0090-collections.md) | `List`/`Map`/`Set` with `[...]`/`{...}` literals *(delegated)* | Accepted; amended by 0143 | 2026-10-04 |
| [0091](0091-control-flow.md) | Expression blocks, `if`/`elif`/`else`, `for in`, `while`, ranges *(delegated)* | Accepted | 2026-10-04 |
| [0092](0092-operators.md) | `and`/`or`/`not`; compound assignment; methods always use `()` *(delegated)* | Accepted | 2026-10-04 |
| [0093](0093-optional-values.md) | `none`; implicit wrapping into optionals *(delegated)* | Accepted | 2026-10-04 |
| [0094](0094-numeric-literals.md) | Numeric literal forms *(delegated)* | Accepted | 2026-10-04 |
| [0095](0095-colorless-tasks.md) | Colorless tasks: any function may block; no async/await | Accepted | 2026-10-04 |
| [0096](0096-scope-bound-tasks.md) | Scope-bound tasks; unique handles must be waited | Accepted | 2026-10-04 |
| [0097](0097-directory-modules.md) | A module is a directory of `.ail` files | Accepted | 2026-10-04 |
| [0098](0098-pub-visibility.md) | `pub` marks exported items; private by default | Accepted | 2026-10-04 |
| [0099](0099-qualified-imports.md) | `use a.b` imports modules; qualified names only *(delegated)* | Accepted | 2026-10-04 |
| [0100](0100-task-communication.md) | Tasks communicate by value, channels, or synchronised types *(delegated)* | Accepted | 2026-10-04 |
| [0101](0101-single-file-programs.md) | Single `.ail` files run without a manifest *(delegated)* | Accepted | 2026-10-04 |
| [0102](0102-unsafe-blocks.md) | `unsafe` blocks; packages opt in to unsafe | Accepted | 2026-10-04 |
| [0103](0103-c-header-import.md) | C via header import (own parser) or manual declarations | Accepted | 2026-10-04 |
| [0104](0104-c-calls-unsafe.md) | Every C call is unsafe; safe AIL wrappers | Accepted | 2026-10-04 |
| [0105](0105-libc-only-with-c.md) | Linux links libc only when a program uses C | Accepted | 2026-10-04 |
| [0106](0106-unsafe-operations.md) | The closed set of unsafe operations; raw pointers `*T` *(delegated)* | Accepted | 2026-10-04 |
| [0107](0107-unknown-names.md) | Every name must resolve; "did you mean" suggestions *(delegated)* | Accepted | 2026-10-04 |
| [0108](0108-call-rules-and-syntax-errors.md) | Calls: one argument per parameter, exact types; `gram.syntax.1` *(delegated)* | Accepted | 2026-10-04 |
| [0109](0109-merge-via-pull-requests.md) | Code branches merge via GitHub PRs; GitHub deletes them on merge | Accepted | 2026-10-04 |
| [0110](0110-bool.md) | `bool` with `true`/`false`; conditions must be `bool` *(delegated)* | Accepted | 2026-10-04 |
| [0111](0111-if-expression-layout.md) | `elif`/`else` align with the line that starts the `if` (corrects 0091's example) *(delegated)* | Accepted | 2026-10-04 |
| [0112](0112-operator-precedence.md) | Operator precedence; comparisons don't chain *(delegated)* | Accepted | 2026-10-04 |
| [0113](0113-struct-literal-field-order.md) | Struct literals name every field once, in any order *(delegated)* | Accepted; amended by 0142 | 2026-10-04 |
| [0114](0114-reserved-keywords.md) | Reserved keywords; no contextual keywords *(delegated)* | Accepted | 2026-10-04 |
| [0115](0115-integer-division.md) | Integer division truncates toward zero *(delegated)* | Accepted | 2026-10-04 |
| [0116](0116-core-library-basics.md) | Sizes and indexes are `int`; `len`, `push`; text forms *(delegated)* | Accepted | 2026-10-04 |
| [0117](0117-typing-rules.md) | Typing rules for fields, assignment, `match`, loops and places *(delegated)* | Accepted | 2026-10-04 |
| [0118](0118-trailing-commas.md) | A trailing comma is allowed in every comma-separated list *(delegated)* | Accepted | 2026-10-04 |
| [0119](0119-copy-read-only-overlaps.md) | A read-only argument that overlaps a `mut` argument is passed as a copy (partly supersedes 0064) | Accepted | 2026-10-05 |
| [0120](0120-error-keyword.md) | Error types are declared with the `error` keyword (amends 0114) | Accepted | 2026-10-05 |
| [0121](0121-for-mut.md) | `for mut x in xs` changes list elements in place | Accepted | 2026-10-05 |
| [0122](0122-error-unions-and-reports.md) | Error unions `A \| B`; `?` needs a matching error type; failures must be handled; default error form *(delegated)* | Accepted | 2026-10-05 |
| [0123](0123-variants-with-data.md) | Variants with data are built with named fields and matched by position *(delegated)* | Accepted; amended by 0142 | 2026-10-05 |
| [0124](0124-string-split-and-to-int.md) | `s.split(sep)` and `s.to_int()` *(delegated)* | Accepted | 2026-10-05 |
| [0125](0125-if-binding.md) | `if x := e` handles success or absence as a statement; `else err` binds the error *(delegated)* | Accepted; amended by 0139 | 2026-10-05 |
| [0126](0126-automatic-equality-and-text.md) | Every type gets `==` and a text form automatically (amends 0073) | Accepted; amended by 0133 | 2026-10-05 |
| [0127](0127-standard-comparison-and-text-traits.md) | The standard traits `Eq`, `Ord` and `ToStr`; an empty `impl` derives them *(delegated)* | Accepted; amended by 0130, 0137 | 2026-10-05 |
| [0128](0128-methods-on-generic-types.md) | Methods and `impl`s on a generic type use its bare name; `Self` *(delegated)* | Accepted | 2026-10-05 |
| [0129](0129-using-generics-and-traits.md) | Type arguments are inferred; values convert to `any Trait` implicitly; rules for `impl` blocks and `any` calls *(delegated)* | Accepted | 2026-10-05 |
| [0130](0130-floats.md) | `float` is IEEE 754 binary64, with Python's text form and a total `cmp` (details delegated) | Accepted | 2026-10-05 |
| [0131](0131-maps-and-sets.md) | Maps and sets keep insertion order; the collection API, places and loops (details delegated) | Accepted | 2026-10-05 |
| [0132](0132-module-constants.md) | Module constants: `NAME := expr` with a constant expression (details delegated) | Accepted | 2026-10-05 |
| [0133](0133-automatic-hash.md) | `Hash` is automatic, like `Eq` and `ToStr` (amends 0126) | Accepted; amended by 0955 | 2026-10-05 |
| [0134](0134-characters.md) | `char`: traits, classification, digits and code points *(delegated)* | Accepted | 2026-10-05 |
| [0135](0135-string-literals-and-methods.md) | Multi-line and raw strings, `\'`, and the `str` methods (amends 0089) *(delegated)* | Accepted | 2026-10-05 |
| [0136](0136-list-methods-and-slices.md) | List methods, open-ended slices that trap, and `xs.sum()` *(delegated)* | Accepted | 2026-10-05 |
| [0137](0137-integer-methods-and-min-max.md) | Integer arithmetic methods; `min`/`max` as `Ord` methods; inherent methods beat prelude defaults (amends 0082, 0127) *(delegated)* | Accepted | 2026-10-05 |
| [0138](0138-literal-and-or-patterns.md) | Literal patterns, or-patterns, and one-line statement arms *(delegated)* | Accepted | 2026-10-05 |
| [0139](0139-unit-results-statement-else-and-divergence.md) | `! E` results; statement `else`; divergence is a rule, not a type (amends 0125) *(delegated)* | Accepted | 2026-10-05 |
| [0140](0140-diagnostics-for-other-languages.md) | Names other languages spell differently get a fixed help *(delegated)* | Accepted | 2026-10-05 |
| [0141](0141-implicit-self.md) | Inside a method, the receiver's fields and methods are in scope by bare name | Accepted (formatter clause superseded by 1081) | 2026-10-05 |
| [0142](0142-positional-constructor-arguments.md) | Struct and variant values can be built positionally when no swap could compile (amends 0058) | Accepted | 2026-10-05 |
| [0143](0143-empty-literals-typed-by-use.md) | An empty `[]` or `{}` declared with `:=` takes its type from its first use (amends 0090) | Accepted | 2026-10-05 |
| [0144](0144-else-as-an-expression.md) | `else` is also an expression: `e else d` and `e else return ...` *(delegated)* | Accepted | 2026-10-05 |
| [0145](0145-diagnostics-schema-v1.md) | Version the JSON diagnostics as schema v1, with spans, notes and fixes *(delegated)* | Accepted | 2026-10-20 |
| [0146](0146-ail-explain.md) | `ail explain RULE` prints a rule from the embedded specification *(delegated)* | Accepted | 2026-10-20 |
| [0147](0147-typed-holes-todo.md) | Typed holes: the keyword `todo` *(delegated)* | Accepted | 2026-10-20 |
| [0150](0150-ail-api-outline.md) | `ail api` prints a program's declarations without bodies, as one-line JSON *(delegated)* | Accepted | 2026-10-20 |
| [0151](0151-stable-symbol-ids-and-references.md) | Symbols have path ids; `--symbols` lists every reference by id *(delegated)* | Accepted | 2026-10-20 |
| [0152](0152-doc-comment-attachment.md) | A doc comment is the run of `##` lines directly above a declaration *(delegated)* | Accepted | 2026-10-20 |
| [0153](0153-package-root-and-loading.md) | A package is rooted at the directory run; imports load lazily and can't form cycles *(delegated)* | Accepted | 2026-10-20 |
| [0154](0154-imports-per-file-at-the-top.md) | Imports come first, apply to their file, and need unique names *(delegated)* | Accepted | 2026-10-20 |
| [0155](0155-member-visibility-and-method-placement.md) | Methods live in their type's module; private fields block outside construction *(delegated)* | Accepted | 2026-10-20 |
| [0156](0156-bootstrap-modules-by-qualified-names.md) | The bootstrap gives every item its qualified name before checking *(delegated)* | Accepted | 2026-10-20 |
| [0157](0157-standard-os-modules.md) | The machine is reached through five built-in standard modules, `use std.fs` and friends *(delegated)* | Accepted | 2026-10-20 |
| [0158](0158-standard-io-errors.md) | Standard failures are `fs.IoError` and `proc.ProcError` with a short `kind` *(delegated)* | Accepted | 2026-10-20 |
| [0159](0159-program-arguments.md) | `ail run FILE ARGS...` gives the program its arguments and its path *(delegated)* | Accepted | 2026-10-20 |
| [0160](0160-process-runs.md) | `proc.run` runs a program directly and fails only if it can't start *(delegated)* | Accepted | 2026-10-20 |
| [0161](0161-std-json-in-the-prelude.md) | `std.json` lives in the prelude as `Json`, `parse_json`, with `float` numbers *(delegated)* | Accepted | 2026-10-20 |
| [0162](0162-optional-prelude-modules.md) | Optional prelude modules are read only when named *(delegated)* | Accepted | 2026-10-20 |
| [0163](0163-inferred-effect-sets.md) | Every function has an inferred effect set, shown by `ail effects` *(delegated)* | Accepted | 2026-10-20 |
| [0164](0164-pure-functions.md) | `@pure` rejects a function with I/O, `unsafe`, `spawn` or a capability *(delegated)* | Accepted | 2026-10-20 |
| [0165](0165-capability-denial.md) | `--deny` rejects at compile time a program that reaches a denied capability *(delegated)* | Accepted | 2026-10-20 |
| [0167](0167-test-functions-and-ail-test.md) | Tests are `@test` functions run by `ail test FILE [--json]` *(delegated)* | Accepted | 2026-10-20 |
| [0168](0168-assert-and-assert-eq.md) | `assert` and `assert_eq` are prelude functions allowed anywhere *(delegated)* | Accepted | 2026-10-20 |
| [0169](0169-canonical-formatting-style.md) | One canonical layout, written by `ail fmt`, with a 100-column limit *(delegated)* | Accepted | 2026-10-20 |
| [0170](0170-ail-fmt-command.md) | `ail fmt` reprints the token stream, and refuses any change but layout *(delegated)* | Accepted | 2026-10-20 |
| [0172](0172-spec-conformance-harness.md) | Every `ail` block in the spec runs as a conformance test, tagged by expectation *(delegated)* | Accepted | 2026-10-20 |
| [0173](0173-format-specifications-in-interpolation.md) | Interpolation takes format specs: `{x:>8}`, `{r:.2}`, `{n:0>4x}` (resolves OQ-041) *(delegated)* | Accepted | 2026-10-20 |
| [0174](0174-language-card.md) | A language card: the whole language in one read under 4,000 tokens, cut from the quick reference *(delegated)* | Accepted | 2026-10-20 |
| [0175](0175-interpreter-fast-paths.md) | Speed up the interpreter with local fast paths and a fast hasher *(delegated)* | Accepted | 2026-10-20 |
| [0176](0176-keep-name-based-locals.md) | Keep name-based local lookup in the interpreter *(delegated)* | Accepted | 2026-10-20 |
| [0177](0177-std-ui-text-grid-window.md) | `std.ui` is a window of text cells, shimmed with two pinned crates *(delegated)* | Accepted | 2026-10-20 |
| [0178](0178-window-host-on-main-thread.md) | The main thread hosts the window; the interpreter keeps the grid *(delegated)* | Accepted | 2026-10-20 |
| [0179](0179-key-events-ctrl-chords.md) | Keys are typed characters, named keys, or Ctrl chords; Command is Ctrl on macOS *(delegated)* | Accepted | 2026-10-20 |
| [0180](0180-editor-written-in-ail.md) | Ship a code editor written in AIL (`tools/edit/`), with a headless core *(delegated)* | Accepted | 2026-10-20 |
| [0181](0181-editor-screen-as-text-runs.md) | The editor's screen is a list of colored text runs plus plain rows *(delegated)* | Accepted | 2026-10-20 |
| [0182](0182-comprehensions.md) | List, set and map comprehensions: `[f(x) for x in xs if c]` *(delegated)* | Accepted | 2026-10-20 |
| [0183](0183-bare-variant-patterns.md) | A bare variant pattern matches any fields: `Rect =>` for `Rect(_, _)` *(delegated)* | Accepted | 2026-10-20 |
| [0184](0184-round-3-syntax-measurements.md) | Keep `:=`, `fn`, explicit `self`, return types and `use` lines after round-3 measurement *(delegated)* | Accepted | 2026-10-20 |
| [0186](0186-deterministic-task-scheduler.md) | Run tasks in the interpreter on a deterministic, one-at-a-time scheduler; deadlocks trap *(delegated)* | Accepted | 2026-10-20 |
| [0187](0187-trapped-and-task-groups.md) | `wait()` fails with `Trapped`; groups hold tasks that return nothing *(delegated)* | Accepted | 2026-10-20 |
| [0188](0188-channel-api.md) | Channels: `Channel[T](capacity)`, `send`, `recv() T?`, `close` *(delegated)* | Accepted | 2026-10-20 |
| [0189](0189-mutex-access-form.md) | Mutex access: `get`, `set`, `swap`, and `lock`/`unlock` pairs *(delegated)* | Accepted | 2026-10-20 |
| [0190](0190-moves-and-use-after-move.md) | Check moves on every path; only owned values move; `move` moves any type *(delegated)* | Accepted | 2026-10-20 |
| [0191](0191-cleanup-with-drop.md) | Clean up unique types with the prelude trait `Drop`; limit where unique values live *(delegated)* | Accepted | 2026-10-20 |
| [0192](0192-ref-equality-and-text.md) | Compare `ref` values by identity and show their contents *(delegated)* | Accepted | 2026-10-20 |
| [0193](0193-ref-cell-exclusivity.md) | Trap when a `ref` cell is used through another copy during a `mut` access *(delegated)* | Accepted | 2026-10-20 |
| [0194](0194-sized-numeric-types.md) | Sized numeric types: contextual literals, same-type arithmetic, trapping overflow *(delegated)* | Accepted | 2026-10-20 |
| [0195](0195-sized-numeric-conversions.md) | Sized numbers convert with `to_<type>()` methods *(delegated)* | Accepted | 2026-10-20 |
| [0196](0196-inline-attribute.md) | `@inline` is accepted and recorded as a hint *(delegated)* | Accepted | 2026-10-20 |
| [0197](0197-deprecated-attribute.md) | `@deprecated("reason")` warns at each use *(delegated)* | Accepted | 2026-10-20 |
| [0198](0198-test-properties.md) | Run a test with parameters as a property *(delegated)* | Accepted | 2026-10-20 |
| [0199](0199-property-seeds-and-shrinking.md) | Seed properties from the test name and shrink failures *(delegated)* | Accepted | 2026-10-20 |
| [0200](0200-doc-examples-as-tests.md) | Run doc-comment examples as tests *(delegated)* | Accepted | 2026-10-20 |
| [0201](0201-doc-example-output-comments.md) | Assert example output with `# => text` comments *(delegated)* | Accepted | 2026-10-20 |
| [0202](0202-more-machine-applicable-fixes.md) | Offer fixes for five more common mistakes: `mut`, `elif`, `?`, missing match arms, `use std.x` *(delegated)* | Accepted | 2026-10-20 |
| [0203](0203-ail-fix-command.md) | `ail fix` applies diagnostics' fixes until none are left *(delegated)* | Accepted | 2026-10-20 |
| [0204](0204-anchored-fix-edits.md) | The checker names some fix positions by rule, and the tool resolves them *(delegated)* | Accepted | 2026-10-20 |
| [0205](0205-related-notes-on-duplicates.md) | Duplicate declarations point at the first one *(delegated)* | Accepted | 2026-10-20 |
| [0206](0206-std-random.md) | Add `std.random` with a process-wide seedable generator *(delegated)* | Accepted | 2026-10-20 |
| [0207](0207-deterministic-mode.md) | `ail run --deterministic` and a determinism guarantee for single-task programs (resolves OQ-046) *(delegated)* | Accepted | 2026-10-20 |
| [0208](0208-str-len-counts-characters.md) | `s.len()` counts characters, `s.byte_len()` bytes (resolves OQ-043) *(delegated)* | Accepted | 2026-10-20 |
| [0209](0209-nested-optionals.md) | Nested optionals are distinct types and `none` is the outermost (resolves OQ-044) *(delegated)* | Accepted | 2026-10-20 |
| [0210](0210-contract-syntax.md) | Contracts are `requires` and `ensures` lines under the signature; `result` names the value *(delegated)* | Accepted | 2026-10-20 |
| [0211](0211-contract-runtime-checks.md) | Contracts are checked at every call and trap with the values, blaming caller or function *(delegated)* | Accepted | 2026-10-20 |
| [0212](0212-pure-conditions-and-no-contracts.md) | Conditions are pure, so `--no-contracts` can't change a program *(delegated)* | Accepted | 2026-10-20 |
| [0213](0213-contracts-in-the-api.md) | `ail api` shows contracts as part of the signature *(delegated)* | Accepted | 2026-10-20 |
| [0214](0214-list-editing-methods.md) | List `insert`, `remove_at`, `extend`, `clear`, `reverse`, `first` (resolves OQ-047) *(delegated)* | Accepted | 2026-10-20 |
| [0215](0215-str-slice-and-search-methods.md) | `str` `slice`, `char_at`, `find`, `replace`, `to_lower`, `to_upper` (resolves OQ-048) *(delegated)* | Accepted | 2026-10-20 |
| [0217](0217-agent-corpus-standing-benchmark.md) | Keep an agent-tool corpus as the standing token and speed benchmark; amends 0029 layer 1 *(delegated)* | Accepted | 2026-10-20 |
| [0218](0218-agent-corpus-conventions.md) | Conventions for the agent-tool corpus programs *(delegated)* | Accepted | 2026-10-20 |
| [0219](0219-interpreter-speed-round-two.md) | Speed up the interpreter again with int fast paths, pooled allocations and in-place updates *(delegated)* | Accepted | 2026-10-20 |
| [0220](0220-interpreter-site-caches.md) | Cache what a call site names, by the address of its expression *(delegated)* | Accepted | 2026-10-20 |
| [0221](0221-ail-skips-toolchain-probe.md) | `./ail` runs the built binary directly when it is up to date *(delegated)* | Accepted | 2026-10-20 |
| [0222](0222-four-compiler-representations.md) | The compiler has four representations: AST, HIR, MIR and machine IR *(delegated)* | Accepted | 2026-10-20 |
| [0223](0223-hir-is-the-versioned-api-shape.md) | The HIR's printed shape is the versioned API; MIR stays internal *(delegated)* | Accepted | 2026-10-20 |
| [0224](0224-incremental-cache-keys.md) | Incremental queries are keyed by stable ids and fingerprinted without spans *(delegated)* | Accepted | 2026-10-20 |
| [0225](0225-constants-call-compile-time-safe-functions.md) | Constants may call compile-time-safe functions, computed with the interpreter's semantics (amends 0132) *(delegated)* | Accepted | 2026-10-20 |
| [0226](0226-compile-time-limits-determinism-and-caching.md) | Compile-time evaluation has step and memory limits, is deterministic, and is cached by what it ran *(delegated)* | Accepted | 2026-10-20 |
| [0227](0227-derive-generators.md) | A trait may name a derive generator: a compile-time function from `TypeInfo` to AIL source *(delegated)* | Accepted | 2026-10-20 |
| [0228](0228-generated-code-is-visible-hir.md) | Generated code is ordinary HIR with a provenance link, shown by `ail api --expanded` *(delegated)* | Accepted | 2026-10-20 |
| [0229](0229-compile-time-reflection-only.md) | Reflection is compile-time only: generators receive a `TypeInfo` value *(delegated)* | Accepted | 2026-10-20 |
| [0230](0230-package-manifest-ail-pkg.md) | A package's manifest is `ail.pkg`, one line per dependency *(delegated)* | Accepted | 2026-10-20 |
| [0231](0231-path-and-git-dependencies.md) | Dependencies come from a path or a pinned git commit, under one global name; a registry waits for the designer *(delegated)* | Accepted | 2026-10-20 |
| [0232](0232-capabilities-per-dependency.md) | The manifest grants each dependency its capabilities, checked at compile time (resolves OQ-045) *(delegated)* | Accepted | 2026-10-20 |
| [0233](0233-lockfile-and-reproducible-builds.md) | `ail.lock` pins every git dependency by commit and content hash; cache keys include grants *(delegated)* | Accepted | 2026-10-20 |
| [0236](0236-automatic-parallel-loops.md) | Run provably independent loops in parallel, with results identical to sequential *(delegated)* | Accepted | 2026-10-20 |
| [0237](0237-par-for.md) | `par for` asserts independence; provable conflicts are errors *(delegated)* | Accepted | 2026-10-20 |
| [0238](0238-parallel-runtime-model.md) | One work-stealing pool for tasks and parallel loops, with a minimum-work threshold *(delegated)* | Accepted | 2026-10-20 |
| [0239](0239-float-sum-fixed-tree.md) | Float `sum()` adds in a fixed four-lane tree (amends 0136) *(delegated)* | Accepted | 2026-10-20 |
| [0240](0240-std-ui-mouse-scroll-clipboard-title.md) | `std.ui` adds mouse, scroll, clipboard and title *(delegated)* | Accepted | 2026-10-20 |
| [0241](0241-editor-tabs-and-running.md) | The editor gets tabs, a file picker, mouse input and F5/F6 runs in its window front end *(delegated)* | Accepted | 2026-10-20 |
| [0242](0242-std-ui-modifiers-method.md) | `Window.modifiers()` reports the modifiers of the last event *(delegated)* | Accepted | 2026-10-20 |
| [0243](0243-std-ui-font-size.md) | `Window.set_font_size` picks from fixed bitmap sizes and keeps the window's pixels *(delegated)* | Accepted | 2026-10-20 |
| [0244](0244-editor-rename-symbol.md) | Editor F2 renames through `ail api` references, else by whole word, after a confirming prompt *(delegated)* | Accepted | 2026-10-20 |
| [0246](0246-simd-vectorisation.md) | Vectorise independent loops on both baselines; never contract to FMA implicitly *(delegated)* | Accepted | 2026-10-20 |
| [0247](0247-std-tensor-and-accel-capability.md) | NPUs and GPUs through a `std.tensor` module and an `accel` capability (design) *(delegated)* | Accepted | 2026-10-20 |
| [0250](0250-key-functions-and-take.md) | Sort and pick by key with `by: \|x\| key`, and `xs.take(n)` *(delegated)* | Accepted | 2026-10-20 |
| [0251](0251-map-op-assign-starts-from-zero.md) | `m[k] += v` on a missing key starts from zero *(delegated)* | Accepted | 2026-10-20 |
| [0252](0252-question-mark-goes-to-else.md) | A `?` on an optional inside an `else` value goes to that `else` *(delegated)* | Accepted | 2026-10-20 |
| [0253](0253-std-path.md) | `std.path`: `dir`, `base`, `ext` and `join` on path text *(delegated)* | Accepted | 2026-10-20 |
| [0254](0254-json-typed-fields-and-set.md) | `Json` gets typed field getters and `set` *(delegated)* | Accepted | 2026-10-20 |
| [0255](0255-round-4-token-measurements.md) | Round-4 token measurements on the agent-tool corpus *(delegated)* | Accepted | 2026-10-20 |
| [0258](0258-api-locals-query.md) | `ail api --locals` lists the names in scope at a position, from facts the checker records *(delegated)* | Accepted | 2026-10-20 |
| [0259](0259-api-type-query.md) | `ail api --type` gives the type of the expression at a position, and the callee for a call *(delegated)* | Accepted | 2026-10-20 |
| [0260](0260-api-refs-query.md) | `ail api --refs` lists a symbol's definition and uses across user modules, and a local's uses in its scope *(delegated)* | Accepted | 2026-10-20 |
| [0261](0261-checker-recording-hooks.md) | The checker records locals and expression types only when `ail api` asks *(delegated)* | Accepted | 2026-10-20 |
| [0262](0262-report-foreign-syntax-once-with-help.md) | Report foreign syntax once, give every error a help, measure with a mistakes corpus *(delegated)* | Accepted | 2026-10-20 |
| [0263](0263-one-diagnostic-per-mistake-recovery.md) | One diagnostic per mistake: line-level recovery and whole-file re-indentation *(delegated)* | Accepted | 2026-10-20 |
| [0264](0264-anchored-call-rewrite-fixes.md) | Fixes that rewrite a call, a bracket form or a chained comparison are anchored to the source line *(delegated)* | Accepted | 2026-10-20 |
| [0265](0265-runtime-trace-mode.md) | `ail run --trace` writes one event per line to standard error *(delegated)* | Accepted | 2026-10-20 |
| [0266](0266-trace-budget-and-value-form.md) | Traces are budgeted, and show values structurally without user code *(delegated)* | Accepted | 2026-10-20 |
| [0267](0267-trap-snapshots.md) | A trap carries a snapshot of the stack and its values *(delegated)* | Accepted | 2026-10-20 |
| [0268](0268-watch-and-no-debugger-protocol.md) | `--watch NAME` shows a variable's assignments; no debugger protocol for now *(delegated)* | Accepted | 2026-10-20 |
| [0269](0269-fuzz-generator-design.md) | The fuzz harness generates typed programs, edge-value probes and token-level mutants from a seed *(delegated)* | Accepted | 2026-10-20 |
| [0270](0270-fuzz-properties.md) | The fuzz harness checks crash-freedom, determinism, fmt idempotence, api validity and a contracts differential *(delegated)* | Accepted | 2026-10-20 |
| [0271](0271-every-crash-becomes-a-case.md) | Every fuzz crash becomes a regression case; `repeat` and `pad_*` trap above 2^30 bytes *(delegated)* | Accepted | 2026-10-20 |
| [0272](0272-interpreter-thread-pool.md) | The interpreter runs `par for` on a pool of copied values *(delegated)* | Accepted | 2026-10-20 |
| [0273](0273-pure-tasks-off-the-scheduler.md) | Pure tasks run on their own threads; tasks with handles keep the scheduler *(delegated)* | Accepted | 2026-10-20 |
| [0274](0274-pool-fallback-runs-in-order.md) | When the pool can't give the sequential result, the loop reruns in order *(delegated)* | Accepted | 2026-10-20 |
| [0275](0275-run-parallel-flag-and-pool-log.md) | `ail run --parallel`, `AIL_THREADS` and `AIL_PAR_LOG` *(delegated)* | Accepted | 2026-10-20 |
| [0276](0276-language-card-budget-8000.md) | The language card budget is 8,000 tokens on every tokenizer (amends 0174) | Accepted | 2026-10-20 |
| [0277](0277-standard-modules-without-use.md) | Standard modules are in scope without `use` *(delegated)* | Accepted | 2026-10-20 |
| [0278](0278-messages-on-error-types.md) | Error types carry their messages *(delegated)* | Accepted | 2026-10-20 |
| [0279](0279-question-mark-on-optionals-fails-with-absent.md) | `?` on an optional fails with `Absent` where errors are inferred, and tests can fail *(delegated)* | Accepted | 2026-10-20 |
| [0280](0280-one-line-if-and-round-5-measurements.md) | One-line `if c then a else b`, `{}` typed by `m[k] op= n`, and the round-5 measurements *(delegated)* | Accepted | 2026-10-20 |
| [0281](0281-std-fs-walk-glob-metadata.md) | `std.fs` gets `walk`, `glob`, `metadata`, `rename` and `copy` *(delegated)* | Accepted | 2026-10-20 |
| [0282](0282-std-proc-input-timeout-which.md) | `proc.run` takes input and a time limit; `proc.which` and `env.vars` *(delegated)* | Accepted | 2026-10-20 |
| [0283](0283-std-net-http-client.md) | `std.net`: a minimal HTTP/1.1 client, without TLS for now *(delegated)* | Accepted | 2026-10-20 |
| [0284](0284-trailing-optional-std-parameters.md) | A standard function's trailing optional parameters may be left off *(delegated)* | Accepted | 2026-10-20 |
| [0285](0285-error-flow-diagnostics-and-fixes.md) | Error-flow diagnostics carry applicable helps and fixes *(delegated)* | Accepted | 2026-10-20 |
| [0286](0286-failure-paths-check-errors.md) | `ail check --errors`: every failure path of a function as JSON *(delegated)* | Accepted | 2026-10-20 |
| [0287](0287-inferred-error-sets-in-api-and-worked-explain.md) | Inferred error sets in `ail api`, worked examples in `ail explain` *(delegated)* | Accepted | 2026-10-20 |
| [0288](0288-run-limits-timeout-memory-steps.md) | `ail run` takes `--timeout`, `--max-memory` and `--max-steps` *(delegated)* | Accepted | 2026-10-20 |
| [0289](0289-test-coverage.md) | `ail test --coverage` reports lines and functions run *(delegated)* | Accepted | 2026-10-20 |
| [0290](0290-test-failure-shows-trap-snapshot.md) | `ail test` prints the trap snapshot under a failure *(delegated)* | Accepted | 2026-10-20 |
| [0291](0291-editor-command-palette-one-key-list.md) | The editor's command palette and README key table come from one list; format moves to Alt-Shift-F *(delegated)* | Accepted | 2026-10-20 |
| [0292](0292-editor-project-search.md) | Editor project search: Ctrl-Shift-F, matches in the pane, F4 for the next *(delegated)* | Accepted | 2026-10-20 |
| [0293](0293-editor-file-tree-and-check-all-tabs.md) | Editor file tree (Ctrl-B) and checking every tab on save, counts on tab labels *(delegated)* | Accepted | 2026-10-20 |
| [0294](0294-std-ui-styles-colors-caret.md) | `std.ui` text styles, named colors and caret shapes *(delegated)* | Accepted | 2026-10-20 |
| [0295](0295-editor-themes.md) | Editor themes recolor the default colors by kind, from an AIL module *(delegated)* | Accepted | 2026-10-20 |
| [0296](0296-env-home-and-editor-settings.md) | `env.home()`, and editor settings in `~/.ail-edit` *(delegated)* | Accepted | 2026-10-20 |
| [0297](0297-no-bytecode-tier-in-the-oracle.md) | Keep the interpreter a tree walker; the compiled form belongs to Stage 3 *(delegated)* | Superseded by 0359 | 2026-10-20 |
| [0300](0300-for-over-a-channel.md) | Iterate a channel with `for v in ch`; no `select` *(delegated)* | Accepted | 2026-10-20 |
| [0301](0301-with-mutex-block.md) | Scoped mutex access: `with m as mut x` *(delegated)* | Accepted | 2026-10-20 |
| [0302](0302-par-comprehension.md) | Parallel map: `[f(x) par for x in xs]` *(delegated)* | Accepted | 2026-10-20 |
| [0303](0303-contract-old.md) | `old(e)` in `ensures`: the value on entry *(delegated)* | Accepted | 2026-10-20 |
| [0304](0304-trait-contract-inheritance.md) | Implementations inherit trait method contracts: own `requires` weaken, own `ensures` add *(delegated)* | Accepted | 2026-10-20 |
| [0305](0305-struct-invariants.md) | Struct `invariant` lines, checked at construction and after `mut self` methods *(delegated)* | Accepted | 2026-10-20 |
| [0306](0306-derive-generators-run-while-loading.md) | The bootstrap runs derive generators while it loads the program; `TypeInfo.is_error` *(delegated)* | Accepted | 2026-10-20 |
| [0307](0307-generated-associated-functions.md) | Generated text may add associated functions of the deriving type *(delegated)* | Accepted | 2026-10-20 |
| [0308](0308-tojson-fromjson-derives.md) | `ToJson` and `FromJson` are prelude derives written in AIL *(delegated)* | Accepted | 2026-10-20 |
| [0309](0309-api-expanded-output.md) | `ail api --expanded` lists generated functions under their `impl`; built-in derives by name *(delegated)* | Accepted | 2026-10-20 |
| [0310](0310-ail-api-card-format.md) | `ail api --card`: a program's own card *(delegated)* | Accepted | 2026-10-20 |
| [0311](0311-ail-api-card-budget-and-trimming.md) | Card budget, token estimate and trimming order *(delegated)* | Accepted | 2026-10-20 |
| [0312](0312-editor-agent-panel.md) | The editor's agent panel: info (Ctrl-I), trace (Shift-F5) and failure paths (Ctrl-Shift-E) in one pane *(delegated)* | Accepted | 2026-10-20 |
| [0313](0313-editor-coverage-overlay.md) | Editor coverage overlay (Ctrl-Shift-C) until the next edit *(delegated)* | Accepted | 2026-10-20 |
| [0314](0314-standard-modules-in-ail.md) | Standard modules written in AIL, starting with `std.ui.widgets` *(delegated)* | Accepted | 2026-10-20 |
| [0315](0315-widget-contract.md) | Widgets render pure runs and turn events into an `Action` *(delegated)* | Accepted | 2026-10-20 |
| [0316](0316-dashboard-example.md) | `examples/dashboard.ail` prints by default and opens a window with `--window` *(delegated)* | Accepted | 2026-10-20 |
| [0317](0317-move-self-receivers.md) | `move self` receivers consume the receiver; the call site is unmarked *(delegated)* | Accepted | 2026-10-20 |
| [0318](0318-unique-values-in-lists-and-maps.md) | Lists and maps may hold unique values; such a collection is unique *(delegated)* | Accepted | 2026-10-20 |
| [0319](0319-copying-methods-on-unique-collections.md) | Methods that copy elements are errors on unique collections; task handles stay out of lists *(delegated)* | Accepted | 2026-10-20 |
| [0320](0320-atomic-int-and-bool.md) | `Atomic[int]` and `Atomic[bool]`: shared cells for tasks and `par for` *(delegated)* | Accepted | 2026-10-20 |
| [0321](0321-task-cancellation.md) | Cancellation: `cancel`, `task.cancelled()`, `task.check()` *(delegated)* | Accepted | 2026-10-20 |
| [0322](0322-random-schedule-testing.md) | `--schedule=random[:SEED]`: replayable random interleavings *(delegated)* | Accepted | 2026-10-20 |
| [0323](0323-chained-call-lines.md) | A line starting with `.name` continues the statement; the formatter splits chains one call per line *(delegated)* | Accepted | 2026-10-20 |
| [0324](0324-attributes-on-their-own-line.md) | Attributes may sit on their own lines above a function, in name order *(delegated)* | Accepted | 2026-10-20 |
| [0325](0325-ail-api-on-package-directories.md) | `ail api` on package directories: outline, symbols, refs, type *(delegated)* | Accepted | 2026-10-20 |
| [0326](0326-ail-test-trace.md) | `ail test --trace` keeps a failing test's trace *(delegated)* | Accepted | 2026-10-20 |
| [0327](0327-watch-through-calls.md) | `--watch` follows a variable through `mut` calls *(delegated)* | Accepted | 2026-10-20 |
| [0328](0328-std-tensor-cpu-api.md) | `std.tensor`: an eager, value-typed `Tensor[T]` of `int` or `float` *(delegated)* | Accepted | 2026-10-20 |
| [0329](0329-std-tensor-kernels-and-dispatch.md) | Tensor kernels: AVX2, NEON or scalar at run time, one specified summation order *(delegated)* | Accepted | 2026-10-20 |
| [0330](0330-tensor-accelerator-backends-plug-in.md) | How an NPU or GPU backend plugs into `std.tensor` (design) *(delegated)* | Accepted | 2026-10-20 |
| [0331](0331-editor-program-card-view.md) | Editor program card view (Ctrl-Shift-I) from `ail api --card --json` *(delegated)* | Accepted | 2026-10-20 |
| [0332](0332-front-end-allocator.md) | `check`, `api` and `effects` use a front-end allocator (size-class free lists, huge pages) *(delegated)* | Accepted | 2026-10-20 |
| [0333](0333-no-front-end-disk-cache-yet.md) | No on-disk prelude or check-result cache yet; the saving is under 0.5 ms *(delegated)* | Accepted | 2026-10-20 |
| [0334](0334-fast-hashing-and-lexing-in-the-front-end.md) | Fast hasher in the checker, resolver and parser; keywords lexed without allocating *(delegated)* | Accepted | 2026-10-20 |
| [0335](0335-explain-and-fix-coverage.md) | Every emitted rule has an explain entry, and ten more mistakes have fixes *(delegated)* | Accepted | 2026-10-20 |
| [0336](0336-explain-listing-and-length.md) | `ail explain` lists every rule with its fix, and entries average under 120 tokens *(delegated)* | Accepted | 2026-10-20 |
| [0337](0337-agent-corpus-round-two.md) | The agent corpus grows to ten programs with Python twins; the five new ones use the round 5–7 features *(delegated)* | Accepted | 2026-10-20 |
| [0338](0338-card-json-positions.md) | `ail api --card --json` gives each declaration's file, line and col *(delegated)* | Accepted | 2026-10-20 |
| [0339](0339-editor-problems-and-sessions.md) | Editor problems pane (Ctrl-Shift-M), replace in files with preview (Ctrl-Shift-H), and tabs remembered per directory *(delegated)* | Accepted | 2026-10-20 |
| [0340](0340-set-operations.md) | `union`, `intersect` and `difference` on sets and lists *(delegated)* | Accepted | 2026-10-20 |
| [0341](0341-map-push.md) | `m.push(k, x)` appends to a list-valued map entry *(delegated)* | Accepted | 2026-10-20 |
| [0342](0342-named-arguments-for-std-optionals-rejected.md) | Named arguments for std optional parameters: measured no gain, not added *(delegated)* | Accepted | 2026-10-20 |
| [0343](0343-json-derives-maps-optionals-enum-payloads.md) | `ToJson`/`FromJson` derives for maps, any optional, enum payloads as `{"kind": ...}`, and error paths; generic `FromJson` came with ADR 0350 *(delegated)* | Accepted | 2026-10-20 |
| [0344](0344-tensor-operators-and-scalar-broadcast.md) | Tensor `+ - *`: elementwise, with a scalar on either side; matrix product stays `matmul` *(delegated)* | Accepted | 2026-10-20 |
| [0345](0345-tensor-axis-reductions-and-transpose.md) | Tensor `sum`/`max`/`mean` over an axis and `transpose`, in the one summation order *(delegated)* | Accepted | 2026-10-20 |
| [0346](0346-formatter-splits-at-binary-operators.md) | The formatter splits long lines before binary operators *(delegated)* | Accepted | 2026-10-20 |
| [0347](0347-watch-shows-collection-writes.md) | `--watch` shows writes to a collection's elements *(delegated)* | Accepted | 2026-10-20 |
| [0348](0348-supertraits.md) | Supertraits written like bounds, `trait Ord Eq` (OQ-039) *(delegated)* | Accepted | 2026-10-20 |
| [0349](0349-associated-functions-in-traits.md) | Associated functions in traits, called as `T.name()` (OQ-039) *(delegated)* | Accepted | 2026-10-20 |
| [0350](0350-default-and-generic-fromjson.md) | `Default` and generic `FromJson` through associated functions *(delegated)* | Accepted | 2026-10-20 |
| [0351](0351-cancel-trees-and-cancelled-send.md) | Cancel trees; `send` in a cancelled task drops its value *(delegated)* | Accepted | 2026-10-20 |
| [0352](0352-weaker-atomic-orderings.md) | Weaker atomic orderings as a trailing `order:` argument *(delegated)* | Accepted | 2026-10-20 |
| [0353](0353-editor-live-problems-and-reviewable-replace.md) | Editor problems list refreshes while shown; replace preview accepts or skips rows *(delegated)* | Accepted | 2026-10-20 |
| [0354](0354-editor-tests-on-save.md) | Editor tests on save (Ctrl-Shift-U) with counts in the status line *(delegated)* | Accepted | 2026-10-20 |
| [0355](0355-fetch-git-dependencies-with-system-git.md) | Fetch git dependencies with the system `git` into a read-only cache *(delegated)* | Accepted | 2026-10-20 |
| [0356](0356-ail-lock-pins-commits.md) | `ail.lock` pins each git dependency to a commit; `run` and `test` write it *(delegated)* | Accepted | 2026-10-20 |
| [0357](0357-ail-pkg-update.md) | `ail pkg update [NAME...]` re-resolves git dependencies *(delegated)* | Accepted | 2026-10-20 |
| [0358](0358-mistakes-corpus-for-rounds-5-9.md) | The mistakes corpus covers the features of rounds 5 to 9 *(delegated)* | Accepted | 2026-10-20 |
| [0359](0359-bytecode-tier-in-the-bootstrap.md) | Add a bytecode tier to the bootstrap interpreter (designer) | Superseded by 0873 | 2026-10-20 |
| [0360](0360-in-house-tls-for-std-net.md) | `https` in `std.net` through an in-house TLS implementation, not a crate (designer) | Accepted | 2026-10-20 |
| [0361](0361-bytecode-tier-design.md) | The bytecode tier: register code over the tree walker's environment, with hand-offs *(delegated)* | Superseded by 0873 | 2026-10-20 |
| [0362](0362-differential-harness-for-the-bytecode-tier.md) | `./ail selftest` runs every program on both tiers; `fuzz.prop.5` compares them *(delegated)* | Superseded by 0873 | 2026-10-20 |
| [0363](0363-bytecode-tier-methods-and-data.md) | The bytecode tier compiles method and trait calls, data construction, `match`, interpolation and `?` *(delegated)* | Superseded by 0873 | 2026-10-20 |
| [0364](0364-bytecode-tier-limits-coverage-and-traces.md) | Run limits, coverage and call and effect traces run on the bytecode tier *(delegated)* | Superseded by 0873 | 2026-10-20 |
| [0366](0366-tls-crypto-primitives.md) | TLS cryptographic primitives, constant time, tested against RFC/NIST vectors and OpenSSL *(delegated)* | Accepted | 2026-10-20 |
| [0367](0367-tls13-client-handshake.md) | The TLS 1.3 client handshake and record layer, tested on a recorded trace and live OpenSSL *(delegated)* | Accepted | 2026-10-20 |
| [0368](0368-tls-certificate-verification.md) | TLS certificate verification: X.509 chains to the platform's roots *(delegated)* | Accepted | 2026-10-20 |
| [0369](0369-https-in-std-net.md) | `https` in `std.net` over the in-house TLS 1.3 client, always verified *(delegated)* | Accepted | 2026-10-20 |
| [0371](0371-self-hosted-front-end-compared-by-dumps.md) | Compare the self-hosted lexer and parser with the bootstrap through text dumps *(delegated)* | Accepted | 2026-10-20 |
| [0372](0372-identifier-characters.md) | A name is letters, ASCII digits and `_` *(delegated)* | Accepted | 2026-10-20 |
| [0373](0373-self-hosted-diagnostics-are-rule-and-position.md) | The self-hosted front end reports a rule and a position, and runs the corpus in one process *(delegated)* | Accepted | 2026-10-20 |
| [0376](0376-syntax-tree-in-ail-and-its-json.md) | The syntax tree in AIL, `compiler/ast/`, and its JSON is the derived shape *(delegated)* | Accepted | 2026-10-20 |
| [0377](0377-checker-in-ail-ported-pass-for-pass.md) | The checker in AIL is ported pass for pass, and says what it covers *(delegated)* | Accepted | 2026-10-20 |
| [0378](0378-checker-gaps-are-skipped-not-guessed.md) | The AIL checker skips what it can't type yet, and the comparison skips it too *(delegated)* | Accepted | 2026-10-20 |
| [0379](0379-allow-attribute.md) | `@allow("rule")` before a function allows a named warning in it *(delegated)* | Accepted | 2026-10-20 |
| [0380](0380-self-hosted-checker-gate.md) | Checker changes are ported to the AIL checker in the same branch | Accepted | 2026-10-20 |
| [0740](0740-self-hosted-checker-parity-gate.md) | The self-hosted checker must match `ail check` with nothing skipped *(delegated)* | Accepted | 2026-10-20 |
| [0381](0381-driver-compilers-are-platform.md) | Accelerator drivers may compile code that AIL generates; the driver's compiler is platform (designer) | Accepted | 2026-10-20 |
| [0382](0382-no-matrix-product-operator.md) | The matrix product stays `a.matmul(b)`; no `@` operator (designer) | Accepted | 2026-10-20 |
| [0383](0383-no-package-registry.md) | No package registry: dependencies are git URLs, with tags as versions (designer) | Accepted | 2026-10-20 |
| [0384](0384-mir-shape-and-native-subset.md) | MIR: flat instructions, block parameters, built on the fly; the native subset *(delegated)* | Accepted | 2026-10-20 |
| [0385](0385-x86-64-code-generation.md) | x86-64 code generation: the C convention, scratch registers, checks that branch to trap stubs *(delegated)* | Accepted | 2026-10-20 |
| [0386](0386-linear-scan-register-allocation.md) | Register allocation by linear scan over reverse postorder, one location per value *(delegated)* | Accepted | 2026-10-20 |
| [0387](0387-static-executables-and-ail-build.md) | `ail build`: static executables with the runtime compiled from AIL, checked first *(delegated)* | Accepted | 2026-10-20 |
| [0388](0388-arm64-code-generation.md) | ARM64 code generation: AAPCS64, the same allocator, checks through the flags *(delegated)* | Accepted | 2026-10-20 |
| [0389](0389-mach-o-signed-by-the-backend.md) | Mach-O executables, dynamically loaded but importing nothing, signed ad hoc by the backend *(delegated)* | Accepted | 2026-10-20 |
| [0390](0390-native-subset-for-the-drag-race.md) | The native subset grows to the drag-race programs: arguments, `to_int() else`, constant expressions, `{x:.N}`, strings and nested lists *(delegated)* | Accepted | 2026-10-20 |
| [0391](0391-native-backend-optimisations.md) | First optimisations in the native backend: lifetime holes, MIR folding and CSE, inlining of small leaf functions, inline `push`, byte-sized `List[bool]`, ELF symbols *(delegated)* | Accepted | 2026-10-20 |
| [0392](0392-native-structs-and-methods.md) | Structs and methods in the native subset: heap header word, slots, copies at the boundary, `T.m` methods *(delegated)* | Accepted | 2026-10-20 |
| [0393](0393-native-modules.md) | Modules in the native backend: package loading, module-prefixed names, files in problems and traps *(delegated)* | Accepted | 2026-10-20 |
| [0394](0394-token-budgets-on-every-command.md) | Token budgets on every command: `--budget N`, most severe first, `--from K` *(delegated)* | Accepted | 2026-10-20 |
| [0395](0395-ail-test-update-rewrites-snapshots.md) | `ail test --update` rewrites failing literal expectations, `# =>` lines and `.out` files *(delegated)* | Accepted | 2026-10-20 |
| [0396](0396-ail-diff-semantic-diff.md) | `ail diff`: a semantic diff of declarations, diagnostics and test outcomes, also against a git revision *(delegated)* | Accepted | 2026-10-20 |
| [0397](0397-ail-edit-structured-edits-by-id.md) | `ail edit`: structured edits by declaration id, formatted and checked *(delegated)* | Accepted | 2026-10-20 |
| [0398](0398-record-and-replay.md) | Record and replay: `--record R` writes every effect's result as JSON lines, `--replay R` answers effects from it *(delegated)* | Accepted | 2026-10-20 |
| [0399](0399-compact-edits-and-diff-hunks.md) | Compact `ail edit` hunks by declaration name, `--test`, and changed lines in `ail diff` *(delegated)* | Accepted | 2026-10-20 |
| [0583](0583-native-coverage-tracker.md) | A native coverage tracker: which programs `ail build` handles, and why the rest fail *(delegated)* | Accepted | 2026-10-20 |
| [0584](0584-native-coverage-classification.md) | Native coverage: status rules, construct keys and counting *(delegated)* | Accepted | 2026-10-20 |
| [0577](0577-bounds-checks-in-counted-loops.md) | Bounds checks in counted loops: versioning with guards, hoisted loads, two-pointer loops *(delegated)* | Accepted | 2026-10-20 |
| [0578](0578-dominated-list-accesses.md) | Accesses that an earlier access or a `push` vouches for *(delegated)* | Accepted | 2026-10-20 |
| [0579](0579-list-presizing.md) | Sizing a list before the loop that fills it (`rt.list_reserve`) *(delegated)* | Accepted | 2026-10-20 |
| [0611](0611-overflow-checks-by-range-guards.md) | Overflow checks removed by range guards in versioned loops, duplicate checked operations dropped, x86-64 immediate and test forms *(delegated)* | Accepted | 2026-10-20 |
| [0618](0618-bytecode-typed-fast-paths.md) | Typed fast paths in the bytecode tier: floats, list element updates, `push`/`len`, computed `op=` and `mut` argument calls *(delegated)* | Superseded by 0873 | 2026-10-21 |
| [0670](0670-explain-examples-from-tagged-blocks.md) | `ail explain` examples: worked examples count in the audit; a `rules=` block fills a rule with no example in its section *(delegated)* | Accepted | 2026-10-21 |
| [0622](0622-loop-starts-steps-and-index-spans.md) | Versioned loops with computed starts and unchanging steps; index arithmetic bounded by a constant loop bound *(delegated)* | Accepted | 2026-10-21 |
| [0623](0623-branch-and-codegen-peepholes.md) | Branch and codegen peepholes: flipped `not` branches, bit operations into the result register, direct spill moves *(delegated)* | Accepted | 2026-10-21 |
| [0624](0624-loop-weighted-spill-choice.md) | Register allocation spills the value cheapest to keep in memory, weighted by loop depth *(delegated)* | Accepted | 2026-10-21 |
| [0646](0646-mistakes-corpus-round-two.md) | Mistakes corpus, round two: 96 programs on the batch's new features; `ail fix` resolves 60% *(delegated)* | Accepted | 2026-10-21 |
| [0647](0647-round-two-diagnostics-help-and-anchors.md) | Diagnostics for round-two mistakes: help texts, fix anchors, and rules for `<<`, tuples, `case` and open ranges *(delegated)* | Accepted | 2026-10-21 |
| [0648](0648-round-two-fix-rules-and-the-corpus-target.md) | Fix rules for round-two mistakes (renames with unchanged arguments, `=` to `:`, `@` to `matmul`, `or` to `else`), and the 60% target *(delegated)* | Accepted | 2026-10-21 |
| [0625](0625-address-folding.md) | Address folding for unchecked `base[y + k]` accesses, and callee-saved registers saved where the calls start (shrink-wrapping) *(delegated)* | Accepted | 2026-10-21 |
| [0562](0562-native-heap-size-classes-and-header.md) | Native heap: reserved region, 16 KiB pages, size-class free lists, header word *(delegated)* | Accepted | 2026-10-20 |
| [0563](0563-conservative-mark-sweep-collector.md) | The first native collector is conservative on the stack and registers, precise by kind inside objects *(delegated)* | Accepted | 2026-10-20 |
| [0564](0564-collection-trigger-and-sweep.md) | Collection trigger, sweep and measurements *(delegated)* | Accepted | 2026-10-20 |
| [0447](0447-calendar-in-ail-std-time-date.md) | The calendar is a standard module written in AIL: `std.time.date` *(delegated)* | Accepted | 2026-10-20 |
| [0448](0448-time-zones-from-the-platform-tz-database.md) | Time zones: UTC, fixed offsets and the platform tz database through `std.tz` *(delegated)* | Accepted | 2026-10-20 |
| [0449](0449-calendar-arithmetic-rules.md) | Calendar arithmetic: months clamp, days keep the wall clock, gaps and folds resolve forward *(delegated)* | Accepted | 2026-10-20 |
| [0450](0450-iso-8601-parse-and-print.md) | ISO 8601: an offset is required, `parse_in` supplies a zone, `iso()` prints round-trippable text *(delegated)* | Accepted | 2026-10-20 |
| [0451](0451-format-patterns-use-strftime-tokens.md) | `format` patterns use strftime tokens *(delegated)* | Accepted | 2026-10-20 |
| [0404](0404-native-calling-convention-and-value-layouts.md) | Native values: C-compatible calling convention, fixed layouts, runtime written in AIL on a machine model *(delegated)* | Accepted | 2026-10-20 |
| [0405](0405-native-traps.md) | Native traps print the interpreter's first line and exit 1 *(delegated)* | Accepted | 2026-10-20 |
| [0406](0406-raw-syscalls-for-native-runtime.md) | Raw syscalls for the first native runtime, including on macOS *(delegated)* | Accepted | 2026-10-20 |
| [0407](0407-bump-allocator-no-header.md) | Bump allocator over mmap, no object header, collector later *(delegated)* | Accepted | 2026-10-20 |
| [0408](0408-buffered-output.md) | Output is buffered in the runtime; errors are not *(delegated)* | Accepted | 2026-10-20 |
| [0409](0409-float-text-by-exact-digit-generation.md) | Float text by exact digit generation, written from the textbook *(delegated)* | Accepted | 2026-10-20 |
| [0412](0412-bytes-type.md) | `bytes` is an immutable value type of `u8` *(delegated)* | Accepted | 2026-10-20 |
| [0413](0413-bytes-literal.md) | The `bytes` literal is `b"..."`, ASCII with `\xHH` *(delegated)* | Accepted | 2026-10-20 |
| [0414](0414-bytes-methods.md) | `bytes` methods: strict `to_str`, lossy `text`, encodings on the type *(delegated)* | Accepted | 2026-10-20 |
| [0415](0415-binary-files-and-input.md) | Binary file and standard-input functions: `read_bytes`, `write_bytes`, `append_bytes` *(delegated)* | Accepted | 2026-10-20 |
| [0416](0416-net-bodies-as-bytes.md) | `std.net` bodies as `bytes`: new functions and last fields, `body` stays `str` *(delegated)* | Accepted | 2026-10-20 |
| [0417](0417-proc-run-bytes.md) | `proc.run_bytes`: bytes in, bytes out, `Output` unchanged *(delegated)* | Accepted | 2026-10-20 |
| [0418](0418-request-options-and-redirect-rules.md) | Request options, redirect rules and streaming deferred for `std.net` *(delegated)* | Accepted | 2026-10-20 |
| [0419](0419-http-server-as-accept-and-respond.md) | The HTTP server is `net.serve(addr)` with `accept` and `respond`; the handler form waits on function values *(delegated)* | Accepted | 2026-10-20 |
| [0420](0420-cooperative-socket-waits.md) | Network waits are cooperative: non-blocking sockets, yield, back off *(delegated)* | Accepted | 2026-10-20 |
| [0421](0421-raw-tcp-and-tls-conn.md) | Raw TCP: `Listener` and `Conn`, text first, TLS over the same `Conn` *(delegated)* | Accepted | 2026-10-20 |
| [0422](0422-net-serve-capability.md) | `net.serve` is a capability on top of `net` *(delegated)* | Accepted | 2026-10-20 |
| [0423](0423-net-platforms-and-deferred.md) | `std.net` on Windows and what is deferred *(delegated)* | Accepted | 2026-10-20 |
| [0424](0424-streaming-child-processes.md) | Streaming child processes: `proc.start`, with the child's streams as channels *(delegated)* | Accepted | 2026-10-20 |
| [0425](0425-child-is-unique-and-consumed.md) | A `Child` is a unique value, consumed by `wait` or `kill`, and ended with the program *(delegated)* | Accepted | 2026-10-20 |
| [0426](0426-scheduler-waits-for-child-streams.md) | The scheduler waits for a child's output instead of reporting a deadlock *(delegated)* | Accepted | 2026-10-20 |
| [0427](0427-task-select.md) | `task.select(chans)` waits on several channels and returns an index *(delegated)* | Accepted | 2026-10-20 |
| [0428](0428-wait-yields-and-time-limit-kills.md) | `proc.wait` lets other tasks run, and the time limit kills the child *(delegated)* | Accepted | 2026-10-20 |
| [0430](0430-unsafe-blocks-and-functions-built.md) | `unsafe` blocks and `unsafe fn`, with a manifest opt-in, are built *(delegated)* | Accepted | 2026-10-20 |
| [0431](0431-extern-c-fn-and-dynamic-loading.md) | `extern c fn` declares a C function, loaded dynamically on first call *(delegated)* | Accepted | 2026-10-20 |
| [0432](0432-c-calls-through-two-register-files.md) | C calls use two register-file trampolines, with no foreign-function library *(delegated)* | Accepted | 2026-10-20 |
| [0433](0433-raw-pointer-type.md) | `*T` is a raw pointer type with a C-layout target *(delegated)* | Accepted | 2026-10-20 |
| [0434](0434-std-mem-c-strings.md) | `std.mem` hands out C strings from C's allocator and traps on bad frees *(delegated)* | Accepted | 2026-10-20 |
| [0435](0435-pointer-operations-and-typed-alloc.md) | Pointer reads, writes and offsets are methods; blocks are bounds-checked *(delegated)* | Accepted | 2026-10-20 |
| [0436](0436-native-c-calls-by-dynamic-imports.md) | Native C calls use the platform convention and dynamic imports, planned before built *(delegated)* | Accepted | 2026-10-20 |
| [0438](0438-regex-engine-is-a-pike-vm.md) | `std.regex` is a linear-time Pike VM, without backreferences or lookaround *(delegated)* | Accepted | 2026-10-20 |
| [0439](0439-regex-syntax-is-the-re2-subset.md) | `std.regex` syntax is the RE2 subset; Unicode classes are a documented subset *(delegated)* | Accepted | 2026-10-20 |
| [0440](0440-regex-api-is-module-functions-and-values.md) | `std.regex` API: module functions over a `Regex` value; no `re"..."` literal yet *(delegated)* | Accepted | 2026-10-20 |
| [0441](0441-regex-iteration-replace-and-split-rules.md) | `find_all`, `replace` and `split` rules for empty matches *(delegated)* | Accepted | 2026-10-20 |
| [0442](0442-std-hash.md) | `std.hash`: digests over `str` now, `bytes` when it lands *(delegated)* | Accepted | 2026-10-20 |
| [0443](0443-std-compress-api.md) | `std.compress` API: `str` and `List[int]` now, one error type *(delegated)* | Accepted | 2026-10-20 |
| [0444](0444-hash64-choice.md) | `hash64` is a wyhash-style multiply-fold hash, written from scratch *(delegated)* | Accepted | 2026-10-20 |
| [0445](0445-deflate-gzip-in-house.md) | DEFLATE, gzip and CRC-32 are written in-house; measured throughput *(delegated)* | Accepted | 2026-10-20 |
| [0446](0446-zip-archives.md) | Zip archives: store and deflate, deterministic, no zip64, no escaping the target *(delegated)* | Accepted | 2026-10-20 |
| [0452](0452-match-guards.md) | Match guards: `pattern if cond => result` *(delegated)* | Accepted | 2026-10-20 |
| [0453](0453-range-patterns.md) | Range patterns: `lo..hi` and `lo..=hi` over integer and character literals *(delegated)* | Accepted | 2026-10-20 |
| [0454](0454-named-arguments.md) | Named arguments at call sites: `f(a, dy: 2, dx: 1)` *(delegated)* | Accepted | 2026-10-20 |
| [0455](0455-function-types-and-values.md) | Function types `fn(A) R`, and declared functions as values *(delegated)* | Accepted | 2026-10-20 |
| [0456](0456-calling-function-values.md) | Calling a function value, and its effects *(delegated)* | Accepted | 2026-10-20 |
| [0460](0460-drag-race-harness.md) | The drag race is one Python harness that times AIL, native AIL, C and Python from twins *(delegated)* | Accepted | 2026-10-20 |
| [0461](0461-module-fuzzers.md) | Module fuzzers are templates plus seeded input generators *(delegated)* | Accepted | 2026-10-20 |
| [0462](0462-native-differential.md) | Every pure-subset program is built and diffed against both interpreter tiers *(delegated)* | Accepted | 2026-10-20 |
| [0463](0463-rule-coverage.md) | Coverage is reported per spec rule, with per-file coverage from `ail test` *(delegated)* | Accepted | 2026-10-20 |
| [0464](0464-json-number-range.md) | A JSON number outside the float range is a parse error, and a non-finite number prints as null *(delegated)* | Accepted | 2026-10-20 |
| [0465](0465-bit-operations-as-int-methods.md) | Bit operations are `int` methods (`bit_and`, `shl`, ...), not operators (delegated) | Superseded by [1070](1070-bit-operators.md) | 2026-10-20 |
| [0468](0468-optional-none-locals-and-if-branches.md) | A `mut` local that starts as `none` takes its type from its first assignment; `if c then x else none` is `T?` *(delegated)* | Accepted | 2026-10-20 |
| [0469](0469-locals-may-shadow-functions.md) | A local may shadow a function; calling it in the local's block is `expr.call.1` *(delegated)* | Accepted | 2026-10-20 |
| [0470](0470-parenthesised-if-operand.md) | A one-line `if` is an operand in parentheses: `col + (if c then a else b)` *(delegated)* | Accepted | 2026-10-20 |
| [0471](0471-watch-on-the-bytecode-tier.md) | `--watch` runs on the bytecode tier: watched assignments and mutating calls hand off to the walker *(delegated)* | Superseded by 0873 | 2026-10-20 |
| [0477](0477-agent-corpus-round-three.md) | Agent corpus round three: ten programs on today's standard library, with Python twins (*delegated*) | Accepted | 2026-10-20 |
| [0483](0483-rule-id-audit.md) | A script audits spec rule IDs against the code; the `api.*` and `fix.*` rules are normative *(delegated)* | Accepted | 2026-10-20 |
| [0489](0489-editor-find-references.md) | Editor find references on Shift-F12 through `ail api --refs` (delegated) | Accepted | 2026-10-20 |
| [0490](0490-editor-regex-find.md) | Editor find bar: regex search on Alt-R (delegated) | Accepted | 2026-10-20 |
| [0492](0492-bench-automation.md) | One command produces the before/after benchmark table for tokens and speed (`tools/bench/run.sh`) *(delegated)* | Accepted | 2026-10-20 |
| [0498](0498-spec-examples-cite-rules.md) | Spec examples cite the rules they demonstrate; batch chapters must have zero uncovered rules *(delegated)* | Accepted | 2026-10-20 |
| [0499](0499-examples-run-on-both-tiers.md) | Every checked spec program runs under both interpreter tiers and must print the same *(delegated)* | Accepted | 2026-10-20 |
| [0500](0500-spec-example-kinds-for-tools-and-fixtures.md) | Spec examples for tools, fixtures and flags: `tool=`, `fixture=`, `flags=` *(delegated)* | Accepted | 2026-10-20 |
| [0501](0501-diagnostics-name-the-other-position.md) | Diagnostics point at the other position; the rule ID is the stable symbol *(delegated)* | Accepted | 2026-10-20 |
| [0504](0504-windows-drive-paths.md) | Windows drive paths in `std.path` and glob: a drive root is a root *(delegated)* | Accepted | 2026-10-20 |
| [0505](0505-windows-process-lookup.md) | Windows process lookup tries `PATHEXT` in `std.proc.run` *(delegated)* | Accepted | 2026-10-20 |
| [0506](0506-windows-status-checklist.md) | Windows is checked by one checklist the designer can run top to bottom *(delegated)* | Accepted | 2026-10-20 |
| [0507](0507-tensor-divide-and-activations.md) | Tensor `/` on floats, and the activations `relu`, `sigmoid`, `tanh` (AI, delegated) | Accepted | 2026-10-20 |
| [0508](0508-tensor-unary-minus.md) | Unary `-` on tensors is elementwise (AI, delegated) | Accepted | 2026-10-20 |
| [0509](0509-tensor-indexing-and-slicing.md) | Tensor `t[i]` drops the first axis, `t[lo..hi]` keeps it; `item()` (AI, delegated) | Accepted | 2026-10-20 |
| [0510](0510-licence-apache-2-with-runtime-exception.md) | Licence: Apache 2.0 with a Runtime Library Exception (designer) | Accepted | 2026-10-20 |
| [0511](0511-pdf-pages-read-on-demand.md) | The PDF reader reads pages on demand; search streams page by page (designer direction) | Accepted | 2026-10-20 |
| [0512](0512-pdf-ocr-by-nearest-shape.md) | OCR of scanned PDF pages by nearest reference shape, on demand; shapes from DejaVu and OFL fonts (designer direction) | Accepted | 2026-10-20 |
| [0513](0513-image-decoders-and-font-rasterizer-in-ail.md) | Image decoders (CCITT fax, JPEG) and a TrueType rasterizer written in AIL *(delegated)* | Accepted | 2026-10-20 |
| [0520](0520-pdf-reader-on-the-text-layer.md) | A PDF reader and viewer in AIL that searches the text layer, no OCR *(delegated)* | Accepted | 2026-10-20 |
| [0521](0521-tensor-conv2d.md) | `conv2d` on 2-D float tensors: valid, stride 1, no padding (AI, delegated) | Accepted | 2026-10-20 |
| [0522](0522-tensor-broadcasting.md) | Tensor broadcasting, NumPy-style, in `+ - * /` and `add`/`mul` (designer) | Accepted | 2026-10-20 |
| [0523](0523-tensor-argmax-clamp-round.md) | Tensor `argmax`, `argmin`, `clamp` and `round` (AI, delegated) | Accepted | 2026-10-20 |
| [0524](0524-tensor-operator-fusion.md) | Operator fusion for elementwise tensor chains, identical results (AI, delegated) | Accepted | 2026-10-20 |
| [0797](0797-tensor-f32-rounding-and-text.md) | `Tensor[f32]` rounding, scalars and text form (AI, delegated) | Accepted | 2026-10-21 |
| [0798](0798-tensor-f32-conversions.md) | `to_f32()` and `to_float()`; mixing and traces (AI, delegated) | Accepted | 2026-10-21 |
| [0813](0813-tensor-top2-margin.md) | Tensor `top2`: best, second-best and margin along an axis (AI, delegated) | Accepted | 2026-10-21 |
| [0532](0532-native-comprehensions-and-slices.md) | List comprehensions and slices in the native subset, and the MIR `trap` operation *(delegated)* | Accepted | 2026-10-20 |
| [0533](0533-native-stack-arguments.md) | Native calls pass integer arguments past the registers on the stack, as the C convention does *(delegated)* | Accepted | 2026-10-20 |
| [0534](0534-macos-os-layer-through-libsystem.md) | The macOS OS layer calls libSystem, not raw syscalls; Linux keeps raw syscalls *(delegated)* | Accepted | 2026-10-20 |
| [0535](0535-native-stack-check-on-recursive-calls.md) | Native recursion traps when the stack runs out: a check on calls that can recurse *(delegated)* | Accepted | 2026-10-20 |
| [0536](0536-native-generics-by-monomorphisation.md) | Generics and traits in native code by monomorphisation: instances named by their types, `impl` methods as methods of their type *(delegated)* | Accepted | 2026-10-20 |
| [0565](0565-native-string-runtime-calls.md) | A native `str` method is one runtime call, and its traps carry constant text *(delegated)* | Accepted | 2026-10-20 |
| [0566](0566-native-char-type.md) | `char` is a MIR type held in a register as an `int`, and `chars()` gives `List[char]` *(delegated)* | Accepted | 2026-10-20 |
| [0567](0567-native-text-of-numbers-and-unicode-tables.md) | Native text of numbers reuses the output routines, and character predicates use generated Unicode tables *(delegated)* | Accepted | 2026-10-20 |
| [0574](0574-native-optional-layout.md) | Native optionals: a pointer or 0, scalars boxed *(delegated)* | Accepted | 2026-10-20 |
| [0575](0575-native-results-error-slot.md) | Native results: the error slot and a return trace *(delegated)* | Accepted | 2026-10-20 |
| [0576](0576-native-error-values-messages.md) | Native error values: messages only, for now *(delegated)* | Accepted | 2026-10-20 |
| [0568](0568-native-map-and-set-layout.md) | Native `Map` and `Set`: one insertion-ordered table with a hash index *(delegated)* | Accepted | 2026-10-20 |
| [0569](0569-native-map-lowering.md) | Lowering `Map` and `Set` operations to runtime calls, with the interpreter's trap *(delegated)* | Accepted | 2026-10-20 |
| [0570](0570-native-constant-lists.md) | Constant lists are built once, on first use, in a global slot *(delegated)* | Superseded by 0604 | 2026-10-20 |
| [0571](0571-key-functions-inline.md) | Key functions are lowered inline, with no closure object *(delegated)* | Accepted | 2026-10-20 |
| [0572](0572-native-stable-sort.md) | Native stable sort over a list of indices *(delegated)* | Accepted | 2026-10-20 |
| [0573](0573-native-function-values.md) | Function values in native code: the address of a declared function *(delegated)* | Accepted | 2026-10-20 |
| [draft 0573a](0573a-native-function-values-deferred.md) | Function values in native code wait for the language *(delegated)* | Superseded by 0573 | 2026-10-20 |
| [0580](0580-os-calls-by-system-calls-with-a-status.md) | The OS layer is direct system calls, and a call that can fail reports a status *(delegated)* | Accepted | 2026-10-20 |
| [0581](0581-output-flush-and-entry-exit.md) | Output flushing at exit, traps, input and terminals; standard error through the same buffer *(delegated)* | Accepted | 2026-10-20 |
| [0582](0582-number-text-builder-and-format-specs.md) | Number text through a byte builder; format specifications and `to_float` in the runtime *(delegated)* | Accepted | 2026-10-20 |
| [0610](0610-self-host-bootstrap-harness.md) | A three-stage self-host bootstrap harness: fixed point plus the native tests *(delegated)* | Accepted | 2026-10-20 |
| [0612](0612-native-contracts.md) | Contracts in native code: `requires` at the call, `ensures` at each return, unchecked `.nc` copies for conditions *(delegated)* | Accepted | 2026-10-20 |
| [0613](0613-native-unsafe-and-attributes.md) | `unsafe` blocks, `@test` and `@deprecated` in native code *(delegated)* | Accepted | 2026-10-20 |
| [0614](0614-native-std-in-the-runtime.md) | `std.path`, `std.hash` and `std.random` as native runtime functions *(delegated)* | Accepted | 2026-10-20 |
| [0585](0585-self-compile-survey-tool.md) | The self-compile survey is a script that lowers each module alone *(delegated)* | Accepted | 2026-10-20 |
| [0586](0586-self-compile-blocker-ranking.md) | Self-compile blockers are ranked by problem count and owned by lane *(delegated)* | Accepted | 2026-10-20 |
| [0591](0591-native-float-round.md) | The native backend lowers `x.round()` to a runtime function (AI, delegated) | Accepted | 2026-10-20 |
| [0592](0592-std-math-float-methods.md) | Transcendental float methods (`sin`…`cbrt`), with a one-ulp bound (Designer) | Accepted | 2026-10-21 |
| [0597](0597-pe-executables-for-windows.md) | Windows x86-64 executables: PE32+ written by the backend, fixed base, no relocations *(delegated)* | Accepted | 2026-10-20 |
| [0598](0598-windows-imports-and-system-calls.md) | Windows imports: kernel32 through an import address table, Win64 convention at each call *(delegated)* | Accepted | 2026-10-20 |
| [0599](0599-native-threads-and-task-pool.md) | Native threads: `clone` and futex on Linux, one pool, replacement workers *(delegated)* | Accepted | 2026-10-20 |
| [0600](0600-native-atomics.md) | Native atomics: sequentially consistent cells, LSE on ARM64, `add` as a compare-and-swap loop *(delegated)* | Accepted | 2026-10-20 |
| [0601](0601-native-par-for.md) | Native `par for`: the body becomes a chunk function with exact accumulators *(delegated)* | Accepted | 2026-10-20 |
| [0602](0602-native-enum-layout.md) | Enums in native code: fieldless enums are `int`s, others kind-5 objects with the index in slot 0; fieldless variants are shared start-up objects; values immutable and shared *(delegated)* | Accepted | 2026-10-20 |
| [0603](0603-native-match.md) | `match` in native code: a compare chain on the variant index, bindings, guards, `|`, ranges, `int`/`bool`/`str` literals *(delegated)* | Accepted | 2026-10-20 |
| [0604](0604-native-computed-constants.md) | Computed module constants in native code: globals filled by `init.consts` before `main`, dependencies first *(delegated)* | Accepted | 2026-10-20 |
| [0702](0702-native-bytes.md) | Native `bytes`: string-object layout, literals decoded at the use, a `Byte` type that only `to_int` accepts *(delegated)* | Accepted | 2026-10-21 |
| [0619](0619-bytecode-calls-loops-and-fusion.md) | Bytecode tier: lean calls, in-place `for` variable, `while` tested after the body, fused `push`/`len` and `a[i][k]`, cheaper register writes *(delegated)* | Superseded by 0873 | 2026-10-21 |
| [0626](0626-std-csv.md) | `std.csv`: CSV as rows and records *(delegated)* | Accepted | 2026-10-21 |
| [0627](0627-std-args.md) | `std.args`: flags, options and positionals with help *(delegated)* | Accepted | 2026-10-21 |
| [0628](0628-std-log.md) | `std.log`: leveled messages on standard error *(delegated)* | Accepted | 2026-10-21 |
| [0629](0629-list-chunks-windows-counts-str-center.md) | List `chunks`, `windows`, `counts`, `flatten`; string `center`, `words`, `trim_start`, `trim_end`, `count`, `rfind` *(delegated)* | Accepted | 2026-10-21 |
| [0724](0724-quick-reference-trim.md) | Quick reference trimmed by 15% (15,176 to 12,891 tokens) with every construct kept *(delegated)* | Accepted | 2026-10-21 |
| [0711](0711-tool-driving-agent-programs.md) | Agent programs that drive AIL's own tools: the twin calls the same tool, output is byte-identical, fixtures are committed (round 5) *(delegated)* | Accepted | 2026-10-21 |
| [0732](0732-counts-and-json-reads-with-defaults.md) | `m.top(n)`, `j.text(key, default)`, `j.flag(key, default)` and `parse_json_lines(text)` *(delegated)* | Accepted | 2026-10-21 |
| [0733](0733-process-result-methods.md) | `proc.Output` has `text()` (stdout then stderr) and `ok()` *(delegated)* | Accepted | 2026-10-21 |
| [0667](0667-regex-group-all.md) | `regex.group_all`: the text of one group in every match, `""` when unset *(delegated)* | Accepted | 2026-10-21 |
| [0673](0673-definitional-rule-marker.md) | Spec rules that only describe the language carry a `[definitional]` marker and need no citation *(delegated)* | Accepted | 2026-10-21 |
| [0679](0679-native-test-runner.md) | `ail test --native`: build each `@test` function natively and compare it with the interpreter *(delegated)* | Accepted | 2026-10-21 |
| [0675](0675-round-4-syntax-candidates.md) | Round-4 syntax candidates: no `for i, x`, no inferred `main !`; `par` over chunks already works *(delegated)* | Accepted | 2026-10-21 |
| [0698](0698-card-spec-audit-for-std.md) | A std audit: every std function has a spec mention, checked by `tools/rules-audit.sh` *(delegated)* | Accepted | 2026-10-21 |
| [0631](0631-json-whole-numbers-print-without-fraction.md) | `Json.to_text` prints whole numbers without a fraction *(delegated)* | Accepted | 2026-10-21 |
| [0637](0637-http-persistent-connections-and-timeouts.md) | HTTP/1.1 client: a connection pool (keep-alive), chunked request bodies, `connect_ms` and `total_ms` timeouts *(delegated)* | Accepted | 2026-10-21 |
| [0638](0638-cookie-jar.md) | A cookie jar in the standard library, `std.net.jar`: RFC 6265 domain, path, expiry, `Secure`, `HttpOnly`; the one jar for Garlic *(delegated)* | Accepted | 2026-10-21 |
| [0639](0639-http-server-keep-alive.md) | HTTP server keeps connections alive between requests *(delegated)* | Accepted | 2026-10-21 |
| [0640](0640-samesite-and-graceful-shutdown.md) | SameSite in the cookie jar; `srv.shutdown()` stops a server without dropping requests under way *(delegated)* | Accepted | 2026-10-21 |
| [0655](0655-build-cache.md) | A content-addressed cache of finished `ail build` executables, and derive expansion with one effect inference *(delegated)* | Accepted | 2026-10-21 |
| [0656](0656-compiler-hot-paths.md) | Cold-build hot paths in the AIL compiler: a per-function profile (`AIL_BYTECODE_STATS=3`), dominators by immediate dominator, the runtime parsed once, cheaper lexer and lookahead *(delegated)* | Accepted | 2026-10-21 |
| [0658](0658-native-json.md) | `parse_json`, `Json.to_text`, `json_quote` run natively *(delegated)* | Accepted | 2026-10-21 |
| [0693](0693-test-captures-stderr.md) | `ail test` captures a test's standard error, shown under a failing test's `FAIL` line marked `!` *(delegated)* | Accepted | 2026-10-21 |
| [0716](0716-streaming-response-bodies.md) | Streaming response bodies: `net.open` returns a `net.Body` read with `read_bytes`, and a body read to its end returns its connection to the pool *(delegated)* | Accepted | 2026-10-21 |
| [0695](0695-fix-bitwise-not.md) | `ail fix` for `~name`: the bitwise-not method *(delegated)* | Accepted | 2026-10-21 |
| [0700](0700-native-exit-sleep-range-traps.md) | Native `io.exit` and `time.sleep_ms` range traps *(delegated)* | Accepted | 2026-10-21 |
| [0701](0701-native-error-context.md) | Native `?` with a context message: contexts ride at the head of the trace list *(delegated)* | Accepted | 2026-10-21 |
| [0703](0703-card-middle-trim-level.md) | `ail api --card` gets a middle trimming level (signatures and first sentences) and picks the richest level within budget; amends 0311 *(delegated)* | Accepted | 2026-10-21 |
| [0717](0717-transparent-content-encoding.md) | `std.net` sends `Accept-Encoding: gzip, deflate` and decodes the response unless the program sets the header itself *(delegated)* | Accepted | 2026-10-21 |
| [0719](0719-per-module-cards.md) | `ail api --card --module NAME` prints one module's card (api.card.6) *(delegated)* | Accepted | 2026-10-21 |
| [0720](0720-package-index-card.md) | `ail api --card --index` lists a package's modules (api.card.7) *(delegated)* | Accepted | 2026-10-21 |
| [0630](0630-is-empty-and-path-stem.md) | `is_empty` on strings and collections, and `path.stem` *(delegated)* | Accepted | 2026-10-21 |
| [0690](0690-fuzz-csv-args.md) | Fuzzers for std.csv and std.args; `csv.write` quotes a row of one empty field *(delegated)* | Accepted | 2026-10-21 |
| [0691](0691-fuzz-group-all-native-bytes.md) | Fuzzers for `regex.group_all` and native `bytes` literals *(delegated)* | Accepted | 2026-10-21 |
| [0722](0722-native-subset-help-lines.md) | `native.subset.1` problems print a `help:` line, keyed in the printer (no lower.ail edit) *(delegated)* | Accepted | 2026-10-21 |
| [0741](0741-ref-arguments-to-spawn.md) | A `ref` is not passed to a spawned task (own.ref.2) *(delegated)* | Accepted | 2026-10-21 |
| [0743](0743-operator-traits-and-constant-types.md) | `trait.op.2`, `ct.const.4` and `err.type.3` get their own diagnostics *(delegated)* | Accepted | 2026-10-21 |
| [0729](0729-api-diff-command.md) | `ail api --diff`: the public API change between two versions *(delegated)* | Accepted | 2026-10-21 |
| [0730](0730-api-diff-breaking-rules.md) | What `ail api --diff` marks as breaking *(delegated)* | Accepted | 2026-10-21 |
| [0731](0731-api-diff-output-and-budget.md) | Output of `ail api --diff` and its token budget *(delegated)* | Accepted | 2026-10-21 |
| [0734](0734-native-integer-text.md) | Native integer text: digits written from the end, one division by 10 each *(delegated)* | Accepted | 2026-10-21 |
| [0725](0725-test-cases-audit.md) | Test-case audit: one audit script, a duplicate removed, one-theme cases split *(delegated)* | Accepted | 2026-10-21 |
| [0748](0748-list-from-json-as-an-associated-call.md) | `List[T].from_json(j)`: associated calls on builtin generic types *(delegated)* | Accepted | 2026-10-21 |
| [0742](0742-compile-time-step-limit.md) | A statement-count limit on computing constants (ct.eval.3) *(delegated)* | Accepted | 2026-10-21 |
| [0746](0746-native-subset-constructs-table.md) | One table of native-subset constructs, with owners *(delegated)* | Accepted | 2026-10-21 |
| [0692](0692-fuzz-http-and-socket-templates.md) | HTTP fuzzers, `fuzz-net` templates that actually run, and the three-digit status fix *(delegated)* | Accepted | 2026-10-21 |
| [0745](0745-branch-free-if-measured-not-enabled.md) | Branch-free `if` with deferred overflow checks: built and measured, not enabled *(delegated)* | Accepted | 2026-10-21 |
| [0754](0754-stop-a-server-from-its-handler-and-poll-faster.md) | A handler stops its server with `req.stop()`; network waits poll faster *(delegated)* | Accepted | 2026-10-21 |
| [0755](0755-http-serve-dir.md) | `http.serve_dir` serves a directory *(delegated)* | Accepted | 2026-10-21 |
| [0751](0751-fs-copy-copies-a-directory-tree.md) | `fs.copy` copies a directory tree *(delegated)* | Accepted | 2026-10-21 |
| [0749](0749-regex-methods.md) | Methods on `regex.Regex` *(delegated)* | Accepted | 2026-10-21 |
| [0753](0753-checker-differential-fuzz.md) | Differential fuzzing of the self-hosted checker against `ail check`: generator, driver, 19,800 programs compared, 0 disagreements *(delegated)* | Accepted | 2026-10-21 |
| [0726](0726-native-memory-limit.md) | A memory cap for native programs, checked when the heap takes a chunk *(delegated)* | Accepted | 2026-10-21 |
| [0727](0727-native-timeout.md) | A time limit for native programs, polled at the allocator *(delegated)* | Accepted | 2026-10-21 |
| [0728](0728-native-limit-settings.md) | Where a native program gets its limits *(delegated)* | Accepted | 2026-10-21 |
| [0738](0738-map-lookup-without-key-conversion.md) | Text and integer map keys are looked up without building a `Key` *(delegated)* | Accepted | 2026-10-21 |
| [0737](0737-runtime-small-block-allocator.md) | `ail run` and `ail test` allocate small blocks from per-thread free lists *(delegated)* | Accepted | 2026-10-21 |
| [0739](0739-json-parse-builds-text-once.md) | The native JSON parser builds each text value once *(delegated)* | Accepted | 2026-10-21 |
| [0620](0620-bytecode-element-method-calls.md) | Bytecode tier: `mut self` method calls on list elements, field `op=` on `self`, nested `else` *(delegated)* | Superseded by 0873 | 2026-10-21 |
| [0621](0621-bytecode-bounds-checks-stay.md) | Bytecode tier: bounds checks stay (no safe way to drop them); what is left is the tier's design *(delegated)* | Superseded by 0873 | 2026-10-21 |
| [0759](0759-bytecode-tier-stalls-not-frames.md) | Bytecode tier: the cost left is stalls, not the frame layout; two frame redesigns measured and dropped *(delegated)* | Superseded by 0873 | 2026-10-21 |
| [0760](0760-packed-list-storage.md) | Lists of `int`, `float` and `bool` are stored packed (`Value::List(Rc<List>)`), invisible to programs; sieve 0.59 s to 0.21 s *(delegated)* | Accepted | 2026-10-21 |
| [0756](0756-bytecode-differential-fuzzing.md) | Differential fuzzing of the bytecode tier: a loop, struct and `mut self` generator, byte-identical output and traps, hand-off counts *(delegated)* | Superseded by 0873 | 2026-10-21 |
| [0757](0757-fuzz-math-special-values.md) | A fuzzer for `std.math` special values; the fuzz budget stays in the harness *(delegated)* | Accepted | 2026-10-21 |
| [0605](0605-native-list-methods.md) | Native list methods: `extend`, `remove_at`, `insert`, `pop`, `contains`, `sort`, `sorted` as runtime calls *(delegated)* | Accepted | 2026-10-20 |
| [0802](0802-native-list-queries.md) | Native `sum`, `min`, `max`, `first`, `last`, `index_of`, `swap` on lists and `Set.sorted` *(delegated)* | Accepted | 2026-10-21 |
| [0803](0803-native-map-top-and-set-algebra.md) | Native `Map.top`, set algebra (`union`, `intersect`, `difference`) and set and map comprehensions *(delegated)* | Accepted | 2026-10-21 |
| [0818](0818-native-struct-and-enum-map-values.md) | Native maps with struct and enum values; copies of tables of structs are independent *(delegated)* | Accepted | 2026-10-21 |
| [0735](0735-module-list-constants-are-globals.md) | Module-level list constants get a program global instead of being rebuilt at every use: native wordfreq 0.277 s to 0.037 s *(delegated)* | Accepted | 2026-10-21 |
| [0758](0758-str-split-limit.md) | `s.split(sep, limit)` makes at most `limit` pieces *(delegated)* | Accepted | 2026-10-21 |
| [0772](0772-native-std-modules-in-ail.md) | Standard modules written in AIL compile natively as modules of the program, read from `bootstrap/src/std`; `std.log` builds *(delegated)* | Accepted | 2026-10-21 |
| [0766](0766-native-string-hash-by-words.md) | Native string map keys are hashed eight bytes at a time: `map_hash` 64 to 44 instructions per word *(delegated)* | Accepted | 2026-10-21 |
| [0767](0767-map-first-probe.md) | The native map lookup tries one probe without saved registers, then falls back to the full probe *(delegated)* | Accepted | 2026-10-21 |
| [0770](0770-native-proc-run.md) | Native `proc.run` and `proc.which` on Linux: `clone`, `execve`, pipes and `ppoll`, optional trailing parameters, the `proc.Output` struct *(delegated)* | Accepted | 2026-10-21 |
| [0763](0763-comprehensions-compile-to-loops.md) | Comprehensions compile to loops in the bytecode tier: tensor_ops_loops 0.380 s to 0.214 s *(delegated)* | Accepted | 2026-10-21 |
| [0786](0786-syntax-diagnostic-text-and-fmt-check.md) | The self-hosted front end prints diagnostic help and fixes; `fmt --check` fails on broken sources *(delegated)* | Accepted | 2026-10-21 |
| [0771](0771-native-proc-output-methods.md) | Native `Output.ok()` and `Output.text()`: two inline lowerings on the registered `proc.Output` struct *(delegated)* | Accepted | 2026-10-21 |
| [0791](0791-tuples.md) | Tuples: anonymous pairs and triples as values (designer, resolves OQ-124) | Accepted | 2026-10-21 |
| [0792](0792-tuple-syntax.md) | Tuple syntax: `(A, B)` types, `t.0` elements, `a, b := t`, `for k, v in m` *(delegated)* | Accepted | 2026-10-21 |
| [0793](0793-tuple-patterns.md) | Tuple patterns in `match`; a bare name binds *(delegated)* | Accepted | 2026-10-21 |
| [0794](0794-tuple-library-names.md) | `xs.enumerate()` and `zip(a, b)`; no `m.items()` or `m.pairs()` *(delegated)* | Accepted | 2026-10-21 |
| [0795](0795-tuple-text-and-json.md) | Tuples as text and JSON: `(1, "a")` and arrays *(delegated)* | Accepted | 2026-10-21 |
| [0796](0796-tensor-f32.md) | `Tensor[f32]`: a 32-bit float element type (designer) | Accepted | 2026-10-21 |
| [0799](0799-one-path-spelling-in-tool-output.md) | One path spelling in every tool output: `/` in diagnostics, JSON and `ail api` *(delegated)* | Accepted | 2026-10-21 |
| [0780](0780-fix-failure-in-operand-and-json-num-hint.md) | `ail fix` for a failure inside an operand; `expr.block.2` hint for `Json.num` in an `int` function *(delegated)* | Accepted | 2026-10-21 |
| [0781](0781-stream-output-before-blocking.md) | A task blocking on a child's stream while output arrives is not a `conc.task.7` deadlock *(delegated)* | Accepted | 2026-10-21 |
| [0764](0764-comprehension-if-and-text-len.md) | `if` elements in comprehensions and `len` of text, maps, sets and bytes compile in the tier: -43% on a micro-benchmark *(delegated)* | Accepted | 2026-10-21 |
| [0776](0776-fieldinfo-doc.md) | `FieldInfo.doc` carries a field's `##` comment (integrator choice) | Accepted | 2026-10-21 |
| [0782](0782-fix-text-plus-chains.md) | `ail fix` joins text with interpolation across a `+` chain and around calls with strings *(delegated)* | Accepted | 2026-10-21 |
| [0768](0768-native-string-scans-by-words.md) | Native strings: character counts and separator search a word at a time, interpolation joined in one allocation; `lines_split` -27% instructions *(delegated)* | Accepted | 2026-10-21 |
| [0837](0837-interpolation-with-number-holes-uses-the-builder.md) | Interpolation with a number, `char` or `bool` hole is written into the builder and allocated once: `map_ops` -19%, `lines_split` -11% *(delegated)* | Accepted | 2026-10-21 |
| [0769](0769-map-hash-finalizer.md) | Native string hash: a second multiply as the finalizer, so keys that differ in their high bytes spread (50 to 1.75 probes) *(delegated)* | Accepted | 2026-10-21 |
| [0814](0814-selftest-speed.md) | Selftest speed: native builds on four threads, then parallel test binaries, then one compiler load for many builds (AI, delegated ADR 0076, integrator-approved) | Accepted | 2026-10-21 |
| [0809](0809-fix-contract-keywords.md) | `ail fix` moves `requires result` to `ensures` and unwraps `old()` in `requires` *(delegated)* | Accepted | 2026-10-21 |
| [0817](0817-token-budgets-for-agent-pages.md) | Token budgets for the language card (8,000) and quick reference (12,518) on the five-tokenizer mean; quick-reference additions are paid for by a trim in the same PR; amends 0276 in part | Accepted | 2026-10-21 |
| [0804](0804-native-error-values-are-structs-and-enums.md) | Native error values are ordinary structs and enums *(delegated)* | Accepted | 2026-10-21 |
| [0805](0805-native-error-message-method.md) | The message of an error type is a method *(delegated)* | Accepted | 2026-10-21 |
| [0806](0806-native-typed-else-err.md) | Typed error binding in native `else err` *(delegated)* | Accepted | 2026-10-21 |
| [0815](0815-native-tensors.md) | Native tensors: a runtime object and kernels written in AIL *(delegated)* | Accepted | 2026-10-21 |
| [0783](0783-std-error-families.md) | Std error families: a closed kind list per module, as spec rules | Accepted | 2026-10-21 |
| [0784](0784-std-error-compress-net.md) | Std error families: compress and net client causes | Accepted | 2026-10-21 |
| [0785](0785-native-subset-help-runs-with-ail-run.md) | Every `native.subset.1` help line says `ail run` runs the program | Accepted | 2026-10-21 |
| [0774](0774-load-json-and-print-json.md) | `load_json[T](path)` and `print_json(v)` *(delegated)* | Accepted | 2026-10-21 |
| [0775](0775-bytes-hex-separator.md) | `b.hex(sep)` spaces the byte pairs *(delegated)* | Accepted | 2026-10-21 |
| [0832](0832-native-str-char-cmp-and-char-index-of.md) | Native `.cmp()` on `str` and `char`, `.index_of()` on `List[char]` *(delegated)* | Accepted | 2026-10-21 |
| [0820](0820-driver-level-unsafe.md) | `unsafe driver` manifest level unlocks hardware access through `std.hw`; 64-bit targets only | Accepted | 2026-10-21 |
| [0823](0823-lower-ail-feature-files.md) | `lower.ail` is a core plus feature files with one-line hooks: a native feature lives in its own `lower_*.ail` and adds at most one dispatch line and one state field *(delegated)* | Accepted | 2026-10-21 |
| [0834](0834-native-sized-numbers.md) | Native sized integers are `Ty.Sized(kind)`, held sign- or zero-extended in a register, with trapping conversions *(delegated)* | Accepted | 2026-10-21 |
| [0835](0835-native-raw-pointers-and-hw.md) | Native `*T` is an address in an integer register; `std.hw` lowers to plain loads, stores and fences *(delegated)* | Accepted | 2026-10-21 |
| [0844](0844-native-libc-subset.md) | Native `std.mem` over runtime page blocks and a fixed libc subset for `extern c fn`; other symbols stay `ffi.native.1` *(delegated)* | Accepted | 2026-10-22 |
| [0810](0810-fix-method-placement.md) | `ail fix` moves methods out of `impl` blocks and structs *(delegated)* | Accepted | 2026-10-21 |
| [0765](0765-plain-return-skips-unwind.md) | A `return` in a plain compiled body returns its value directly, without unwinding: binarytrees -1.7% *(delegated)* | Accepted | 2026-10-21 |
| [0807](0807-native-build-passes-derived-methods.md) | `ail build` passes derived methods to the native backend as `gen:impl` sources; `Self` in method signatures *(delegated)* | Accepted | 2026-10-21 |
| [0808](0808-native-prelude-json-lowered-on-demand.md) | Native builds get the prelude's JSON and `Default` modules, lowered only where the program reaches them *(delegated)* | Accepted | 2026-10-21 |
| [0840](0840-native-std-tz-in-the-runtime.md) | Native `std.tz`: a TZif reader and POSIX rules in the runtime, so `std.time.date` builds natively *(delegated)* | Accepted | 2026-10-21 |
| [0830](0830-native-raw-tcp.md) | Native raw TCP: sockets by system calls, handles in a table, `net.NetError` *(delegated)* | Accepted | 2026-10-21 |
| [0821](0821-unsafe-driver-manifest-and-flag.md) | `unsafe driver` in `ail.pkg`, and `--unsafe driver` for a program without one *(delegated)* | Accepted | 2026-10-21 |
| [0822](0822-std-hw-volatile-and-hw-effect.md) | `std.hw`: `at`, volatile methods and fences, and the effect `hw` *(delegated)* | Accepted | 2026-10-21 |
| [0819](0819-native-std-regex-in-ail.md) | Native `std.regex` is an AIL module compiled with the program *(delegated)* | Accepted | 2026-10-21 |
| [0811](0811-fix-for-over-recv.md) | `ail fix` turns `for v in ch.recv()` into `for v in ch` *(delegated)* | Accepted | 2026-10-21 |
| [0827](0827-tensor-softmax.md) | `softmax()` along the last axis (AI, delegated) | Accepted | 2026-10-21 |
| [0831](0831-tensor-from-fn.md) | `tensor.from_fn(shape, \|i\| e)` (AI, delegated) | Accepted | 2026-10-21 |
| [0816](0816-native-codegen-entry-addressing-constants.md) | Native code generation: parameters stay in their arrival registers, scaled addressing for load and store primitives, held constants *(delegated)* | Accepted | 2026-10-21 |
| [0777](0777-args-from-struct.md) | `Opts.parse(...)`: command-line options derived from a struct *(delegated)* | Accepted | 2026-10-21 |
| [0812](0812-fix-ternary.md) | `ail fix` writes `c ? a : b` as `if c then a else b` *(delegated)* | Accepted | 2026-10-21 |
| [0836](0836-native-tls.md) | Native TLS: the in-house client ported to AIL in slices, starting with native `std.hash` *(delegated)* | Accepted | 2026-10-21 |
| [0824](0824-native-loop-list-reuse.md) | Native: reuse lists that do not outlive a loop iteration *(delegated)* | Accepted | 2026-10-21 |
| [0828](0828-native-compress.md) | Native `std.compress`: the Rust codec ported to the runtime, byte for byte *(delegated)* | Accepted | 2026-10-21 |
| [0841](0841-fix-single-case-mistakes.md) | `ail fix` for fn-type `->`, `reshape(2, 3)`, `e.message()` and `\u{..}` in bytes *(delegated)* | Accepted | 2026-10-21 |
| [0829](0829-native-tuples.md) | Native tuples: a kind-4 struct object with one slot per element; `eq.T`, `lt.T` and `copy.T` generated; table keys stay out | Accepted | 2026-10-21 |
| [0838](0838-std-app-native-windows.md) | `std.app`: native windows from a view tree, drawn by a generic host (SwiftUI on macOS) over JSON lines; capability `app` | Accepted | 2026-10-21 |
| [0825](0825-native-ascii-join-chars.md) | Native: an ASCII path for joining characters *(delegated)* | Accepted | 2026-10-21 |
| [0606](0606-win64-calls-to-system-dlls.md) | Win64 calls to system DLL functions: `libsys.NAME` through the PE import table *(delegated)* | Accepted | 2026-10-20 |
| [0607](0607-windows-runtime-guard.md) | `ail build --target x86_64-windows` is refused until the Windows runtime lands *(delegated)* | Accepted | 2026-10-20 |
| [0608](0608-windows-target-and-runtime.md) | The Windows x86-64 target: `target()` is 3, the entry runs the runtime, and output, exit and the heap use kernel32 *(delegated)* | Accepted | 2026-10-20 |
| [0616](0616-native-any-trait-dispatch.md) | Native `any Trait`: a tagged box and a generated dispatcher per method *(delegated)* | Accepted | 2026-10-20 |
| [0826](0826-native-reset-in-place.md) | Native: reset a list in place *(delegated)* | Accepted | 2026-10-21 |
| [0641](0641-native-trap-stack-from-frame-pointers.md) | A native trap prints its stack by walking frame pointers over a call table in the code *(delegated)* | Accepted | 2026-10-21 |
| [0642](0642-native-trap-stack-format.md) | The native stack lines name the function and the call position, and leave out values *(delegated)* | Accepted | 2026-10-21 |
| [0643](0643-inlined-callees-keep-their-frame.md) | A check inlined from a callee keeps the callee's frame and the call that was inlined *(delegated)* | Accepted | 2026-10-21 |
| [0644](0644-native-trap-stack-always-on.md) | The trap stack is always built, with no flag: cost measured *(delegated)* | Accepted | 2026-10-21 |
| [0845](0845-vectorization-order-and-idioms.md) | Vectorization order: idioms first, a runtime fill and copy on both ISAs, `break` loops under a bit-identical rule *(delegated)* | Accepted | 2026-10-21 |
| [0842](0842-mac-first-gate.md) | Gate on the designer's Mac with `tools/gate/mac-gate.sh`; no Linux VM *(designer)* | Accepted | 2026-10-22 |
| [0846](0846-native-proc-start-and-macos-proc.md) | Native `proc.start` with streams fed by a poll through the channels' `source` word; `std.proc` on macOS by libSystem `fork` *(delegated)* | Accepted | 2026-10-21 |
| [0855](0855-retire-the-rust-bootstrap.md) | Retire the Rust bootstrap: `ail` is a native AIL program, seeded from private release binaries with a committed SHA-256 *(designer; detail delegated)* | Accepted | 2026-10-22 |
| [0852](0852-native-tls-one-suite-first.md) | Native TLS 1.3 offers ChaCha20-Poly1305 first and lands in checked slices; verification ships with the handshake *(delegated)* | Accepted | 2026-10-22 |
| [0858](0858-native-assert-lowering.md) | `assert` and `assert_eq` lower natively; `@test` stays a rewrite *(delegated)* | Accepted | 2026-10-22 |
| [0864](0864-native-recursive-default-text.md) | A recursive type's default text is a generated function, not an unbounded inline expansion *(delegated)* | Accepted | 2026-10-23 |
| [0848](0848-idle-values-leave-registers-in-loops.md) | Values idle in a loop leave their registers; a checked `mul` result avoids its operands' registers *(delegated)* | Accepted | 2026-10-22 |
| [0850](0850-native-compiler-runs-builtin-derive-generators.md) | The native compiler runs the prelude's derive generators itself, so it needs no `AIL_DERIVED` file *(delegated)* | Accepted | 2026-10-22 |
| [0854](0854-std-app-bitmap-op.md) | `std.app` `Bitmap` op: RGBA pixels in the display list, kept by the host under an id; Garlic sends decoded images once *(delegated)* | Accepted | 2026-10-22 |
| [0839](0839-std-app-text-measure.md) | `std.app` text measure: the host reports advance widths for the app's runs, Garlic lays out with them *(delegated)* | Accepted | 2026-10-22 |
| [0856](0856-native-compiler-runs-user-derive-generators.md) | The native compiler runs user `@derive` generators as a native program, so it needs no interpreter *(delegated)* | Accepted | 2026-10-22 |
| [0861](0861-std-app-inbox.md) | `std.app` inbox: tasks post `external` events into the running app, woken by a channel, not a tick *(delegated)* | Accepted | 2026-10-23 |
| [0859](0859-ail-edit-in-ail.md) | `ail edit` is ported to AIL (`compiler/edit/`), byte-identical to the Rust tool; syntax-only check until the loader lands *(delegated)* | Accepted | 2026-10-23 |
| [0860](0860-ail-fix-machinery-in-ail.md) | The machinery of `ail fix` (edit types, anchor resolution, driver) is ported to AIL (`compiler/fix/`), same fixes and output as the Rust tool *(delegated)* | Accepted | 2026-10-23 |
| [0862](0862-native-mut-cells-and-for-mut-write-through.md) | Native `mut` parameters and `mut self` of value types use cells; `for mut` over scalars writes through *(delegated)* | Accepted | 2026-10-23 |
| [0863](0863-native-std-ui-through-the-app-host.md) | Native `std.ui` and widgets render through the `std.app` host: one window path for edit, pdf and Garlic; Mac first | Accepted | 2026-10-23 |
| [0887](0887-std-app-host-for-linux.md) | `std.app` host for Linux: a GTK 3 program linked dynamically from the system | Accepted | 2026-10-24 |
| [0865](0865-no-double-work.md) | One implementation (native only), one checker (AIL), one Mac gate per batch, generated files once: no double work | Accepted | 2026-10-23 |
| [0868](0868-ai-first-tooling-after-rust-removal.md) | AI-first tooling after Rust removal: `ail ask`, `uhwhatnow?` ranked typed holes, edit impact, native time travel, tool-checked contracts, token-priced diagnostics, native sandbox, live attach; zero cost when unused | Accepted | 2026-10-23 |
| [0869](0869-http-proxy-from-the-environment.md) | HTTP proxies from the environment in the native `std.net` client: `HTTP_PROXY`, `HTTPS_PROXY` through `CONNECT`, `NO_PROXY` | Accepted | 2026-10-23 |
| [0871](0871-native-instrumentation-layer.md) | One native instrumentation layer for coverage and call traces (`AIL_INSTRUMENT`) | Accepted | 2026-10-24 |
| [0872](0872-uhwhatnow-ranked-holes.md) | `uhwhatnow?`, the hole that asks: locals, fields, then calls that fit, best first, eight at most, a fix each | Accepted | 2026-10-24 |
| [0870](0870-native-extern-c-bound-lazily-with-dlopen.md) | A native `extern c fn` outside the libc subset binds lazily with `dlopen` and `dlsym`; libc-only programs stay static | Accepted | 2026-10-24 |
| [0866](0866-native-composite-map-keys.md) | Native maps and sets take struct, tuple, enum and list keys, hashed by the key's shape *(delegated)* | Accepted | 2026-10-23 |
| [0867](0867-native-unique-values-and-drop.md) | Native unique values and `Drop`: ownership flags on the defer machinery *(delegated)* | Accepted | 2026-10-23 |
| [0873](0873-rust-only-features-after-rust-removal.md) | Rust-only features: native record/replay and deterministic scheduler; the bytecode tier goes | Accepted | 2026-10-24 |
| [0874](0874-purity-driven-speed-optimizations.md) | Purity-driven speed optimizations: memoization by cost model, pure-call CSE and hoisting, merging only for speed | Accepted | 2026-10-24 |
| [0875](0875-packed-f32-tensors.md) | Native `Tensor[f32]` stores 4-byte floats (designer) | Accepted | 2026-10-24 |
| [0876](0876-tensor-simd-and-multicore.md) | Tensor kernels use SIMD, then all cores, automatically; deterministic runs stay bit-identical | Accepted | 2026-10-24 |
| [0884](0884-observe-field-and-attach-protocol.md) | `@observe` marks a struct field; attach speaks JSON lines on loopback with a 0600 token file | Accepted | 2026-10-24 |
| [0877](0877-no-downcast-of-any-trait.md) | No downcasting of `any Trait`: an enum is the checked way to hold one of several known types (resolves OQ-094) | Accepted | 2026-10-24 |
| [0880](0880-budgeted-diagnostic-forms.md) | Token-priced diagnostics: one-line and expanded forms under `--budget` (ADR 0868 item 6) | Accepted | 2026-10-24 |
| [0888](0888-std-app-windows-host.md) | `std.app` host for Windows: WinForms and GDI+ in C#, built with the framework csc | Accepted | 2026-10-24 |
| [0883](0883-native-max-steps.md) | Native `--max-steps` on the instrumentation layer | Accepted | 2026-10-24 |
| [0886](0886-native-values-mode.md) | Native `ail run --values`: frame values in a trap | Accepted | 2026-10-24 |
| [0878](0878-screen-model-for-std-app.md) | The screen model: what every std.app program shows, for agents (display list first, pixels for canvas, budgeted queries) | Accepted | 2026-10-24 |
| [0879](0879-observe-and-live-attach.md) | `@observe` and live attach: read-only program state for agents; on in `ail run`, opt-in for `ail build` | Accepted | 2026-10-24 |
| [0891](0891-ail-context-budgeted-pack.md) | `ail context SYM --budget N`: the smallest useful context pack (designer chose; details delegated) | Accepted (built by [1020](1020-ail-context.md)) | 2026-10-24 |
| [0892](0892-tool-functions-mcp-server.md) | `@tool` functions served by `ail serve --mcp` (first slice built by [0986](0986-app-tools-and-attach-mcp.md)) | Accepted (first slice built by [0986](0986-app-tools-and-attach-mcp.md))  | 2026-10-24 |
| [0893](0893-ail-try.md) | `ail try`: check and test a patch in memory | Accepted | 2026-10-24 |
| [0894](0894-effect-changes-in-ail-diff.md) | Effect changes in `ail diff` | Accepted (built by [1024](1024-ail-review.md)) | 2026-10-24 |
| [0895](0895-ail-infer.md) | `ail infer`: suggested `requires` and `ensures` | Accepted (built by [1023](1023-ail-infer-contracts.md)) | 2026-10-24 |
| [0896](0896-ail-test-mutate.md) | `ail test --mutate`: which tests never notice a break | Superseded by 1022 | 2026-10-24 |
| [0897](0897-ail-check-perf.md) | `ail check --perf`: speed hints that share the optimizer's facts | Deferred past 0.1.0 | 2026-10-24 |
| [0898](0898-ail-database-engine.md) | A database engine and its shell, as a showcase program | Deferred past 0.1.0 | 2026-10-24 |
| [0899](0899-ail-media-and-image-editor.md) | A media player or image editor, as a showcase program | Deferred past 0.1.0 | 2026-10-24 |
| [0900](0900-refuse-rust-only-routes.md) | `./ail` refuses `test --native` and `selftest --test` (Rust-only routes) | Accepted | 2026-10-24 |
| [0901](0901-std-toml.md) | `std.toml`: TOML read into the `Json` value *(delegated)* | Accepted | 2026-10-25 |
| [0902](0902-std-bigint.md) | `std.bigint`: integers of any size *(delegated)* | Accepted | 2026-10-25 |
| [0912](0912-list-u16-u32.md) | Native `List[u16]` and `List[u32]` hold two and four bytes per element *(delegated)* | Accepted | 2026-10-25 |
| [0907](0907-list-u8-one-byte.md) | Native `List[u8]` holds one byte per element *(delegated)* | Accepted | 2026-10-25 |
| [0905](0905-std-app-serif-font.md) | `std.app` text family `serif` and Garlic's font-family mapping | Accepted | 2026-10-25 |
| [0906](0906-retire-native-coverage-tracker.md) | The native coverage tracker is retired; its report stays as a snapshot *(delegated)* | Accepted | 2026-10-25 |
| [0904](0904-rust-bootstrap-deleted.md) | The Rust bootstrap is deleted (D5) | Accepted | 2026-10-25 |
| [0903](0903-ail-new.md) | `ail new NAME`: a project scaffold *(delegated)* | Accepted | 2026-10-25 |
| [0881](0881-ail-ask.md) | `ail ask`: semantic questions (who writes a field, what calls a function, which effects reach it) answered from the checker's graph | Accepted | 2026-10-24 |
| [0882](0882-edit-impact.md) | Edit impact: which tests and effects an edit touches (`ail edit --impact`, `ail ask --affected`, `ail test --affected`) | Accepted | 2026-10-24 |
| [0885](0885-contracts-checked-by-fuzzing.md) | Contracts checked by fuzzing: `ail test --contracts` gives a counterexample for a failing `requires` or `ensures` | Accepted | 2026-10-24 |
| [0890](0890-optimizer-roadmap-to-match-c.md) | Optimizer roadmap to match C at -O2/-O3: allocation, register allocation, inlining, loop opts, range analysis, auto-vectorization, maps, tail calls; speed-only per ADR 0874 *(delegated)* | Accepted | 2026-10-24 |
| [0908](0908-apps-in-their-own-repositories.md) | The apps (Garlic, Garlic Press, ail-edit, Clove) live in their own private AGPL-3.0 repositories; ail holds the language only | Accepted | 2026-10-25 |
| [0909](0909-ail-verify-diff.md) | `ail verify --diff`: proved, tested, unverified and new failures for a change | Accepted | 2026-10-25 |
| [0910](0910-test-bundle-failure.md) | `ail test --bundle-failure`: one file that reproduces a failing test | Accepted | 2026-10-25 |
| [0911](0911-default-automatic-parallel-loops.md) | Native code runs proven independent loops in parallel by default, past a work threshold | Accepted | 2026-10-25 |
| [0913](0913-http2-client-over-alpn.md) | An HTTP/2 client over ALPN in the native `std.net` *(delegated)* | Accepted | 2026-10-26 |
| [0914](0914-automatic-fork-join-of-independent-calls.md) | Run independent pure calls in a straight-line block as a fork-join, when each is worth it | Accepted | 2026-10-26 |
| [0915](0915-http-retry-any-method-on-stale-pooled-connection.md) | The HTTP client resends any method once when a pooled connection dies before a response byte | Accepted | 2026-10-26 |
| [0916](0916-agent-end-to-end-benchmark.md) | Agent end-to-end benchmark: the same model builds and fixes software in AIL and in C, with logged debug rounds and oracle-generated cases *(delegated)* | Accepted (harness on main; full runs pending) | 2026-10-26 |
| [0917](0917-ail-task-protocol.md) | `ail task`: start, status, verify and finish one change with a recorded base and a stated next step *(delegated)* | Accepted | 2026-10-26 |
| [0918](0918-ail-mcp-compiler-tools.md) | `ail mcp`: the compiler's own tools (check, test, ask, impact, context, edit, fix, explain, api, review, task; amended 2026-11-10) as an MCP server on stdio *(delegated)* | Accepted | 2026-10-26 |
| [0919](0919-attach-input-opt-in.md) | `ail run --attach-input`: an opt-in `input` request so an agent can click and type in a running app | Accepted | 2026-10-26 |
| [0920](0920-replay-shrink.md) | `ail replay --shrink`: the smallest recording that still fails the same way | Accepted | 2026-10-26 |
| [0921](0921-replay-probes.md) | Probes: `--at FILE:LINE --show a,b` prints chosen values each time a line runs | Accepted | 2026-10-26 |
| [0923](0923-struct-scalar-replacement.md) | Native: scalar replacement of struct objects that stay in one function, and inlining of constructors *(delegated)* | Accepted | 2026-10-26 |
| [0924](0924-ship-the-tz-database.md) | Ship the IANA tz database with AIL, so zones give the same answers on every OS | Accepted | 2026-10-27 |
| [0925](0925-windows-release-gate.md) | Windows: the full gate green before each release | Accepted | 2026-10-27 |
| [0926](0926-c-header-import-subset.md) | `use c "header.h"`: a subset C header reader that writes `pub extern c fn` declarations *(delegated)* | Accepted | 2026-10-28 |
| [0927](0927-fs-scan.md) | `fs.scan`: one bulk walk with allocated size, inode and kind *(delegated)* | Accepted | 2026-11-04 |
| [0928](0928-fs-volume.md) | `fs.volume`: total and free bytes of a volume *(delegated)* | Accepted | 2026-11-04 |
| [0929](0929-fs-read-range-trash.md) | `fs.read_range` and `fs.trash`: partial reads and recoverable delete *(delegated)* | Accepted | 2026-11-04 |
| [0930](0930-std-app-progress-events.md) | `std.app` progress events for long jobs | Accepted | 2026-11-04 |
| [0931](0931-std-app-canvas-hover.md) | `std.app` canvas hover events | Accepted | 2026-11-04 |
| [0932](0932-std-app-click-count-and-menus.md) | `std.app` click count and context menus | Accepted | 2026-11-04 |
| [0933](0933-std-app-alpha-fill.md) | `std.app` translucent fills | Accepted | 2026-11-04 |
| [0934](0934-promote-struct-fields-in-loops.md) | Native: struct fields kept in registers across loops: measured no gain, not merged *(delegated)* | Rejected | 2026-11-05 |
| [0935](0935-split-live-ranges-at-loops.md) | Native: live-range splitting at call-free loops: measured no gain, not merged *(delegated)* | Rejected | 2026-11-05 |
| [0936](0936-native-map-key-cache.md) | Native string-keyed maps cache keys by object, and an insert reuses the missed probe's hash *(delegated)* | Accepted | 2026-11-04 |
| [0937](0937-grid-flat-2d-list.md) | `Grid[T]`: a flat 2-D list in the prelude, indexed `g[i, j]` *(delegated)* | Accepted | 2026-11-04 |
| [0938](0938-auto-parallel-push-builder-loops.md) | Run push-builder loops on the pool: one push per iteration, chunks joined in order *(delegated)* | Accepted | 2026-11-04 |
| [0939](0939-match-jump-tables.md) | Native: `match` through a jump table, with the dispatch repeated in each arm *(delegated)* | Accepted | 2026-11-04 |
| [0940](0940-inliner-cost-model.md) | Native: inlining functions that call others, by a call-graph cost model *(delegated)* | Accepted | 2026-11-05 |
| [0941](0941-data-range-loop-guards.md) | Native: data-range guards for loop nests, and ikj interchange of reduction nests *(delegated)* | Accepted | 2026-11-04 |
| [0942](0942-user-loop-vector-kernels.md) | Native: vector kernels for swapped reduction nests (ADR 0845 stage 3, first step) *(delegated)* | Accepted | 2026-11-04 |
| [0943](0943-per-thread-allocation-caches.md) | Native: per-thread allocation caches once the pool exists, so allocation takes no lock after a parallel loop *(delegated)* | Accepted | 2026-11-04 |
| [0944](0944-parallel-gc-marking.md) | Native: the parked threads help mark on heaps of 32 MB and more *(delegated)* | Accepted | 2026-11-04 |
| [0945](0945-vector-escape-loops.md) | Native: vector kernel for escape-time loops, mandelbrot-style (ADR 0845 stage 2) *(delegated)* | Accepted | 2026-11-05 |
| [0946](0946-adaptive-thread-pool.md) | Native: a pool sized to every core (up to 64), and each region uses the workers its work repays *(delegated)* | Accepted | 2026-11-05 |
| [0949](0949-std-audio.md) | std.audio: sound as data, devices through a host program *(delegated)* | Accepted | 2026-11-05 |
| [0954](0954-const-generics.md) | Const generics: `int` parameters on types and functions (`struct Vec[T, const N int]`), by monomorphisation *(delegated)* | Accepted | 2026-11-05 |
| [0955](0955-custom-hash-names-fields.md) | Custom hashing: `impl Hash` names the fields that take part (amends 0133, resolves OQ-042) | Accepted | 2026-11-05 |
| [0956](0956-std-gfx-frame-loop.md) | `std.gfx`: the frame loop for real-time programs (key-up, vsync, `Input` snapshots) *(delegated)* | Accepted | 2026-11-05 |
| [0957](0957-shared-surface-transport.md) | Shared-surface transport for `std.gfx` (no JSON on the hot path) *(delegated)* | Accepted | 2026-11-05 |
| [0958](0958-gpu-compute-first.md) | SPIR-V and MSL compute first, then rendering *(delegated)* | Superseded by 1073 | 2026-11-05 |
| [0959](0959-std-crypto.md) | std.hash and std.crypto: in-house, constant-time, vector-checked primitives; HMAC-SHA-384/512 first *(delegated)* | Accepted | 2026-11-05 |
| [0965](0965-generators.md) | Generators: `yield` in a function returning `Seq[T]`, lowered to a state machine *(delegated)* | Superseded by 1073 | 2026-11-05 |
| [0966](0966-ail-lsp-language-server.md) | `ail lsp`: the semantic API as a language server on stdio *(delegated)* | Accepted | 2026-11-05 |
| [0967](0967-sanitizers.md) | Sanitizers: `ail sanitize` finds schedule-dependent results; leak and unsafe-race checks staged *(delegated)* | Accepted | 2026-11-05 |
| [0968](0968-aarch64-windows-target.md) | Native: the aarch64-windows target *(delegated)* | Deferred past 0.1.0 (experimental: compile-only, unverified) | 2026-11-05 |
| [0969](0969-leak-report.md) | The leak report: `ail leak` shows growth by kind and retention by root on the GC heap *(delegated)* | Accepted | 2026-11-06 |
| [0950](0950-c-abi-exports.md) | C ABI exports: `pub export c fn` and `ail header` *(delegated)* | Accepted (library output deferred) | 2026-11-06 |
| [0952](0952-layout-c-structs.md) | `@layout(c)` structs: raw memory behind `*P`, not GC objects *(delegated)* | Deferred | 2026-11-08 |
| [0971](0971-std-app-file-chooser.md) | `std.app` native file and folder chooser *(delegated)* | Accepted | 2026-11-06 |
| [0972](0972-windows-system-zone.md) | Windows system zone: map the Windows zone ID to its IANA name *(delegated)* | Accepted | 2026-11-07 |
| [0973](0973-std-ui-layout-and-more-widgets.md) | `std.ui.widgets`: layout on `Rect`, Label, Checkbox, RadioGroup, ProgressBar, Tabs *(delegated)* | Accepted | 2026-11-07 |
| [0974](0974-flac-decoding.md) | std.audio: decode FLAC in AIL, amends ADR 0949 *(delegated)* | Accepted | 2026-11-07 |
| [0975](0975-library-entry-rule.md) | Library entry: lazy runtime init, caller stack as GC root, serialised first slice *(delegated)* | Accepted | 2026-11-07 |
| [0976](0976-public-release-and-seed-trust.md) | Public release and seed trust: designer-built seeds, hash-pinned in the tree, self-reproducible | Accepted | 2026-11-07 |
| [0977](0977-fs-attributes.md) | `std.fs` gets `mode`, `set_mode`, `set_modified`, `read_link`, `symlink`, `truncate` and `write_at`, for archivers *(delegated)* | Accepted | 2026-11-07 |
| [0978](0978-std-app-paste-event.md) | `std.app`: a canvas `paste` event with the system clipboard's text *(delegated)* | Accepted | 2026-11-07 |
| [0979](0979-gfx-windows-publish-attach.md) | `std.gfx` windows publish the attach socket (`gfx.watch.run`, `App.shown`) *(delegated)* | Accepted | 2026-11-07 |
| [0980](0980-windows-c-calls.md) | Native C calls on Windows: `LoadLibraryA`, `GetProcAddress` and the Win64 convention *(delegated)* | Accepted | 2026-11-07 |
| [0981](0981-std-app-edge-mode.md) | `std.app` edge mode: the app draws into the title bar, with the window controls' insets and `drag`/`zoom` lines *(delegated)* | Accepted | 2026-11-07 |
| [0982](0982-std-app-clipboard-line.md) | `std.app` clipboard line: an app puts text on the system clipboard through `lines()` *(delegated)* | Accepted | 2026-11-08 |
| [1031](1031-variadic-c-calls.md) | Variadic C calls: `extern c fn f(a, ...)` with the platform's convention *(delegated)* | Accepted | 2026-11-09 |
| [0983](0983-std-app-mouse-down-up.md) | `std.app` `down` and `up` events with pointer capture, for drag selection *(delegated)* | Accepted | 2026-11-08 |
| [0985](0985-gfx-gamepad-input.md) | `std.gfx` gamepad input: `Input.pads`, the `pad` event, GameController on the Mac *(delegated)* | Accepted | 2026-11-08 |
| [0986](0986-app-tools-and-attach-mcp.md) | `@tool` methods: typed actions a running app offers agents (`tools`, `call` behind `--attach-input`), and `ail attach mcp` *(delegated)* | Accepted | 2026-11-08 |
| [0987](0987-auto-parallel-inner-loops.md) | Native: a proven loop inside a loop run in order may go to the pool (amends 0911) *(delegated)* | Accepted | 2026-11-08 |
| [0984](0984-std-app-font-op.md) | `std.app` `Font`: web fonts in the display list, kept by the host under an id | Accepted | 2026-11-08 |
| [0988](0988-std-media-webm.md) | `std.media.webm`: WebM and Matroska demuxing in AIL, fed in pieces for Media Source Extensions *(delegated)* | Accepted | 2026-11-08 |
| [0989](0989-std-video-vp9.md) | `std.video.vp9`: a VP9 profile 0 decoder in AIL, I420 frames and RGBA *(delegated)* | Accepted | 2026-11-08 |
| [0990](0990-std-audio-opus.md) | `std.audio.opus`: an Opus decoder in AIL, CELT first, then SILK and hybrid *(delegated)* | Accepted | 2026-11-08 |
| [0992](0992-fs-scan-count.md) | `fs.scan_count`: entries and bytes every scan has listed so far, for a live progress bar *(delegated)* | Accepted | 2026-11-08 |
| [0993](0993-audio-streaming-output.md) | std.audio.device: streaming output with a played-frames clock, amends ADR 0949 *(delegated)* | Accepted | 2026-11-08 |
| [0994](0994-overflow-facts-in-multi-exit-loops.md) | Native: drop overflow checks that bounds and index uses rule out, in loops that leave in more than one place *(delegated)* | Accepted | 2026-11-08 |
| [0996](0996-try-recv-and-group-fields.md) | `try_recv`, `recv_for`, and a `TaskGroup` field of a `unique` struct *(delegated)* | Accepted | 2026-11-08 |
| [1007](1007-std-io-write-bytes.md) | `io.write_bytes`: raw bytes to standard output *(delegated)* | Accepted | 2026-11-09 |
| [1008](1008-std-app-appearance.md) | `std.app`: an `appearance` event (`dark` or `light`) after start and on every system change *(delegated)* | Accepted | 2026-11-09 |
| [1009](1009-readiness-io.md) | Readiness I/O: `epoll` and `kqueue` sets for the HTTP server's parked connections *(delegated)* | Accepted | 2026-11-09 |
| [1010](1010-c-header-constants-enums-fnptr.md) | C header import breadth: `#define` constants, enums and function-pointer types *(delegated)* | Accepted | 2026-11-09 |
| [1011](1011-os-signals.md) | `os.signals`: SIGINT, SIGTERM, SIGHUP, SIGQUIT and SIGWINCH as a `Channel[str]`; the Windows console Ctrl-C handler *(delegated)* | Accepted | 2026-11-09 |
| [1005](1005-std-app-font-family-list.md) | `std.app` font-family lists: the host measures and draws in the first installed family and says which *(delegated)* | Accepted | 2026-11-08 |
| [1012](1012-std-term.md) | `std.term`: raw mode, key, mouse and resize events, the alternate screen and colours, in AIL over libc *(delegated)* | Accepted | 2026-11-09 |
| [1015](1015-user-fuzzing.md) | Coverage-guided fuzzing of user code: `ail test --fuzz NAME` with a corpus directory *(delegated)* | Accepted | 2026-11-09 |
| [1016](1016-fs-map.md) | `fs.map`: a read-only memory-mapped `bytes` view of a file *(delegated)* | Accepted | 2026-11-09 |
| [1017](1017-std-app-screen-size-and-scale.md) | `std.app` reports the display's size and scale on `start` and `resize` *(delegated)* | Accepted | 2026-11-09 |
| [1018](1018-task-set-threads.md) | `task.threads()` and `task.set_threads(n)`: the pool's cap at run time *(delegated)* | Accepted | 2026-11-09 |
| [1019](1019-ail-merge-driver.md) | `ail merge`: a git merge driver that merges `.ail` files by declaration; `--check` reports semantic conflicts *(delegated)* | Accepted | 2026-11-09 |
| [1020](1020-ail-context.md) | `ail context`: a token-budgeted bundle for an edit; amends ADR 0891 *(delegated)* | Accepted | 2026-11-09 |
| [1021](1021-perf-explain.md) | `ail perf --explain FN FILE`: per-loop optimisation remarks (bounds checks, vector kernels, parallel verdict, calls left) with rule and fix *(delegated)* | Accepted | 2026-11-09 |
| [1022](1022-mutation-testing.md) | Mutation testing: `ail test --mutate [--max N]` reports the changes no test notices *(delegated)* | Accepted | 2026-11-09 |
| [1023](1023-ail-infer-contracts.md) | `ail infer --contracts`: proposed `requires` and `ensures` from test calls; amends ADR 0895 *(delegated)* | Accepted | 2026-11-09 |
| [1024](1024-ail-review.md) | `ail review BASE..HEAD`: a semantic summary of a change in one budgeted report; amends ADR 0894 *(delegated)* | Accepted | 2026-11-09 |
| [1032](1032-std-app-vector-ops.md) | `std.app` vector ops: `Round`, `Poly`, `Line` *(delegated)* | Accepted | 2026-11-09 |
| [1033](1033-std-app-inset-right.md) | `std.app` edge mode: `inset_right` for window controls at the top-right (Windows, Linux); amends ADR 0981 *(delegated)* | Accepted | 2026-11-09 |
| [1034](1034-std-app-text-placement.md) | `std.app` text placement and `sans_line`; Windows font table *(delegated)* | Accepted | 2026-11-09 |
| [1035](1035-v0-1-0-freeze-and-release-candidate.md) | v0.1.0: freeze after the lanes in flight, one release candidate gated on three hosts | Accepted | 2026-11-09 |
| [1037](1037-no-downcast-of-any-trait.md) | No downcasting of `any Trait` | Accepted | 2026-10-24 |
| [1043](1043-versions-tags-changelog.md) | Pre-1.0 versions, release tags and the changelog *(delegated)* | Accepted | 2026-11-09 |
| [1044](1044-windows-gfx-surface-mapping.md) | `std.gfx` on Windows: the shared surface is a named file mapping, `vsync` after `DwmFlush`, XInput gamepads; amends ADR 0957 *(delegated)* | Accepted | 2026-11-10 |
| [1045](1045-attach-listener-on-windows.md) | The attach listener on Windows: `%TEMP%` discovery directory, and the listener stops when the window closes *(delegated)* | Accepted | 2026-11-10 |
| [1046](1046-agent-kit-in-ail-new.md) | `ail new` writes an agent kit: `AGENTS.md` and `.mcp.json`, `--no-agent` to skip *(delegated)* | Accepted | 2026-11-10 |
| [1047](1047-ail-check-fix.md) | `ail check --fix`: repair, then check, in one command *(delegated)* | Accepted | 2026-11-10 |
| [1048](1048-task-ledger.md) | The task ledger: `ail task` measures itself *(delegated)* | Accepted | 2026-11-10 |
| [1049](1049-x86-64-allocates-rdx-and-r11.md) | x86-64 allocates rdx and r11; divisions keep other values out of rdx; amends ADR 0385 *(delegated)* | Accepted | 2026-11-10 |
| [1050](1050-dominated-checks-and-field-alias-classes.md) | Drop checks a dominating check already passed; field stores forget one offset *(delegated)* | Accepted | 2026-11-10 |
| [1051](1051-core-card-and-std-lookup.md) | Core card, `ail card` and `ail ask std` *(delegated)* | Accepted | 2026-11-10 |
| [1052](1052-change-certificate.md) | Change certificate: `ail verify --diff --attest` and `--check` *(delegated)* | Accepted | 2026-11-10 |
| [1054](1054-ail-edit-refactors.md) | `ail edit` refactors: `change_sig`, `extract` and `move` *(delegated)* | Accepted | 2026-11-10 |
| [1055](1055-contracts-proved-by-interval-facts.md) | Prove contracts from interval facts and drop the proved checks; `ail prove` *(delegated)* | Accepted | 2026-11-10 |
| [1056](1056-warm-ail-mcp.md) | Keep `ail mcp` warm: direct native calls and a memo of unchanged read-only calls; amends ADR 0918 *(delegated)* | Accepted | 2026-11-10 |
| [1057](1057-ail-help.md) | `ail help`, `--help` and a real no-argument usage; JSON for agents *(delegated)* | Accepted | 2026-11-11 |
| [1058](1058-ail-on-path-and-doctor.md) | `ail` works from PATH through symlinks; `ail doctor` and `ail doctor --path` *(delegated)* | Accepted | 2026-11-11 |
| [1059](1059-no-unix-tools-in-shipped-commands.md) | Shipped commands run no Unix program; `fs.temp_dir` and `fs.make_temp_dir` *(delegated)* | Accepted | 2026-11-11 |
| [1061](1061-known-list-lengths.md) | Native: lists built to a known count check against that count *(delegated)* | Accepted | 2026-11-11 |
| [1062](1062-version-multi-exit-loops.md) | Native: version counted loops that leave in more than one place, and drop the checks on `xs[a + i]` *(delegated)* | Accepted | 2026-11-11 |
| [1063](1063-function-wide-value-ranges.md) | Native: function-wide value ranges drop checks and fold comparisons they decide *(delegated)* | Accepted | 2026-11-12 |
| [1064](1064-reserve-256-mib-ahead-of-push-loops.md) | Native: reserve up to 256 MiB of list elements ahead of a push loop *(delegated)* | Accepted | 2026-11-11 |
| [1065](1065-goals-c-parity-ai-first.md) | Aim for speed equal to C and AI-first features ahead of the market; accept more tokens *(designer)* | Accepted | 2026-11-11 |
| [1066](1066-pair-adjacent-float-operations.md) | Native: pair adjacent float element operations into two-lane vector operations (`Vec2`, native.vec.3) *(delegated)* | Accepted | 2026-11-12 |
| [1067](1067-closures.md) | Function literals that copy their locals, failing function values, and `any`/`all`/`find`/`filter`/`map`/`group_by` *(delegated)* | Accepted | 2026-11-12 |
| [1068](1068-std-error-kinds-are-enums.md) | The kinds of the standard library's errors are enums; a string compared with an enum gets a fix *(designer; details delegated)* | Accepted | 2026-11-12 |
| [1069](1069-nested-match-patterns.md) | Nested patterns in variant fields, checked as a pattern matrix *(delegated)* | Accepted | 2026-11-12 |
| [1070](1070-bit-operators.md) | Bit operators `& \| ^ ~ << >>` as sugar for the `int` methods; mixing needs parentheses; supersedes ADR 0465 *(delegated)* | Accepted | 2026-11-12 |
| [1071](1071-streaming-file-handles.md) | `std.file` Reader and Writer (unique, flush on drop) over `std.fs`; `io.read_exact`; resolves OQ-141 *(delegated)* | Accepted | 2026-11-12 |
| [1072](1072-scoped-runtime-capabilities.md) | `--allow`: scoped capabilities (paths, hosts, programs, variables) checked at run time; amends ADR 0165 *(delegated)* | Accepted | 2026-11-12 |
| [1073](1073-generators-and-user-iteration.md) | Generators, `Seq[T]` and `for` over any type with `next(mut self) T?` *(delegated)* | Accepted | 2026-11-12 |
| [1074](1074-c-structs-by-value-and-callbacks.md) | C structs by value and C callbacks: plain structs copied in the platform layout, callbacks are named functions; amends ADR 0952 *(delegated)* | Accepted | 2026-11-12 |
| [1075](1075-test-filter-and-fixtures.md) | `ail test --filter` and `--list`, and `@fixture` / `@needs` test fixtures *(delegated)* | Accepted | 2026-11-12 |
| [1076](1076-struct-field-defaults.md) | Struct field defaults `name Type = value`, plain constants only *(delegated)* | Accepted | 2026-11-12 |
| [1077](1077-decision-order-ai-understanding-first.md) | Delegated decisions: AI understanding first, human readability second, then speed, then tokens *(designer)* | Accepted | 2026-11-12 |
| [1078](1078-error-union-member-matching.md) | `match err` on an error union takes member types and their variants, narrows the name, and must cover every member *(designer; details delegated)* | Accepted | 2026-11-12 |
| [1079](1079-keep-loop-elements-in-registers.md) | Native: keep elements a loop assigns at an unchanging index in registers *(delegated)* | Accepted | 2026-11-12 |
| [1080](1080-one-guard-per-list-for-bounded-indexes.md) | Native: one guard test per list for indexes bounded by the counted loops around *(delegated)* | Accepted | 2026-11-12 |
| [1081](1081-formatter-writes-self.md) | The formatter writes `self.` before a bare member | Accepted | 2026-11-12 |
| [1082](1082-discarded-result-is-an-error.md) | A call whose value is thrown away and that does nothing else is an error (`eff.use.1`) | Accepted | 2026-11-12 |
| [1083](1083-ail-ask-answers-honestly.md) | `ail ask` answers honestly: built-in methods, a minimum score, misses with a hint, `ail ask how` *(delegated)* | Accepted | 2026-11-12 |
| [1084](1084-language-version-line-in-ail-pkg.md) | A language version line in `ail.pkg` and a `fix --migrate` hook *(delegated)* | Accepted | 2026-11-12 |
| [1085](1085-keywords-as-member-names.md) | Keywords as field, member, label and method names *(delegated)* | Accepted | 2026-11-13 |
| [1086](1086-heap-and-deque.md) | `Heap[T]` and `Deque[T]` in the prelude *(delegated)* | Accepted | 2026-11-12 |
| [1087](1087-std-papercuts.md) | Std papercuts: `assert(c, msg)`, `assert_err`, `assert_near`, format specs `08` `b` `,` `+` `e` *(delegated)* | Accepted | 2026-11-13 |
| [1088](1088-while-binding.md) | `while name := e` loops while a value is present *(delegated)* | Accepted | 2026-11-12 |
| [1089](1089-caps-attribute-on-functions.md) | `@caps(LIST)` limits a function to the listed capabilities *(delegated)* | Accepted | 2026-11-12 |
| [1090](1090-std-yaml.md) | `std.yaml`: the common subset of YAML read into the `Json` value *(delegated)* | Accepted | 2026-11-13 |
| [1091](1091-std-mcp-client.md) | `std.mcp`: a client for MCP servers over stdio *(delegated)* | Accepted | 2026-11-12 |
| [1092](1092-foreign-habit-diagnostics.md) | Rust, Go and TypeScript habits get an AIL message and a fix; `${` in a string is an error *(delegated)* | Accepted | 2026-11-13 |
| [1093](1093-tojson-maps-optionals.md) | `ToJson` on `Map[K, V]` and `T?` without an `impl` block *(delegated)* | Accepted | 2026-11-13 |
| [1094](1094-local-functions.md) | Named local functions: `fn name(...)` inside a body copies the locals it names, can recurse and is a value *(delegated)* | Accepted | 2026-11-13 |
| [1095](1095-else-leave-as-block-tail.md) | `e else return ...`, `else break` or `else continue` as a block's last statement gives `e`'s value (`err.else.7`) *(delegated)* | Accepted | 2026-11-13 |
| [1096](1096-x64-jumps-inside-32-byte-lines.md) | Native x86-64: keep jumps inside 32-byte lines *(delegated)* | Accepted | 2026-11-13 |
| [1097](1097-faster-gates.md) | Faster gates: `NAME@K/N` shards, one process per runnable spec fence, stage 1 in the pool, `--failed` rerun, fast `run` on Windows *(delegated)* | Accepted | 2026-11-13 |
| [1098](1098-std-kv.md) | `std.kv`: a key-value store in one file with a crash-safe log; `fs.sync` *(delegated)* | Accepted | 2026-11-13 |
| [1099](1099-collection-vocabulary.md) | One collection vocabulary: `fold`, `count(f)`, `flat_map` and the rest on every collection *(delegated)* | Accepted | 2026-11-13 |
| [1100](1100-sandboxed-tests.md) | `ail test` sandboxes each test; `@needs(net)`, `@needs(proc)`, `@needs(fs)` lift it; `--allow all` *(delegated)* | Accepted | 2026-11-13 |
| [1101](1101-test-stubs.md) | `std.test` stubs for files, variables, the clock and HTTP in tests *(delegated)* | Accepted | 2026-11-13 |
| [1102](1102-select-statement.md) | `select` waits on several channel operations; first ready arm in order, `after`, `closed` *(delegated)* | Accepted | 2026-11-13 |
| [1103](1103-distinct-types.md) | Distinct types `type UserId = int`: nominal, erased to the base in native code *(delegated)* | Accepted | 2026-11-13 |
| [1104](1104-default-parameters.md) | Default parameter values `name Type = value`, passed by name *(delegated)* | Accepted | 2026-11-13 |
| [1105](1105-scoped-caps-in-manifests-and-review.md) | Scoped capabilities in `ail.pkg` (`caps fs.read=./data`) give a run its policy; `ail review` reports widened scopes *(delegated)* | Accepted | 2026-11-13 |
| [1106](1106-struct-patterns.md) | Struct patterns and fields matched by name *(delegated)* | Accepted | 2026-11-13 |
| [1107](1107-labeled-break-continue.md) | Labeled loops: `outer: for`, `break outer`, `continue outer` *(delegated)* | Accepted | 2026-11-13 |
| [1108](1108-grammar-file.md) | `docs/spec/grammar.ebnf`, the whole grammar, checked against the tree by `tools/check-grammar.py` *(delegated)* | Accepted | 2026-11-13 |
| [1109](1109-check-means-build.md) | What `ail check` accepts the native build builds: a generated method-by-element-type matrix and the `native_matrix` gate suite *(delegated)* | Accepted | 2026-11-13 |
| [1110](1110-language-card-budget-16000.md) | The language card budget is 16,000 tokens (core about 3,000); coverage is checked by a docs-gate script, and `llms-full.txt` holds the card and how-tos *(designer)* | Accepted | 2026-11-13 |
| [1111](1111-checkbench.md) | checkbench: `ail check` measured against one injected mistake per copy of the corpus and examples, floors in the gate suite `checkbench` *(delegated)* | Accepted | 2026-11-13 |
| [1112](1112-check-one-pass-and-fuzz.md) | `ail check` reports everything in one pass, and a checker fuzz guards it *(delegated)* | Accepted | 2026-11-13 |
| [1113](1113-check-output-for-agents.md) | `ail check` output for agents: `ok` line, `next:` on each error, JSON schema and summary record, exit statuses *(delegated)* | Accepted | 2026-11-13 |
| [1115](1115-windows-short-parallel-regions.md) | Windows: after two parallel regions in a row that end within 40 ms, loops the compiler chose run in order (waiters poll there) *(delegated)* | Accepted | 2026-11-14 |
| [1116](1116-verify-evidence-categories.md) | `ail verify` names its evidence (checked, contracts, tested states, unverified); certificate 2 records environment and inputs *(delegated)* | Accepted | 2026-11-15 |
| [1117](1117-api-contract-tests.md) | Contract tests for the semantic API: positions, fix `expect` and `ail edit` `span`, `--refs` `complete`, MCP -32600 *(delegated)* | Accepted | 2026-11-15 |
| [1118](1118-bench-attribution-stress-gates-crypto-vectors.md) | 0.1.1 evidence: benchmark median, spread and single-thread column; gate suites `ffi_abi` and `stress`; Wycheproof vectors and a timing test for the in-house crypto *(delegated)* | Accepted | 2026-11-15 |
| [1119](1119-never-return-type.md) | `never`: the result type of a function that never returns; `n := s.to_int() else usage()` compiles *(designer)* | Accepted | 2026-11-15 |
| [1120](1120-one-line-if-elif.md) | The one-line `if` takes `elif d then e` arms before its `else`; `else if` stays a nested one-line `if` | Accepted | 2026-11-15 |
| [1121](1121-locals-may-shadow-functions-and-modules.md) | A local may hide a function, an imported module or (in a method) a field until its block ends; types, traits and constants stay errors *(designer)* | Accepted | 2026-11-15 |
| [1122](1122-key-function-errors-agree.md) | Key functions: `by: \|x T\| key` accepted; no `type.fn.4` after a failed receiver; each `expr.key.1` error names what is wrong *(delegated)* | Accepted | 2026-11-15 |
| [1123](1123-ail-ask-std-answers-in-one-step.md) | `ail ask std` answers in one step: a description line per answer, std sources in the index, a synonym table, a stricter `ail ask how` match *(delegated)* | Accepted | 2026-11-15 |
| [1124](1124-arithmetic-operator-traits.md) | Arithmetic operator traits `Add`, `Sub`, `Mul`, `Div`, `Rem`, `Neg` for any struct or enum; `BigInt` implements them, `Ord` and `ToStr` | Accepted | 2026-11-15 |
| [1125](1125-methods-agents-guess.md) | Guessed methods: `xs.slice(a, b)`/`slice(a)`, `s.slice(a)`, `set.to_list()`; fixes for `unwrap`/`unwrap_or`/`expect` and set `keys()`, help for map `insert`/`set` *(delegated)* | Accepted | 2026-11-15 |
| [1126](1126-std-modules-in-ail-without-use.md) | The standard modules written in AIL are in scope without `use` too, loaded when a file names one; parts of another module keep `use` *(delegated)* | Accepted | 2026-11-15 |
| [1128](1128-else-block-after-assignment.md) | `x = e else` accepts an indented block (`target = tmp` after `tmp := e else`) *(delegated)* | Accepted | 2026-11-15 |
| [1129](1129-str-errors.md) | `T ! str`: a function may fail with text; any `str` returned is the failure, `?` passes any error on as text, `else e` binds a `str` *(designer, details delegated)* | Accepted | 2026-11-15 |
| [1130](1130-certain-fixes-applied-by-check.md) | `ail check`, `build`, `run` and `test` write certain fixes (another language's spelling of an AIL form) into the file and go on; `--no-fix`, `AIL_FIX=0` | Accepted | 2026-11-15 |
| [1133](1133-map-literal-first-use.md) | `mut m := {}` is typed by `m.get_or(k, d)` as a first use; `m.contains(k)` before the first store waits for it *(delegated)* | Accepted | 2026-11-15 |
| [1134](1134-entry-copy-warning.md) | Warning `expr.bind.15`: a `mut` copy of a map or list entry that is changed and never stored back or read *(delegated)* | Accepted | 2026-11-15 |
| [1135](1135-self-read-store-and-text-failure-fixes.md) | `{}` typed by a store that reads it (`m[k] = (m.get(k) else 0) + n`); `T ! str`: certain `return` before a `str` tail, one err.str.4 for `! str` meant as no result; `_ := ` for a statement `else`; amends 1129, 1130 *(delegated)* | Accepted | 2026-11-17 |
| [1136](1136-every-error-in-one-run.md) | A syntax error in a function body hides only its statement, a failed import only its uses; `use src.x` from the root is checked as `use x` *(delegated)* | Accepted | 2026-11-15 |
| [1137](1137-agent-lessons-in-kit-and-mcp.md) | `ail new` AGENTS.md and the MCP `initialize` `instructions` open with the same two lessons: read the whole card once, re-read a file after `fixed ...` *(delegated)* | Accepted | 2026-11-15 |
| [1138](1138-string-slip-certain-fixes.md) | A literal `{` in a string that only text can follow, `\}`, `sep.join(xs)` and `"x" + s.pad_left(2, '0')` are certain fixes; amends 1130 *(delegated)* | Accepted | 2026-11-16 |
| [1139](1139-small-agent-slips-certain-fixes.md) | `for i, x in xs`, `n.pad_left` and `by: fn` are certain fixes; a missing project `use`, a held `T ! E`, `[[] for ..]` and `opt.map` get one clear help; amends 1130 *(delegated)* | Accepted | 2026-11-17 |
| [1140](1140-standard-text-failure.md) | The standard text failure: prelude `error struct Fail "{msg}"`, `fn f() ! Fail`, `return Fail(msg: "...")`; err.str.4 rewrites `! str` to it as a certain fix; `?` passes text on as a `Fail`; amends 1129, 1130, 1135 | Accepted | 2026-11-17 |
| [1141](1141-branch-free-selects-with-guards.md) | Small two-way branches become branch-free selects (`csel`/`csinc`, `cmov`); checked arms run unchecked only inside a range guard, else the original checked branch runs; exact halving after a parity test *(delegated)* | Accepted | 2026-11-17 |
| [1142](1142-element-wise-vector-loops-and-list-alias-classes.md) | Native: element-wise `float` loops run two iterations at a time with no overlap test (lists never share elements); an element store forgets only lists not known apart; loop headers keep what the loop can't write; repeated field and list-header loads move before the loop *(delegated)* | Accepted | 2026-11-17 |
| [1143](1143-split-loops-parallel-head.md) | A loop the judge keeps in order runs its leading bindings (the head) on the pool a block at a time and the rest in order (conc.par.20); the chunk scan takes fields, `match`, `if x :=`, `else`, tuples, literals and comprehensions (amends conc.par.8, conc.par.13) *(delegated)* | Accepted | 2026-11-17 |
| [1144](1144-four-element-vector-passes.md) | Native: element-wise `float` loops run four iterations a pass with a `Vec4` (one 256-bit register on x86-64, two 128-bit on ARM64, split before allocation), after single iterations up to a 32-byte boundary; then two, then one; same bits as the scalar loop *(delegated)* | Accepted | 2026-11-18 |
| [1145](1145-int-lanes-with-an-overflow-mask.md) | Native: element-wise `int` loops (`+ - & \| ^`) run in lanes; each pass tests an overflow mask before it stores, and a pass that overflows writes nothing and hands over to the scalar loop, which traps as before; a vector may be read in a block its definition dominates; splats are made once before the loop *(delegated)* | Accepted | 2026-11-18 |
| [1146](1146-optional-results-in-the-callers-frame.md) | Native: a present optional of a number that its caller only tests and unwraps is stored in a cell of the caller's frame, through a second entry `NAME$out`, not in a heap box (native.stack.3); a second result register is rejected | Accepted | 2026-11-18 |
| [1147](1147-calc-miss-fixes.md) | Certain fixes after the calculator rerun: `mut` on a stored-into parameter when nothing reads it after, `pass` deleted never `todo`, two err.str.4 gaps, no err.prop.1 under err.str.2, a core card line on `% -1` and `checked_rem`. |
| [1148](1148-map-lookups-compiled-into-the-caller.md) | Native: a `Map` or `Set` lookup that hits at once makes no call (hash and two index slots for word keys, the key cache for `str` keys, compiled into the caller); `m[k] op= v` and `m[k] = v` after `m.get(k) else` store through the entry the lookup found (native.map.1); a table's slot is the top bits of its hash, so integer keys in a row never collide *(delegated)* | Accepted | 2026-11-18 |
| [1149](1149-structs-kept-in-place-in-lists.md) | Native: a `List[S]` keeps `S` in place when its one to eight fields are `int`, `float`, `bool` or `char`; `xs[i]` is the address of the fields in the list's block; the layout is chosen per struct for the whole program, and any use that needs objects boxes it (native.flat.1) *(delegated)* | Accepted | 2026-11-18 |
| [1150](1150-select-chains-index-selects-unsigned-division.md) | Native: an `if`/`elif` chain becomes selects level by level; a picked index keeps its branch only when the pick feeds itself round a loop (amends native.select.1); `x / c`, `x % c` of a never-negative `x` use the unsigned high multiply (extends native.range.3); grow calls out of line, list literals without capacity tests *(delegated)* | Accepted | 2026-11-18 |
| [1151](1151-conditional-stores-as-selects.md) | Native: a branch whose arms store integers to slots of an object its block already stores to becomes selects and stores that always run; a checked `+` or `-` runs unchecked behind a sign-bit overflow test, else the original checked branch (native.select.3) *(delegated)* | Accepted | 2026-11-18 |
| [1152](1152-optionals-across-edges-as-two-values.md) | Native: an optional of a number carried from block to block (`mut best := none` in a loop) is two SSA values, present and number, with no box (native.stack.4); a cell per variable is rejected | Accepted | 2026-11-18 |
| [1154](1154-arm64-multiply-add.md) | arm64: an unchecked `mul` read only by the unchecked `add` or `sub` after it in its block is one `madd` or `msub`; x86-64 has no such instruction and is unchanged *(delegated)* | Accepted | 2026-11-18 |
| [1155](1155-builder-loop-pool-threshold.md) | Native: a loop the compiler chose that builds a list by `push` runs in order below 256 operations an iteration when the elements are not numbers (16 for numbers, as before); measured (amends conc.par.14, ADR 0938) *(delegated)* | Accepted | 2026-11-18 |
| [1156](1156-structs-with-pointers-kept-in-place.md) | Native: a struct kept in place in a list may hold texts, lists and other pointers; its block has kind 32 + fields and the collector scans `len` elements of it; copies are those of a struct object (native.flat.2) *(delegated)* | Accepted | 2026-11-18 |
| [1157](1157-counted-loops-need-more-work-for-the-pool.md) | Native: a loop the compiler chose whose work is all counted (no loop, recursion, unpriced call or allocation in its body) needs 1048576 operations for the pool, 16777216 when its body is one element-wise store of floats; a `/` or `%` counts 16, or 4 and 6 by an integer literal (amends conc.par.14) *(delegated)* | Accepted | 2026-11-18 |
| [1158](1158-heap-page-of-265-cache-lines.md) | Native heap: a page is 265 cache lines (16,960 bytes), so blocks in neighbouring pages are 576 bytes apart in the low 12 address bits and a loop that reads one list and writes the next does not alias at 4 KB (amends ADR 0562) *(delegated)* | Accepted | 2026-11-18 |
| [1159](1159-string-sort-keyed-pairs.md) | Runtime: `sort()` of 64 or more strings merges (first eight bytes as a big-endian key, string) pairs; equal keys compare the strings; same order, still stable *(delegated)* | Accepted | 2026-11-18 |
| [1160](1160-struct-list-loops-checks-constants-addresses.md) | Native: the check of an element kept in place leaves a counted loop (native.bce.18); `float` constants of a call-free loop are held in registers (completes ADR 1154's open item); arm64 `add` with a shifted operand for `a + x * 2^k`, x86-64 shift for `x * 2^k` *(delegated)* | Accepted | 2026-11-18 |
| [1161](1161-in-place-report-and-field-cap.md) | Tools and native: `ail perf --explain` prints, for each struct the program has a list of, whether lists keep it in place and the first reason when not; a struct of up to 16 fields is kept in place (was eight), measured (amends tool.perf.1, native.flat.1) *(delegated)* | Accepted | 2026-11-18 |
| [1162](1162-conditional-sums-as-checked-selects.md) | Native: `if c: n += w` (one checked `add` or `sub` on one arm) is `n + select(c, w, 0)` with the sum still checked: no guard and no slow path; `w` of 1 is the condition as 0 or 1; `AIL_OPT_OFF=selsum` (native.select.4) *(delegated)* | Accepted | 2026-11-18 |
| [1163](1163-error-type-shape-fixes.md) | Certain fix for `str ! str` (err.str.2) when the body shows which returns fail; err.msg.3 declares the text field an `error struct` message shows; card wording. |
| [1164](1164-guessed-std-names-fixes.md) | Diagnostics: `some(x)` is rewritten to `x`, `str.index_of`/`position` to `find`, a blank argument to `trim_start` is dropped (certain); `a.min()` and `a if c else b` get help *(delegated)* | Accepted | 2026-11-18 |
| [1165](1165-sweep-skips-unreached-pages.md) | Runtime: the collector's marker sets one byte per page a pointer reached; the sweep gives small pages with no such byte back to the page pool without reading them *(delegated)* | Accepted | 2026-11-18 |
| [1166](1166-held-result-one-step-fix.md) | Diagnostics: `r := f(x)` then `if t := r` gets a certain fix writing `if t := f(x)` (err.use.1; amends ADR 1130) *(delegated)* | Accepted | 2026-11-18 |
| [1167](1167-else-text-and-discarded-value-fixes.md) | Certain fixes: `else "text"` in a function that fails with text, `_ :=` on a `T?` statement, and a fallback line for a discarded value with a block. |
| [1168](1168-open-collection-later-use-fix.md) | Certain fix: a `[]` or `{}` whose first use only reads it gets `List[T]()`, `Set[T]()` or `Map[K, V]()` from a later `push`, `add` or `m.get(k) else d` (type.coll.13) *(delegated)* | Accepted | 2026-11-19 |
| [1169](1169-final-run-first-draft-misses.md) | Fixes and help for the final run's first-draft misses: pad fix for any receiver, pad-name help, `BigInt.to_int`, `to_list` on a list, optional if branches *(delegated)* | Accepted | 2026-11-19 |
| [1170](1170-float-parse-marker-and-split-marks.md) | Runtime: `s.to_float() else` gets the number or one marker NaN no text parses to, with no boxed optional and no flag; a one-byte `split` reads every separator from a word of marks *(delegated)* | Accepted | 2026-11-20 |
| [1171](1171-language-version-0-2.md) | The language version becomes 0.2: `ail new` writes `ail 0.2`, `ail 0.1` trees stay accepted, `fix --migrate` rewrites the line *(delegated)* | Accepted | 2026-11-20 |
| [1172](1172-first-draft-build-fixes.md) | Certain fixes that make a failed first draft build: helper on a built-in type as a function and its calls, `abs_val`, `xs.get(i)`, `c.to_int() - 48`, list of one, `(a, b) :=`, `xs[a else d]` *(delegated)* | Accepted | 2026-11-20 |
| [1173](1173-optional-typing-first-draft-fixes.md) | A `none` local compared with `none` keeps waiting for its type; `T?` method and `?` help quote the line; err.else.3 fix withheld when the name is used; `\"` in an interpolation and `io.print` get right fixes *(delegated)* | Accepted | 2026-11-21 |
| [1174](1174-public-corpus-layout.md) | The public corpus: `corpus/sets/SET/NAME/` beside the first 300, one licence line, the mistakes pairs with `why.tsv`, `tools/corpus/corpus.py` and the gate suite `corpus_sets` *(delegated)* | Accepted | 2026-11-21 |
| [1175](1175-more-first-draft-certain-fixes.md) | More certain fixes for first drafts: empty `{}` that nothing can fill, `+` on text in comprehensions, closures and interpolations, `unreachable` with no message, `return a, b`, `(a..b).rev()`, `xs.count(v)` *(delegated)* | Accepted | 2026-11-25 |
| [1176](1176-cards-teach-the-form-to-write.md) | The cards give the form to write next to each rewritten spelling, add the forms with no fix (a `mut` parameter and its call, a method on a `T?`, counting down, joining lists, `unreachable("why")`) and state edge behaviour from the spec; the core card is chosen by measured cause *(delegated)* | Accepted | 2026-11-25 |
| [1177](1177-one-call-lookup-and-forgiving-ask.md) | One call answers a lookup: `ail ask NAME...` prints each function's rule sentences and example lines, other languages' names answered from the checker's table, questions in any common form, `tests` and `where`, an unknown command names the one meant *(delegated)* | Accepted | 2026-11-25 |
| [1178](1178-list-get-and-concat.md) | `xs.get(i)` gives a `T?` (`none` outside the list) and `xs.concat(ys)` a new list, `s.find(sub, from)` searches from a position, `b.pow(e)` is an `int` power; `xs + ys` on lists gets the certain fix to `concat`; `at`, `nth` and `remove` on a list get help *(delegated)* | Accepted | 2026-11-25 |
| [1179](1179-checker-rough-edges-optional-branches.md) | Checker rough edges: `T` and `T?` branches give `T?` (type.opt.11), `_` repeats in nested scopes, certain fixes for `for c in s`, omitted optional fields, `char - char`, `int` beside a sized integer, `extend` on a new `[]` *(delegated)* | Accepted | 2026-11-25 |
| [1180](1180-literal-parameters-may-hide-a-local.md) | A function literal's parameters and a comprehension's variables may hide a local (expr.bind.16); statements keep no shadowing | Accepted | 2026-11-26 |
| [1181](1181-optional-try-fails-with-text.md) | `?` on an optional fails with text in a `T ! str` or `Fail` function: the context, else `a value is absent` (err.prop.6) | Accepted | 2026-11-26 |
| [1182](1182-tested-optional-reads-as-its-value.md) | A tested optional reads as its value (type.opt.12): an immutable `T?` local after `!= none`, in operators, receivers, arguments and returns; elsewhere still `T?` | Accepted | 2026-11-26 |
| [1183](1183-json-text-form-and-task-results.md) | A `Json` value's text is JSON (`j.to_str()`, `print(j)`, `"{j}"`), no longer the variant dump; a task handle pushed to a list gets help that writes `[f(x) par for x in xs]` *(delegated)* | Accepted | 2026-11-26 |
| [1184](1184-ail-eval.md) | `ail eval PART...` prints the value and type of expressions with no file: the parts run in one scratch `main`, a binding or `use` part sets up, a failing part names itself *(delegated)* | Accepted | 2026-11-26 |
| [1185](1185-bare-commands-mean-the-package-here.md) | A bare `ail check`, `run`, `test` or `build` acts on the package in the current directory; `ail test DIR` and `ail build DIR` work; `assert a == b` gets a certain fix to `assert(a == b)`; `Num(int)` says the field needs a name *(delegated)* | Accepted | 2026-11-26 |
| [1186](1186-ail-perf-run-counts.md) | `ail perf FILE [ARGS...]` runs the program with a counter at each statement and prints the functions and lines that ran the most (counts, not time); instrument mode `counts` |
| [1187](1187-heap-goal-and-page-return.md) | Runtime: the collection trigger is a goal that adapts (twice the live data, wider only while collecting takes over 6% of run time), and free pages the next cycle does not need go back to the system *(delegated)* | Accepted | 2026-11-26 |
| [1188](1188-channels-wake-one-sleeper.md) | Runtime: a channel change wakes one sleeping task (two lines of sleepers, each on its own word), the wake-up call is made after the lock is released, a thread looks briefly before it sleeps; the task queue stays one queue, by measurement *(delegated)* | Accepted | 2026-11-26 |
| [1189](1189-edit-oracle.md) | The edit oracle: a plain `ail test` on a file that differs from `HEAD` runs the changed pure functions of both versions on generated inputs and prints `changed behaviour:` and `same behaviour:` lines; `--against OLD`, `--no-oracle` | Accepted | 2026-11-26 |
| [1190](1190-affected-tests-inside-check.md) | `ail check` with no error runs the tests that have no stored pass and prints `tests: R run, P pass (C cached)`; a pass is keyed by the text of every declaration the test reaches by name; a failing test is exit 1; `--no-tests`, `AIL_CHECK_TESTS=0` *(delegated)* | Accepted | 2026-11-26 |
| [1191](1191-start-up-image-for-constants.md) | `ail build` computes constants at build time and keeps the objects in a start-up image in the executable's data (macOS host builds; lists, text, structs, enums, optionals, tuples); tables and `ref`s are computed at start-up as before *(delegated)* | Accepted | 2026-11-26 |
| [1194](1194-tool-reports-cross-checked-by-a-fuzz-gate.md) | Tool reports are cross-checked by a fuzz gate: generated programs with a known graph and a random edit, `ask`, the impact, `test --affected`, `verify`, `diff` and the fixes against the model and real runs (fuzz.tool.1-6, suite `toolfuzz`) *(delegated)* | Accepted | 2026-11-26 |
| [1195](1195-new-projects-deny-by-default.md) | `ail new` projects run deny-by-default: `package NAME caps fs=.,env,clock,random`; manifest path scopes are taken from the manifest's directory, and a refused call names the `caps` entry to add | Accepted | 2026-11-26 |
| [1196](1196-ail-mv.md) | `ail mv NAME... MODULE` moves declarations into a module in one step (the `move` edit takes `"ids"`), makes the module's first file, and makes a moved struct's fields `pub` when it is used outside |
| [1197](1197-bytes-loops-in-place-and-joined-map-keys.md) | `for b in bs` over a `bytes` reads the bytes in place, and `m[xs.join("")]` for a `List[char]` asks the map before it makes the string (amends native.map.1) *(delegated)* | Accepted | 2026-11-26 |
| [1198](1198-none-local-typed-by-its-other-operand.md) | A `none` local used as an operand before its first assignment takes its type from the other operand (type.opt.9), so `mut best := none` then `best == none or v > best` compiles *(delegated)* | Accepted | 2026-11-26 |
| [1199](1199-public-error-set-fix.md) | err.type.3 names the errors a public function was inferred to fail with and offers the fix that writes them (a guess, not a certain fix); the float-pattern help (expr.match.7) points to a guard; `check --fix` writes certain fixes behind a syntax error; `+` on text after the `else` of a one-line `if` | Accepted | 2026-11-26 |
| [1200](1200-first-draft-retries-round-two.md) | `run`, `build` and `test` show the `expr.match.3` warnings as one note line (`ail check` keeps the full text); a worker-pool how-to; `+` on text help for parts that hold quotes *(delegated)* | Accepted | 2026-11-26 |
| [1201](1201-enumerate-loops-without-pairs.md) | `for i, x in xs.enumerate()` over a list walks the list with a hidden count, no list of pairs; a `List[char]` join bound by the statement before a map lookup is looked up from the characters too (widens the native.map.1 sentence of ADR 1197) *(delegated)* | Accepted | 2026-11-26 |
| [1202](1202-first-draft-foreign-habit-fixes.md) | Fixes for `xs[i], xs[j] = xs[j], xs[i]` (`xs.swap(i, j)`), `[...xs, x]` (`xs.concat`), two keyword names in one declaration (both reported), and `+` after a fallback `else` *(delegated)* | Accepted | 2026-11-26 |
| [1203](1203-certain-first-draft-fixes.md) | The error set of a public function, `?` on an unhandled failure the function can pass on (or in `main`), and a float literal in a pattern as a guard are certain fixes *(designer)* | Accepted | 2026-11-26 |
| [1204](1204-task-group-results.md) | `TaskGroup[T]()` holds tasks that return `T`; `g.wait_all()?` returns their results as a `List[T]` in the order added (extends conc.task.8) *(delegated)* | Accepted | 2026-11-26 |
| [1205](1205-audit-agent-gaps.md) | Audit gaps: `fs=@tmp` for new projects (eff.allow.13), a warning for `e else a OP b` (expr.prec.1), `ail card full` named first in the kit and docs, `push(spawn ...)` help names `TaskGroup[T]` *(delegated)* | Accepted | 2026-11-26 |
| [1206](1206-more-certain-fixes-from-the-mistakes-corpus.md) | Certain fixes for `a, b = e1, e2` (through a temporary), a module's error type in the `main` signature fix, and `.unwrap()` on a call *(delegated)* | Accepted | 2026-11-26 |
| [1207](1207-std-db.md) | `std.db`: an embedded SQL database written in AIL; strict column types, `?` parameters, typed rows, transactions, a crash-safe log file *(delegated)* | Accepted | 2026-11-26 |
| [1208](1208-checkout-ail-is-stage-2.md) | In a checkout `./ail` runs stage 2 (stage 1, built by the seed, rebuilds the tree); `AIL_USE_SEED=1` keeps stage 1; amends ADR 0855 *(delegated)* | Accepted | 2026-11-26 |
| [1209](1209-fallible-task-groups.md) | `TaskGroup[T ! E]()` holds tasks that can fail; `wait_all()` fails with the first failure in add order and cancels the tasks after it; a `unique` struct may hold a `TaskGroup[T]` (amends conc.task.8, conc.task.10) *(delegated)* | Accepted | 2026-11-26 |
| [1210](1210-seed-mirror-voltare.md) | Seeds mirrored on seeds.voltare.us (immutable tags, manifest); fetch tries the mirror, then the GitHub release; pinned hashes stay the trust root | Accepted | 2026-11-26 |
| [1211](1211-keyword-names-and-silent-surprises.md) | A keyword used as a name is renamed by a certain fix inside a function; `s.to_int()` where an `int` is wanted gets help and an `else 0` fix; `x ^ 2` warns *(delegated)* | Accepted | 2026-11-26 |
| [1212](1212-std-xml.md) | `std.xml`: `parse` to a `Node` tree, slash-path `find`/`find_all`, `write`, `escape`; DOCTYPE refused so no entity expansion *(delegated)* | Accepted | 2026-11-26 |
| [1213](1213-proc-run-options.md) | `proc.options()` and `run_with`, `run_bytes_with`, `start_with` (working directory, environment, input, time limit); `proc.pid`, `os.hostname` *(delegated)* | Accepted | 2026-11-26 |
| [1214](1214-std-url.md) | `std.url`, written in AIL: RFC 3986 `parse` into a `Url`, `to_str`, `resolve`, `join_path`, percent and form codecs, query pairs; used as `url` *(delegated)* | Accepted | 2026-11-26 |
| [1215](1215-bytes-binary-reads-and-pushes.md) | Fixed-width integer and float reads on `bytes` (`u32_le(at)` and kin, `uvarint`) and binary pushes onto a `List[u8]` *(delegated)* | Accepted | 2026-11-26 |
| [1216](1216-path-completion.md) | `std.path` gains `is_abs`, `clean`, `abs`, `parts`, `join_all`, `rel`, `with_ext`, `split_ext`, `match` (glob with `[a-z]` classes) and `expand_home` (std.path.6–14) *(delegated)* | Accepted | 2026-11-26 |
| [1217](1217-http-router.md) | `std.net.http` router: `http.router()` with `:name` and `*name` patterns, 404 and 405 with `allow`, static mounts, a hook; `http.Req` with `param`, `json`, `form`; response helpers `text`, `html`, `json`, `redirect`, `not_found`, `mime_type` *(delegated)* | Accepted | 2026-11-26 |
| [1218](1218-std-textdiff.md) | `std.textdiff` in AIL: Myers shortest line diff (`lines`), `diff -u` text (`unified`), strict `apply` with a `PatchError`, `stat`, Levenshtein `distance` and `similarity` *(delegated)* | Accepted | 2026-11-26 |
| [1219](1219-std-semver.md) | `std.semver`: SemVer 2.0.0 `Version` (parse, `compare`, `Ord`, `bump`) and npm-style ranges (`parse_range`, `satisfies`, `max_satisfying`), written in AIL *(delegated)* | Accepted | 2026-11-26 |
| [1220](1220-std-ini.md) | `std.ini`: `ini.parse` reads INI into `Json` (all values `Str`), `ini.write`, `ini.parse_env` reads `.env` into a `Map` with `${VAR}` from the file; a duplicate is an error *(delegated)* | Accepted | 2026-11-26 |
| [1221](1221-monotonic-clock-and-durations.md) | `time.mono_ns()` and `time.since_ms(start_ns)` (monotonic on Linux, macOS and Windows; the runtime's deadline clock too), `date.duration_text(ms)` and `date.parse_duration(text)`; `test.clock` covers `mono_ns` *(delegated)* | Accepted | 2026-11-26 |
| [1222](1222-std-net-websocket.md) | `std.net.websocket`: a WebSocket client and server (RFC 6455) in AIL over `Conn`; `recv` joins fragments, answers ping and close, enforces the RFC rules (std.net.33-41) *(delegated)* | Accepted | 2026-11-26 |
| [1223](1223-std-textwrap.md) | `std.textwrap`: display width, wrap, dedent, indent, aligned tables, natural sort key, written in AIL *(delegated)* | Accepted | 2026-11-26 |
| [1224](1224-std-stats.md) | `std.stats`: mean, median, mode, variance, stdev, percentile, quantiles, histogram, correlation, moving average on a `List[float]`; `none` for empty input *(delegated)* | Accepted | 2026-11-26 |
| [1225](1225-channels-look-longer-before-sleeping.md) | Runtime: a thread looks at a channel 50,000 times (was 8,000) before it sleeps; a pool of one worker goes from 0.92 s to 0.72 s *(delegated)* | Accepted | 2026-11-26 |
| [1226](1226-compiler-wider-heap-goal.md) | The toolchain's own compiler (`compiler/main.ail`) may widen its heap goal to five times the live bytes; programs keep ADR 1187's three *(delegated)* | Accepted | 2026-11-26 |
| [1237](1237-tls-server.md) | A TLS 1.3 server: `net.serve_tls`, `net.listen_tls`, `listen_tls_alpn`, `Conn.alpn`, `http.serve_tls`, `Router.serve_tls`; ECDSA P-256 and RSA-PSS keys from PEM files, HelloRetryRequest, unaudited signing (std.net.46-49) *(delegated)* | Accepted | 2026-11-27 |
| [1227](1227-int-and-float-math-helpers.md) | `int` `gcd`, `lcm`, `isqrt`, `is_pow2`, `log2`; `float` `log2`, `is_nan`, `is_finite`, `is_infinite` and `float.PI`, `E`, `TAU`, `INF`, `NAN` *(delegated)* | Accepted | 2026-11-27 |
| [1228](1228-std-log-fields.md) | `std.log`: `key=value` fields, JSON lines and a file target, opt-in; amends 0628 *(delegated)* | Accepted | 2026-11-27 |
| [1229](1229-test-golden-and-temp-fixtures.md) | `std.test` golden files (`AIL_UPDATE_GOLDEN=1`) and temporary fixture directories *(delegated)* | Accepted | 2026-11-27 |
| [1230](1230-http-gzip-responses.md) | `std.net.http`: `r.gzip()` answers text bodies with gzip when the client accepts it *(delegated)* | Accepted | 2026-11-27 |
| [1231](1231-args-subcommands-env-version.md) | `std.args`: subcommands, environment fallbacks (`Spec.from_env`) and `--version` *(delegated)* | Accepted | 2026-11-27 |
| [1232](1232-json-pointer-and-merge-patch.md) | `Json.pointer` (RFC 6901) and `Json.merge` (RFC 7386) as prelude methods *(delegated)* | Accepted | 2026-11-27 |
| [1233](1233-http-client-helpers-and-retry.md) | `std.net.http` client helpers: `get_json`, `post_json`, `post_form`, multipart, `retry` with jittered backoff *(delegated)* | Accepted | 2026-11-27 |
| [1234](1234-std-lru.md) | `std.lru`: `Cache` (least recently used eviction) and `Ttl` (entries also expire) on one `Map` and index-linked arrays, written in AIL *(delegated)* | Accepted | 2026-11-27 |
| [1235](1235-std-throttle.md) | `std.throttle`: token bucket, sliding-window limiter and `pace` over channels, written in AIL on `time.mono_ns` *(delegated)* | Accepted | 2026-11-27 |
| [1240](1240-std-compress-unzstd.md) | `compress.unzstd`: an in-house Zstandard (RFC 8878) decoder; every frame, skippable frames, content size and XXH64 checksum checked; dictionaries `unsupported` (std.compress.14) *(delegated)* | Accepted | 2026-11-27 |
| [1241](1241-brotli-decompression.md) | `compress.unbrotli`: an in-house Brotli decoder (RFC 7932) in the native runtime, static dictionary generated from the RFC data and inflated once per process (std.compress.12-13) *(delegated)* | Accepted | 2026-11-27 |
| [1242](1242-std-uuid-base32-base58.md) | `std.uuid` (v4, v7, parse, order), `std.base32` (RFC 4648, extended hex, Crockford) and `std.base58` (Bitcoin), written in AIL *(delegated)* | Accepted | 2026-11-27 |
| [1243](1243-std-template.md) | `std.template`: `{{name}}` text templates over `Json` with `#each`, `#if` and HTML escaping on by default *(delegated)* | Accepted | 2026-11-27 |
| [1244](1244-std-hash-raw-digests.md) | `std.hash`: `_bytes` forms that return the raw digest as `bytes` *(delegated)* | Accepted | 2026-11-27 |
| [1245](1245-std-decimal.md) | `std.decimal`: exact base-10 `Decimal` on `std.bigint` (`+ - *`, comparison by value, `divide` and `round` with an explicit scale and one of seven rounding modes), written in AIL *(delegated)* | Accepted | 2026-11-27 |
| [1246](1246-std-bitset-bloom.md) | `std.bitset` (dense set of 0 to n - 1 in 64-bit words) and `std.bloom` (Bloom filter on it), written in AIL *(delegated)* | Accepted | 2026-11-27 |
| [1247](1247-date-patterns-and-rfc-text.md) | `std.time.date`: `format`/`parse` with patterns, RFC 3339 and RFC 2822 text *(delegated)* | Accepted | 2026-11-27 |
| [1248](1248-std-cbor-msgpack.md) | `std.cbor` and `std.msgpack`: `encode(j Json) bytes` and `decode(data bytes) Json` for CBOR and MessagePack; byte strings read as base64 `Str`, non-text keys and `ext` are refused *(delegated)* | Accepted | 2026-11-27 |
| [1249](1249-std-cron.md) | `std.cron` in AIL: `cron.parse` (five or six fields, names, steps, `@daily` macros), `next`, `prev`, `upcoming`, `matches` and `in_zone` for time zones, with skipped and repeated wall times defined *(delegated)* | Accepted | 2026-11-27 |
| [1250](1250-std-image-png.md) | `std.image` (RGBA8 `Image`) and `std.image.png`: PNG decode (all color types, depths, tRNS, Adam7) and encode, written in AIL *(delegated)* | Accepted | 2026-11-27 |
| [1251](1251-std-tar.md) | `std.tar`: `read`/`write` of a `List[Entry]` (ustar, pax, GNU long names, gzip by magic), `list`, `extract` that refuses unsafe paths and links before writing, `create`, written in AIL *(delegated)* | Accepted | 2026-11-27 |
| [1252](1252-fs-watch.md) | `fs.watch`: changes under a directory as a channel of events *(delegated)* | Accepted | 2026-11-27 |
| [1253](1253-std-net-sse.md) | `std.net.sse`: Server-Sent Events server streams and a reconnecting client *(delegated)* | Accepted | 2026-11-27 |
| [1254](1254-std-term-progress-and-prompts.md) | `std.term`: styled text, progress bars, spinners and prompts *(delegated)* | Accepted | 2026-11-27 |
| [1255](1255-std-jwt.md) | `std.jwt`: JSON Web Tokens, HMAC signing and claim checks (unaudited) *(delegated)* | Accepted | 2026-11-27 |
| [1256](1256-std-unicode.md) | `std.unicode`: normalization, case folding, mark stripping, grapheme clusters *(delegated)* | Accepted | 2026-11-27 |
| [1257](1257-std-config-writers.md) | `toml.write` and `yaml.write`, and `inf`/`nan` in the TOML and YAML readers *(delegated)* | Accepted | 2026-11-27 |
| [1258](1258-std-markdown.md) | `std.markdown`: CommonMark and GFM tables to a tree and to escaped HTML *(delegated)* | Accepted | 2026-11-27 |
| [1259](1259-std-csv-writer.md) | `std.csv`: dialects, streaming, and struct rows by derive *(delegated)* | Accepted | 2026-11-27 |
| [1260](1260-std-ordcoll.md) | `std.ordcoll`: ordered map, ordered set and indexed priority queue *(delegated)* | Accepted | 2026-11-27 |
| [1261](1261-std-imageops.md) | `std.image.ops`: resize, crop, rotate, flip, grayscale, compositing and drawing *(delegated)* | Accepted | 2026-11-27 |
| [1262](1262-std-net-smtp.md) | `std.net.smtp`: an SMTP client and MIME builder *(delegated)* | Accepted | 2026-11-27 |
| [1263](1263-regex-literal-prefix-scan.md) | std.regex scans for a literal prefix before the DFA and the Pike VM *(delegated)* | Accepted | 2026-11-27 |
| [1264](1264-std-humanize.md) | `std.humanize`: locale-aware numbers, percentages, compact numbers, byte sizes, ordinals and relative times, written in AIL *(delegated)* | Accepted | 2026-11-27 |
| [1265](1265-std-kv-scan-batch.md) | `std.kv`: sorted scans and all-or-nothing batches *(delegated)* | Accepted | 2026-11-27 |
| [1266](1266-semver-resolve.md) | `semver.resolve`: a dependency resolver in `std.semver` *(delegated)* | Accepted | 2026-11-27 |
| [1267](1267-std-oauth2.md) | `std.oauth2`: an OAuth 2.0 client written in AIL *(delegated)* | Accepted | 2026-11-27 |
| [1268](1268-std-textdiff-words-patience.md) | `std.textdiff` gains a word diff and a patience line diff *(delegated)* | Accepted | 2026-11-27 |
| [1269](1269-std-net-web.md) | `std.net.web`: cookies, sessions, CSRF, CORS and uploads for the HTTP server *(delegated)* | Accepted | 2026-11-27 |
| [1270](1270-std-resilience.md) | `std.resilience`: retry with backoff and jitter, circuit breaker, deadlines *(delegated)* | Accepted | 2026-11-27 |
| [1271](1271-std-postgres.md) | `std.db.postgres`: a PostgreSQL client in AIL *(delegated)* | Accepted | 2026-11-27 |
| [1272](1272-std-html.md) | `std.html`, written in AIL: browser-rule `parse` and `parse_fragment` that never fail, CSS selector `select`, readable `to_text`, `write`, `escape`/`unescape`, allowlist `sanitize` *(delegated)* | Accepted | 2026-11-27 |
| [1273](1273-std-i18n.md) | `std.i18n`: message catalogs *(delegated)* | Accepted | 2026-11-27 |
| [1274](1274-zstd-brotli-writers.md) | `compress.zstd` and `compress.brotli` write valid frames without a match search *(delegated)* | Accepted | 2026-11-27 |
| [1275](1275-std-cloud-s3.md) | `std.cloud.s3`: an S3-compatible object storage client with SigV4 *(delegated)* | Accepted | 2026-11-27 |
| [1276](1276-std-net-http-middleware.md) | `std.net.http` router middleware, groups and built-ins *(delegated)* | Accepted | 2026-11-27 |
| [1277](1277-std-graphql.md) | `std.graphql`: `graphql.parse` reads query documents into a tree, `write` prints it, a `Client` posts a query as JSON over `std.net` and decodes `data` and `errors`; replies with errors are replies *(delegated)* | Accepted | 2026-11-27 |
| [1278](1278-std-qr.md) | `std.qr`: QR Code Model 2 encoder (modes, versions 1-40, levels, masks) with matrix, text art and SVG output, written in AIL *(delegated)* | Accepted | 2026-11-27 |
| [1279](1279-std-barcode.md) | `std.barcode`: Code 128, Code 39, EAN-13, UPC-A and Interleaved 2 of 5 as bar-width lists, check digits, decoding from widths, text art and SVG, written in AIL *(delegated)* | Accepted | 2026-11-27 |
| [1280](1280-router-hijack-websocket-upgrade.md) | A router route can take over its connection: `Exchange.hijack`, `Router.upgrade`, `websocket.upgrade` *(delegated)* | Accepted | 2026-11-27 |
| [1281](1281-std-jobs.md) | `std.jobs`: an in-process job queue and scheduler *(delegated)* | Accepted | 2026-11-27 |
| [1282](1282-std-feed.md) | `std.feed`: RSS 2.0, RSS 1.0, Atom and JSON Feed read into one `Feed`/`Item` model, date parsing, enclosures; RSS 2.0, Atom and JSON Feed written *(delegated)* | Accepted | 2026-11-27 |
| [1283](1283-std-mysql.md) | `std.db.mysql`: a MySQL and MariaDB client in AIL *(delegated)* | Accepted | 2026-11-27 |
| [1284](1284-std-db-tls.md) | TLS for `std.db.postgres` and `std.db.mysql` *(delegated)* | Accepted | 2026-11-27 |
| [1285](1285-std-net-http2-server.md) | `std.net.http2`: an HTTP/2 server *(delegated)* | Accepted | 2026-11-27 |
| [1286](1286-std-net-grpc.md) | `std.net.grpc`: gRPC over HTTP/2 *(delegated)* | Accepted | 2026-11-27 |
| [1287](1287-std-net-static-files.md) | `std.net.fileserve`: validators, ranges and precompressed files for static serving *(delegated)* | Accepted | 2026-11-27 |
| [1288](1288-std-db-redis.md) | `std.db.redis`: a Redis client in AIL *(delegated)* | Accepted | 2026-11-27 |
| [1289](1289-std-image-gif-webp.md) | `std.image.gif` (GIF decode to whole-canvas RGBA8 frames with delays and loop count) and `std.image.webp` (lossless VP8L decode), written in AIL *(delegated)* | Accepted | 2026-11-27 |
| [TBD](TBD-std-webp-lossy-alpha-animation.md) | `std.image.webp` lossy (VP8 key frames), alpha (`ALPH`) and animation (`ANIM`/`ANMF`) decode, with `webp.decode_animation` *(delegated)* | Accepted | 2026-11-27 |
| [1290](1290-std-wasm.md) | `std.wasm`: WebAssembly binary modules, read, checked and written *(delegated)* | Accepted | 2026-11-27 |
| [1291](1291-std-net-imap.md) | `std.net.imap`: an IMAP client and MIME parser *(delegated)* | Accepted | 2026-11-27 |
| [1292](1292-deflate-flush-compressed-websocket-send.md) | `compress.deflate_flush` and compressed WebSocket sends *(delegated)* | Accepted | 2026-11-27 |
| [1293](1293-std-protobuf.md) | `std.protobuf`: proto3 wire format (`decode`, `encode`), a dynamic `Message` by field number with unknown fields kept, `.proto` schemas (`parse_schema`), typed `to_json` and `from_json` *(delegated)* | Accepted | 2026-11-27 |
| [1294](1294-std-sqlite.md) | `std.sqlite`: read SQLite 3 database files in AIL (schema, tables by scan and rowid, index entries, overflow pages); no SQL, no writes *(delegated)* | Accepted | 2026-11-27 |
| [1295](1295-std-toml-yaml-conformance.md) | `std.toml` and `std.yaml` against the official test suites *(delegated)* | Accepted | 2026-11-27 |
| [1296](1296-std-unicode-layout.md) | `std.unicode` layout: display width, line breaking and word segmentation *(delegated)* | Accepted | 2026-11-27 |
| [1236](1236-std-crypto-signing.md) | `std.crypto` signing and key agreement: Ed25519, ECDSA P-256 with RFC 6979 nonces, X25519, on unaudited in-house code; bad secrets trap, bad outside input is `false` or `none` (std.crypto.5-8) *(delegated)* | Accepted | 2026-11-27 |
| [1238](1238-std-crypto-seal-open.md) | `std.crypto` authenticated encryption: `seal`/`open` (ChaCha20-Poly1305) and `seal_aes_gcm`/`open_aes_gcm` (AES-256-GCM), random nonce prepended, `bytes?` on any mismatch; labelled unaudited | Accepted | 2026-11-27 |
| [1237](1237-tls-server.md) | A TLS 1.3 server: `net.serve_tls`, `net.listen_tls`, `listen_tls_alpn`, `Conn.alpn`, `http.serve_tls`, `Router.serve_tls`; ECDSA P-256 and RSA-PSS keys from PEM files, HelloRetryRequest, unaudited signing (std.net.46-49) *(delegated)* | Accepted | 2026-11-27 |
| [1236](1236-std-crypto-signing.md) | `std.crypto` signing and key agreement: Ed25519, ECDSA P-256 with RFC 6979 nonces, X25519, on unaudited in-house code; bad secrets trap, bad outside input is `false` or `none` (std.crypto.5-8) *(delegated)* | Accepted | 2026-11-27 |
| [TBD](TBD-std-png-apng.md) | `std.image.png` APNG: `png.decode_animation` (whole-canvas frames, dispose and blend ops), `png.encode_animation` and `png.is_animated` *(delegated)* | Accepted | 2026-11-27 |
