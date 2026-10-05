# Open questions

Unresolved design questions. Each question lists its options with pros and
cons, plus a recommendation. Since [ADR 0076](decisions/0076-delegated-decisions.md),
routine questions are decided by the AI using the priority order (tokens, AI
understanding, readability, speed). Serious ones go to the designer. When a question is resolved it moves into an ADR
and its entry here is replaced by a link.

IDs (`OQ-NNN`) are never reused.

---

## OQ-001 — ISA priority policy
**Resolved** by [ADR 0008](decisions/0008-isa-priority-policy.md) (strict lockstep).

## OQ-002 — Default atomic memory ordering
**Resolved** by [ADR 0025](decisions/0025-memory-model-atomics.md).

## OQ-003 — ISA-specific hardware features in the GC and runtime
**Resolved** by [ADR 0026](decisions/0026-hardware-feature-acceleration.md).

## OQ-004 — CI merge gating
**Resolved** by [ADR 0027](decisions/0027-ci-gating-all-targets.md).

## OQ-005 — Compiler host platforms
**Resolved** by [ADR 0031](decisions/0031-hosts-and-cross-compilation.md).

## OQ-006 — Spec chapter outline
Approve, reorder, or replace the provisional chapter list in
[spec/README.md](spec/README.md). *Recommendation:* approve it as a starting
point. Chapters can be added later without renumbering existing rule IDs.

## OQ-007 — Token-efficiency benchmark methodology
**Resolved** by [ADR 0029](decisions/0029-token-efficiency-benchmarks.md).

## OQ-008 — Performance benchmark methodology
**Resolved** by [ADR 0030](decisions/0030-performance-benchmarks.md) (initial scope; three-bar suite deferred).

## OQ-009 — Integer overflow semantics
**Resolved** by [ADR 0035](decisions/0035-integer-overflow-traps.md).

## OQ-010 — Implementation language of the bootstrap compiler
**Resolved** by [ADR 0023](decisions/0023-rust-bootstrap.md).

## OQ-011 — Self-hosting and roadmap order
**Resolved** by [ADR 0022](decisions/0022-staged-self-hosting.md) (self-hosting) and [ADR 0024](decisions/0024-semantic-api-from-stage-2.md) (semantic API timing). The remaining roadmap refinements (#2 and #4) are still open in [roadmap.md](roadmap.md).

## OQ-012 — May the compiler emit C as a temporary path to native code?
**Resolved** by [ADR 0017](decisions/0017-no-c-emission.md).

## OQ-013 — Strategy for the first native backend
**Resolved** by [ADR 0018](decisions/0018-optimizing-backend-from-start.md).

## OQ-014 — Linux runtime without libc?
**Resolved** by [ADR 0020](decisions/0020-linux-no-libc.md).

## OQ-015 — Default data layout
**Resolved** by [ADR 0013](decisions/0013-compiler-chosen-layout.md).

## OQ-016 — Compressed managed references
**Resolved** by [ADR 0014](decisions/0014-compressed-managed-references.md).

## OQ-017 — Whole-program optimisation by default
**Resolved** by [ADR 0015](decisions/0015-always-whole-program-incremental.md), [ADR 0016](decisions/0016-optimized-and-observable-debugging.md).

## OQ-018 — Which memory-management R&D tracks to pursue
**Resolved** by [ADR 0012](decisions/0012-memory-rd-tracks.md).

## OQ-019 — Debug-info formats
**Resolved** by [ADR 0021](decisions/0021-debug-info-formats.md).

---

## OQ-020 — Correctness oracles for the optimising backend
**Resolved** by [ADR 0019](decisions/0019-miscompilation-defence.md).

## OQ-021 — How do Linux programs that call C libraries get a libc?
**Resolved** by [ADR 0105](decisions/0105-libc-only-with-c.md).

## OQ-022 — Keep the Rust interpreter after self-hosting?
**Resolved** by [ADR 0032](decisions/0032-keep-rust-interpreter-as-oracle.md).

## OQ-023 — Backend-ready core subset
[ADR 0022](decisions/0022-staged-self-hosting.md) requires the language features
the backend uses to stabilise early. Which features make up the "core
subset"? To be decided once the type system and syntax are drafted.

## OQ-024 — Git workflow once code exists
**Resolved** by [ADR 0028](decisions/0028-git-workflow.md).

## OQ-025 — What kind of C do the drag races compare against?
**Resolved** by [ADR 0036](decisions/0036-drag-race-c-styles.md).

## OQ-026 — Indentation unit
**Resolved** by [ADR 0040](decisions/0040-four-space-indentation.md).

## OQ-027 — Attribute and annotation syntax
**Resolved** by [ADR 0083](decisions/0083-attribute-syntax.md).

## OQ-028 — Local names vs. module-level, imported and built-in names
**Resolved** by [ADR 0084](decisions/0084-no-shadowing-module-names.md).

## OQ-029 — Error and absence handling forms
**Partly resolved:** fallbacks and early exit ([ADR 0049](decisions/0049-else-for-fallback-and-early-exit.md)),
context ([ADR 0050](decisions/0050-error-return-traces.md)), `?` on optionals
([ADR 0051](decisions/0051-no-implicit-optional-to-error.md)), and handling
success *or* failure as a statement ([ADR 0125](decisions/0125-if-binding.md):
`if x := e` … `else err`).
**Still open:** safe member access (`a?.b`), and matching on specific errors,
including on a set of error types (`IoError | ParseError`). Options for the
latter:
- qualified variant patterns from every member type (`ParseError.Empty =>`),
  plus a pattern for a whole error type;
- `match` with success and error arms.

## OQ-030 — What is an error type?
**Partly resolved:** error types are declared with `error`
([ADR 0120](decisions/0120-error-keyword.md)). Unions, propagation and
recursion are covered by [ADR 0122](decisions/0122-error-unions-and-reports.md).
**Still open:** how an inferred set converts to a public API's error type,
and inference through generic functions.

## OQ-031 — Canonical spelling of the default numeric types
**Resolved** by [ADR 0059](decisions/0059-default-numeric-spelling.md).

## OQ-032 — Exclusivity of `mut` arguments
**Resolved** by [ADR 0064](decisions/0064-exclusivity.md).

## OQ-033 — Visibility of copy-on-write copies
**Resolved** by [ADR 0065](decisions/0065-copy-reporting.md).

## OQ-034 — Deterministic cleanup of resources
**Resolved** by [ADR 0066](decisions/0066-unique-resource-types.md).

## OQ-035 — Parameter form that takes ownership
**Resolved** by [ADR 0068](decisions/0068-move-parameters.md) (`move`) and [ADR 0069](decisions/0069-unique-keyword.md) (`unique`).

## OQ-036 — Method placement, associated functions, and coherence
**Resolved** by [ADR 0082](decisions/0082-method-placement-coherence.md).

## OQ-037 — Associated types vs. trait type parameters
Should traits declare associated types (`type Item` inside `trait Iterator`;
one per implementing type), take type parameters (`trait Iterator[Item]`), or
both, as in Rust (`From[T]` vs. `Iterator`)? To be decided with collections
and iteration, measured on a real corpus. *Owner: AI under ADR 0076, unless
it proves hard to reverse.*

## OQ-038 — In-place iteration over list elements
**Resolved** by [ADR 0121](decisions/0121-for-mut.md) (`for mut x in xs`).

---

### Not yet opened
The proposals in [design/ai-author-wishlist.md](design/ai-author-wishlist.md)
will be raised as questions under their respective topics.

The 28 design topics in [status.md](status.md) each still need their own
questions. They will be opened topic by topic during Stage 1.
