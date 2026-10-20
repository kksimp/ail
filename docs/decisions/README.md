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
| [0133](0133-automatic-hash.md) | `Hash` is automatic, like `Eq` and `ToStr` (amends 0126) | Accepted | 2026-10-05 |
| [0134](0134-characters.md) | `char`: traits, classification, digits and code points *(delegated)* | Accepted | 2026-10-05 |
| [0135](0135-string-literals-and-methods.md) | Multi-line and raw strings, `\'`, and the `str` methods (amends 0089) *(delegated)* | Accepted | 2026-10-05 |
| [0136](0136-list-methods-and-slices.md) | List methods, open-ended slices that trap, and `xs.sum()` *(delegated)* | Accepted | 2026-10-05 |
| [0137](0137-integer-methods-and-min-max.md) | Integer arithmetic methods; `min`/`max` as `Ord` methods; inherent methods beat prelude defaults (amends 0082, 0127) *(delegated)* | Accepted | 2026-10-05 |
| [0138](0138-literal-and-or-patterns.md) | Literal patterns, or-patterns, and one-line statement arms *(delegated)* | Accepted | 2026-10-05 |
| [0139](0139-unit-results-statement-else-and-divergence.md) | `! E` results; statement `else`; divergence is a rule, not a type (amends 0125) *(delegated)* | Accepted | 2026-10-05 |
| [0140](0140-diagnostics-for-other-languages.md) | Names other languages spell differently get a fixed help *(delegated)* | Accepted | 2026-10-05 |
| [0141](0141-implicit-self.md) | Inside a method, the receiver's fields and methods are in scope by bare name | Accepted | 2026-10-05 |
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
