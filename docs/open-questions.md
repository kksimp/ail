# Open questions

Unresolved design questions. Each question lists its options with pros and
cons, plus a recommendation. When a question is resolved it moves into an ADR
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
[ADR 0020](decisions/0020-linux-no-libc.md) removes libc from AIL's Linux
runtime, but C libraries expect an initialised libc (malloc, errno, TLS,
pthreads). To be settled in the FFI topic. Candidate approaches: load libc
only when FFI is used, with AIL threads made libc-compatible; or AIL provides
its own libc-compatible shim.

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
`#` is taken by comments ([ADR 0043](decisions/0043-hash-comments.md)).
Attributes are needed for fixed layout ([ADR 0013](decisions/0013-compiler-chosen-layout.md)),
inlining hints, FFI and tests. The natural candidate is `@name` (e.g. `@layout(c)`).
To be asked with the declarations grammar.

## OQ-028 — Local names vs. module-level, imported and built-in names
[ADR 0044](decisions/0044-no-shadowing.md) forbids local shadowing. Should a
local binding be allowed to reuse the name of a module-level item, an import,
or a built-in (e.g. a local `len`)? To be decided with the modules topic.

## OQ-029 — Error and absence handling forms
**Partly resolved:** fallbacks and early exit ([ADR 0049](decisions/0049-else-for-fallback-and-early-exit.md)),
context ([ADR 0050](decisions/0050-error-return-traces.md)), and `?` on optionals
([ADR 0051](decisions/0051-no-implicit-optional-to-error.md)).
**Still open:** safe member access (`a?.b`) and matching on specific errors.
Both depend on the `match` design.

## OQ-030 — What is an error type?
[ADR 0046](decisions/0046-inferred-error-sets.md). Can any type be an error,
or only declared error types? How is an explicit union written? How does an
inferred set convert to a public API's error type? How are recursion and
generics handled during inference?

## OQ-031 — Canonical spelling of the default numeric types
**Resolved** by [ADR 0059](decisions/0059-default-numeric-spelling.md).

---

### Not yet opened
The proposals in [design/ai-author-wishlist.md](design/ai-author-wishlist.md)
will be raised as questions under their respective topics.

The 28 design topics in [status.md](status.md) each still need their own
questions. They will be opened topic by topic during Stage 1.
