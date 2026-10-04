# Feasibility: is AIL achievable, and how good can it get?

> **Non-normative design note.** These are an architect's early estimates for
> setting expectations, written 2026-10-04 before any design or measurement.
> Every number here is a hypothesis to be replaced by benchmark results
> ([OQ-007](../open-questions.md#oq-007--token-efficiency-benchmark-methodology),
> [OQ-008](../open-questions.md#oq-008--performance-benchmark-methodology)).

## 1. Is it possible?

**Yes, technically.** Every major component has precedent:

| Ambition | Existence proof |
|---|---|
| Native, GC'd, fast | Go, OCaml, C#/.NET NativeAOT, D |
| Escape analysis → stack allocation | Java HotSpot, Go |
| Custom backend instead of LLVM | Go, Zig (self-hosted x86-64 backend), Cranelift |
| Custom GC + runtime on 3 OSes | Go, .NET, OCaml 5 |
| Value-based errors with `?` | Rust, Swift, Zig |
| Rich compiler query API | Roslyn (C#), rust-analyzer, TypeScript language service |
| Canonical formatter | gofmt, zig fmt |

What makes AIL new is the **combination**, plus treating AI as a design
constraint. That is a large scope, not an impossible one.

**The real risk is scope and time, not feasibility.** For comparison, Go,
Rust and Zig each took many years (often with funded teams) to reach
production quality, and fully custom backends lag behind LLVM for a long
time. The plan must deliver something usable at every stage. The roadmap
refinements in [roadmap.md](../roadmap.md) are aimed at that.

## 2. Token efficiency: how much better?

Token cost has **three layers**. Syntax is the smallest of them.

### Layer 1 — Syntax (tokens to write a given program)
Informal public comparisons of the same tasks across languages show roughly a
2–3x spread between the most and least token-efficient mainstream languages.
Java, C++ and C# sit at the verbose end. Python, Haskell, F# and Clojure sit
at the terse end.

**Estimated target for AIL:**

| Compared with | Fewer tokens for equivalent programs |
|---|---|
| Java, C++, C# | ~40–55% |
| Rust | ~30–45% |
| Go, C | ~25–40% |
| Python | ~0–15%, while carrying full static types and performance information Python lacks |

**Important constraint: the tokenizer is not ours.** Model tokenizers were
trained on existing code. Syntax built from common English words and familiar
punctuation tokenizes well. Novel symbols or unusual operators can cost
*several tokens each* and make things worse. AIL's syntax choices must be
checked against real tokenizers, not intuition.

### Layer 2 — Context (tokens to understand a program before changing it)
This is where the **big wins** are. Today an AI reads whole files and greps
to work out types, callers and effects. With the AI semantic API it can ask
structured questions ("callers of X", "does f allocate", "type of this
expression") and receive compact answers. For understand-and-modify tasks on
real projects, a **several-fold (≈3–10x) reduction in context** is plausible.
That is far more than syntax can deliver.

### Layer 3 — Repair loops (tokens wasted on failed attempts)
Structured diagnostics with spec rule IDs and machine-applicable fixes,
canonical formatting (no style churn in diffs), and structural edits (no
whole-file regeneration) cut both the number of iterations and the tokens
per iteration.

### Bottom line
- Per-line syntax: **~25–50% fewer tokens** than mainstream statically-typed
  languages.
- Whole-task cost (write + understand + modify + repair): a **2–5x
  improvement** is a realistic stretch goal, mostly from layers 2 and 3.

### The biggest risk: AI has never seen AIL
Current models were trained on huge amounts of Python, C, Rust and Go, and
on zero AIL. A brand-new language starts with *worse* AI fluency than a
mainstream one, which costs tokens in errors. Mitigations, to be raised as
design questions:
1. Keep the grammar small, regular and close to familiar conventions where
   familiarity costs nothing.
2. Maintain a compact **"language card"**: the whole language summarised in a
   few thousand tokens that fits in any AI context window.
3. Make diagnostics so precise that the first repair attempt usually works.
4. Treat the formatter and semantic API as the way an AI "learns" a codebase
   at runtime, instead of relying on training data.

## 3. Performance: how close to C/assembly?

### Estimates by workload (vs. optimised C, clang/gcc -O2/-O3)

| Workload | Estimate | Why |
|---|---|---|
| Numeric and SIMD kernels, tight loops, no allocation | **~0–10%**, sometimes faster | Same machine model as C. Stronger aliasing information than C (no `restrict` needed) helps vectorisation. |
| Generic code (sorting, containers) | **Parity or faster** | Monomorphised generics beat C's `void*` callbacks (e.g. `qsort`). |
| Typical systems and server applications | **~1.0–1.3x** | Bounds and overflow checks (mostly eliminated), GC barriers, safepoint polls. |
| Allocation-heavy idiomatic code | **~1.2–2x** | GC cost, which depends on how much escape analysis and arenas catch. |
| Hand-tuned assembly kernels | Reachable only via SIMD intrinsics / `unsafe` | No compiler consistently beats expert hand-written assembly on narrow kernels. That expectation should be stated honestly. |

### Where AIL can beat C
- **Aliasing:** the type system can prove non-aliasing, which C can't
  without `restrict`.
- **Whole-program optimisation and devirtualisation** by default.
- **Monomorphised generics** instead of function-pointer callbacks.
- **PGO as a first-class build mode.**
- **Layout control** (struct-of-arrays and similar) available as a language
  feature rather than a manual rewrite.

### Where AIL pays a cost relative to C
- **GC write barriers:** typically a small single-digit-percent overhead on
  pointer-heavy code.
- **Safepoints and stack maps:** limit some register allocation freedom.
- **Safety checks:** bounds and possibly overflow checks
  ([OQ-009](../open-questions.md#oq-009--integer-overflow-semantics)) where
  the compiler can't prove them away.
- **Memory footprint:** GC heaps need headroom.

### The custom-backend reality
LLVM and GCC embody decades of optimisation work. A custom backend will
almost certainly produce **slower code than LLVM for years**. Cranelift-class
backends, which favour compile speed, typically produce code noticeably slower
than LLVM -O3. Recommendation (roadmap refinement #3): keep the bootstrap
backend as a **permanent benchmark reference**, and make the custom backend
the default only once it meets a parity threshold that the designer sets.

### Bottom line
**C-class performance is realistic for code written with performance in mind**
(no hot-path allocation, which the compiler and allocation reports help
enforce). **Typical application code can plausibly land within 1.0–1.3x of
C.** Matching hand-written assembly is achievable only through explicit
low-level escape hatches, not automatically, as the brief itself states.
