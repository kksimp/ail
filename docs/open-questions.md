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
Which platforms must the `ail` compiler itself **run on**, as opposed to targeting?

| Option | Pros | Cons |
|---|---|---|
| **The same 5 Tier 1 targets, all able to cross-compile to all (recommended)** | Symmetric. Any developer machine can build any target. | Cross-linking to Mach-O and PE needs our own linker or bundled SDK stubs. Apple SDK licensing needs checking. |
| Same 5 hosts, but each builds only for its own OS | Simpler linking. | Breaks "one machine builds everything" for CI and release workflows. |

## OQ-006 — Spec chapter outline
Approve, reorder, or replace the provisional chapter list in
[spec/README.md](spec/README.md). *Recommendation:* approve it as a starting
point. Chapters can be added later without renumbering existing rule IDs.

## OQ-007 — Token-efficiency benchmark methodology
How do we **measure** token efficiency (brief §4)? See
[design/feasibility.md](design/feasibility.md).

| Option | Pros | Cons |
|---|---|---|
| **Fixed corpus of equivalent programs in AIL, C, Rust, Go, Python, and others, measured with several real model tokenizers, plus task-level metrics (tokens to modify or repair) (recommended)** | Honest and repeatable. Measures the whole loop, not just syntax. | Costly to build and maintain the corpus. Task metrics need a harness. |
| Static token counts only | Cheap and quick. | Misses the biggest wins (context and repair loops). Easy to game with cryptic syntax. |

## OQ-008 — Performance benchmark methodology
How do we measure "C-class" performance (brief §1, §20)?

| Option | Pros | Cons |
|---|---|---|
| **A suite of equivalent programs against C (clang and gcc, -O2/-O3) and Rust on all 5 targets, with per-commit regression tracking and published results (recommended)** | Honest and measurable. Catches regressions early. | Infrastructure cost. Writing "equivalent" programs takes care. |
| Microbenchmarks only | Cheap. | Doesn't reflect real applications. Easy to over-fit. |

## OQ-009 — Integer overflow semantics
Affects both safety and performance. Raised early because it constrains the optimizer.

| Option | Pros | Cons |
|---|---|---|
| Trap on overflow in all builds, with explicit wrapping and saturating operators | Safe. Bugs are found. | A small runtime cost, partly removable by range analysis. It can block some vectorisation. |
| Trap in debug, wrap in release | Fast release builds. | Behaviour differs between builds, so bugs hide in production. |
| Always wrap | Fastest and simplest. | Silent bugs. |
| Overflow is undefined behaviour (as in C) | Maximum optimizer freedom. | Unsafe. Contradicts "safe by default". |

*No recommendation yet.* This needs the type-system discussion first.

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
After the compiler is fully AIL ([ADR 0022](decisions/0022-staged-self-hosting.md)),
the interpreter remains the differential-testing oracle ([ADR 0019](decisions/0019-miscompilation-defence.md)).

| Option | Pros | Cons |
|---|---|---|
| **Keep it in Rust permanently as an independent oracle (recommended)** | Two independent implementations in different languages catch bugs that a shared codebase would hide (N-version testing). | A permanent development-time Rust dependency (not shipped). Two implementations to keep in sync with the spec. |
| Port it to AIL | Everything is AIL. No Rust anywhere. | The oracle shares the compiler's language and tooling, so bugs in common code can hide from both. |

## OQ-023 — Backend-ready core subset
[ADR 0022](decisions/0022-staged-self-hosting.md) requires the language features
the backend uses to stabilise early. Which features make up the "core
subset"? To be decided once the type system and syntax are drafted.

## OQ-024 — Git workflow once code exists
**Resolved** by [ADR 0028](decisions/0028-git-workflow.md).

---

### Not yet opened
The proposals in [design/ai-author-wishlist.md](design/ai-author-wishlist.md)
will be raised as questions under their respective topics.

The 28 design topics in [status.md](status.md) each still need their own
questions. They will be opened topic by topic during Stage 1.
