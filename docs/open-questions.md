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
[ADR 0010](decisions/0010-no-llvm-backend.md) bans LLVM and other external
backends. Emitting C and compiling it with clang or gcc would be a backend crutch too.

| Option | Pros | Cons |
|---|---|---|
| **No: own backend only (recommended, consistent with ADR 0010)** | No external influence on the IR. No crutch. | No native binaries until the custom backend works. |
| Yes, for running tests only, never shipped | Earlier native test runs. | Still a crutch. C semantics (undefined behaviour) leak into the test results. |

## OQ-013 — Strategy for the first native backend
| Option | Pros | Cons |
|---|---|---|
| **A simple, correct code generator first (linear-scan register allocation, minimal optimisation) on a shared machine-IR framework, then optimisation passes added (recommended)** | Earliest native binaries. Every later optimisation is measurable against a baseline. | Early code is slow (published honestly). |
| An optimising backend from the start (SSA, graph-colouring or SSA-based allocation) | Better code sooner after it lands. No rewrite. | Much later first binary. Harder to debug. |

## OQ-014 — Linux runtime without libc?
See [design/toolchain-constraints.md](design/toolchain-constraints.md).

| Option | Pros | Cons |
|---|---|---|
| **No libc on Linux; raw syscalls (recommended)** | Full control, static binaries, and no glibc version problems. Fits ADR 0009. | Must reimplement thread creation, TLS setup, DNS resolution and locale behaviour, and handle `vDSO` (the kernel-provided time functions). FFI to C libraries then needs a libc-compatible mode. |
| Link libc on Linux | Easier. Mixing with C libraries is easier. | Inconsistent with macOS and Windows minimal-dependency stance. glibc versioning problems. |

## OQ-015 — Default data layout
See [design/performance-beyond-c.md §3.1](design/performance-beyond-c.md#31-compiler-chosen-data-layout).

| Option | Pros | Cons |
|---|---|---|
| **Compiler-chosen by default, explicit fixed-layout opt-in, always reported (recommended)** | Enables reordering, hot/cold splitting and struct-of-arrays, which C can't do. | Layout isn't obvious from the source (the report is required). FFI types need annotations. |
| Declaration order (like C) by default, optimisations opt-in | Predictable. Simple FFI. | Leaves performance that C can't reach on the table. |

## OQ-016 — Compressed managed references
See [design/performance-beyond-c.md §3.2](design/performance-beyond-c.md#32-compressed-managed-references).
Would refine spec rule `plat.ptr.1`: raw pointers stay 64-bit, managed
reference representation becomes implementation-defined.

| Option | Pros | Cons |
|---|---|---|
| **Allow it (representation implementation-defined; raw pointers stay 64-bit) (recommended)** | Pointer-dense data can shrink by up to half, improving cache use. | Decode cost (one shift and add). A heap size cap per compressed region. More complex FFI. |
| Always 64-bit references | Simple. Unlimited heap. | Wastes cache in pointer-heavy code. |

## OQ-017 — Whole-program optimisation by default
| Option | Pros | Cons |
|---|---|---|
| **Whole-program in release, incremental in debug, with whole-program facts cached (recommended)** | Maximum optimisation where it matters. Fast edit loop. | Debug and release can behave differently in performance (not semantics). Complex build cache. |
| Always separate compilation | Simple and fast. | Gives up devirtualisation, specialisation and cross-module constant propagation. |
| Always whole-program | One mode. | Slow edit loop on large projects. |

## OQ-018 — Which memory-management R&D tracks to pursue
See [design/performance-beyond-c.md §5](design/performance-beyond-c.md#5-suggested-rd-tracks-for-the-designer-to-approve-or-reject).
Allocation ladder, task-local heaps, compile-time reuse, region inference.

## OQ-019 — Debug-info formats
| Option | Pros | Cons |
|---|---|---|
| **AIL's own rich format, plus DWARF and PDB emitters (recommended)** | Works with gdb, lldb, WinDbg, Visual Studio, perf and crash reporters, *and* AIL's own debugger and AI API get full information. | Three formats to maintain. |
| Own format only | Least work. Full control. | No existing debugger, profiler or crash tool can read AIL binaries. Adoption suffers. |

---

### Not yet opened
The 28 design topics in [status.md](status.md) each still need their own
questions. They will be opened topic by topic during Stage 1.
