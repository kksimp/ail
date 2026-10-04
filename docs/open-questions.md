# Open questions

Unresolved design questions. Each question lists its options with pros and
cons, plus a recommendation. When a question is resolved it moves into an ADR
and its entry here is replaced by a link.

IDs (`OQ-NNN`) are never reused.

---

## OQ-001 — ISA priority policy
**Context:** [ADR 0008 (Proposed)](decisions/0008-isa-priority-policy.md). The
designer leans toward equal priority but is worried x86 will hold the project
back.

| Option | Pros | Cons |
|---|---|---|
| **Equal Tier 1, ARM64 as reference (recommended)** | No adoption hit. Weak-memory bugs caught. Guardrails protect the language. | Most total work. Brief x86 lag within a stage. |
| Strict lockstep | Perfect parity. Neutral IR. | Slowest iteration. x86 sets the pace. |
| ARM64 first, x86 by 1.0 | Fastest first custom backend. | Long x86 lag. ARM assumptions creep in. |

## OQ-002 — Default atomic memory ordering
**Context:** decides whether x86's TSO model shapes the concurrency semantics.

| Option | Pros | Cons |
|---|---|---|
| **Weakly-ordered abstract machine. Data-race-free safe code. Atomics default to sequential consistency (SeqCst), weaker orderings opt-in (recommended)** | Correct by default. Maps well to both ISAs (ARMv8 LDAR/STLR). Full expert control. | SeqCst stores cost a little more on x86. Defaults can hide over-synchronisation, though the perf report can flag it. |
| Ordering always explicit | Fully transparent. | More tokens and boilerplate. Easy to choose too-weak orderings. Conflicts with "safe by default". |
| TSO (x86-like) guaranteed | Easy x86 intuition. | Barriers everywhere on ARM. This is the "x86 holds us back" failure mode. |

## OQ-003 — ISA-specific hardware features in the GC and runtime

| Option | Pros | Cons |
|---|---|---|
| **Portable core with optional accelerators (ARM TBI/MTE/PAC, x86 LAM/AVX-512) (recommended)** | Each ISA plays to its strengths. Neither penalises the other. | More code paths to test. Platform results differ (reported via the perf API). |
| Design around ARM features | Fastest, most elegant ARM runtime. | Constant x86 tax. Contradicts equal priority. |
| Lowest common denominator | One code path. | Leaves measurable performance on the table. |

## OQ-004 — CI merge gating

| Option | Pros | Cons |
|---|---|---|
| **All 5 targets gate every merge (recommended)** | No platform drifts into second-class. Cheap while the compiler is small. | Slower and more costly CI. Windows ARM64 runners are scarcer. |
| 4 gate, `windows-arm64` nightly | Faster and cheaper. | The smallest platform can break for a day and risks becoming de facto Tier 2. |
| Only Linux (both ISAs) gates | Fastest. | macOS and Windows effectively become ports. |

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
Which language is the first AIL compiler written in, before AIL can compile
itself? The custom-tooling policy ([ADR 0009](decisions/0009-custom-tooling-policy.md))
governs AIL's *product*. The bootstrap compiler is scaffolding, and is thrown
away if AIL self-hosts (OQ-011).

| Option | Pros | Cons |
|---|---|---|
| Rust | Fast, memory-safe, strong enums and pattern matching for compilers. | Slow builds. Ownership friction in graph-heavy compiler code. |
| Zig or C | Minimal runtime, and closest to the "own everything" ethos. | Less safety, more effort and more bugs. |
| OCaml | Excellent for type checkers and fast to prototype in. | Far from the final self-hosted compiler. Smaller talent pool. |
| Go | Simple, fast builds, trivial cross-compilation. | Less expressive for ASTs and IR (no sum types). |

*No recommendation yet.* Depends on OQ-011.

## OQ-011 — Self-hosting and roadmap order
Should the compiler eventually be written in AIL (self-hosting), and at which
stage? Should the brief's stage order be changed? See [roadmap.md](roadmap.md).

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
| Option | Pros | Cons |
|---|---|---|
| **AIL's own rich format, plus DWARF and PDB emitters (recommended)** | Works with gdb, lldb, WinDbg, Visual Studio, perf and crash reporters, *and* AIL's own debugger and AI API get full information. | Three formats to maintain. |
| Own format only | Least work. Full control. | No existing debugger, profiler or crash tool can read AIL binaries. Adoption suffers. |

---

## OQ-020 — Correctness oracles for the optimising backend
**Resolved** by [ADR 0019](decisions/0019-miscompilation-defence.md).

## OQ-021 — How do Linux programs that call C libraries get a libc?
[ADR 0020](decisions/0020-linux-no-libc.md) removes libc from AIL's Linux
runtime, but C libraries expect an initialised libc (malloc, errno, TLS,
pthreads). To be settled in the FFI topic. Candidate approaches: load libc
only when FFI is used, with AIL threads made libc-compatible; or AIL provides
its own libc-compatible shim.

---

### Not yet opened
The proposals in [design/ai-author-wishlist.md](design/ai-author-wishlist.md)
will be raised as questions under their respective topics.

The 28 design topics in [status.md](status.md) each still need their own
questions. They will be opened topic by topic during Stage 1.
