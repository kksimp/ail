# Glossary

Shared terminology for AIL. Use these terms consistently in docs, code,
diagnostics, and AI interfaces. Entries are alphabetical.

| Term | Definition |
|---|---|
| **ABI** | Application Binary Interface: calling conventions, data layout, and symbol rules that let separately compiled code interoperate. Windows x64 and the System V ABI (Linux, macOS) differ. |
| **ADR** | Architecture Decision Record: one file per design decision in `docs/decisions/`. See ADR 0001. |
| **Allocation ladder** | The ordered list of allocation strategies the compiler tries per site: eliminate > stack > in-place reuse > inferred arena > task-local heap > shared GC heap. See ADR 0012. |
| **`any Trait`** | A type holding any value whose type implements `Trait`. Calls dispatch dynamically (ADR 0079). |
| **AoS / SoA** | Array of Structs vs. Struct of Arrays: two memory layouts for collections of records. SoA often vectorises and caches better. |
| **Arena (scoped allocation)** | A region where many objects with a shared lifetime are allocated, then freed together in one operation. |
| **Associated function** | A function declared `fn Type.name(...)` without `self`, called `Type.name(...)` (ADR 0082). |
| **Attribute** | `@name` or `@name(args)` metadata placed before a declaration (ADR 0083). |
| **Baseline (ISA)** | The minimum CPU feature level the compiler may assume without runtime checks. See ADR 0005 and ADR 0006. |
| **Bootstrap backend** | An existing code generator (e.g. LLVM) used temporarily, before AIL's custom backend exists. |
| **Coherence** | The guarantee that a type has at most one implementation of a given trait across a whole program (OQ-036). |
| **Colorless tasks** | Concurrency where any function may block and the runtime suspends the task; no async/await marking (ADR 0095). |
| **Compressed reference** | A managed reference stored in fewer than 64 bits (e.g. a 32-bit offset into a heap region). See ADR 0014. |
| **Conformance test** | A test, usually extracted from a spec example, that checks an implementation follows a normative rule. |
| **Constraint (bound)** | A trait a generic type parameter must implement, written inline as `[T Trait]` (ADR 0074). |
| **Copy-on-write (COW)** | Values share storage until one of them is mutated, at which point that one gets its own copy. Unobservable to programs (`own.val.2`). |
| **Delegated decision** | A routine design decision made by the AI using the designer's priority order (ADR 0076). |
| **Drag race** | A head-to-head benchmark: the same small program in AIL, idiomatic C and tuned C, built and run automatically with identical outputs checked (ADR 0030, ADR 0036). |
| **Error return trace** | The list of source locations an error passed through via `?`, recorded automatically and shown in error reports (ADR 0050). |
| **Error set** | The closed union of error types a function can fail with. Inferred from the body for non-public functions (`!T`), explicit at package APIs (`T ! E`). See ADR 0046. |
| **Escape analysis** | Compiler analysis that determines whether a value can outlive or leave its creating scope. A value that does not escape can be stack-allocated. |
| **Exclusivity** | The rule that two `mut` arguments of one call can't overlap. A read-only argument that overlaps a `mut` one is passed as a copy (ADR 0064, ADR 0119). |
| **Hot/cold splitting** | Placing rarely used fields (or code) away from frequently used ones to improve cache use. |
| **In-place reuse** | Reusing a uniquely owned value's memory at its last use for a new value, avoiding the allocator (Perceus-style). See ADR 0012. |
| **INDENT / DEDENT** | Tokens the lexer produces when indentation increases or decreases, giving indentation-based blocks a structure the parser can use (ADR 0033). |
| **ISA** | Instruction Set Architecture, e.g. x86-64 or ARM64 (AArch64). |
| **Keyword** | A reserved word with fixed meaning, such as `fn` or `match`. Keywords can never be used as names (`lex.kw.1`, ADR 0114). |
| **LSE** | Large System Extensions (ARMv8.1+): single-instruction atomic operations (CAS, LDADD, SWP…). Guaranteed by AIL's ARMv8.2-A baseline. |
| **Managed reference** | A reference to an object whose storage is managed by the compiler or runtime, as opposed to a raw pointer. See spec `plat.ptr.3`. |
| **Module** | A directory of `.ail` files sharing one namespace (ADR 0097). |
| **MTE** | Memory Tagging Extension (ARMv8.5+): hardware tags on memory and pointers that detect memory-safety errors. |
| **Mutable value semantics** | The model where every assignment or pass yields an observably independent value, and mutation happens only through owned variables or `mut` parameters (ADR 0060). |
| **Normative** | Part of the binding specification. Contrast with non-normative notes and rationale. |
| **PAC** | Pointer Authentication (ARMv8.3+): cryptographic signatures stored in a pointer's unused bits. |
| **PGO** | Profile-Guided Optimisation: optimising using measurements from real runs. |
| **Pinning** | Preventing the GC from moving an object, e.g. while foreign code holds its address. |
| **Place** | A local binding, parameter or `self`, or a field or element of a place: something that can be assigned to or passed `mut`. It is *mutable* when its root is a `mut` binding, `mut` parameter or `mut self` (`own.place.1`, ADR 0117). |
| **Poisoned lock** | A synchronisation primitive whose holder trapped. Later users get an error instead of possibly inconsistent state (ADR 0047). |
| **Receiver** | The value a method is called on, written `self` (read-only) or `mut self` (mutable) in the method's declaration (ADR 0072). |
| **`ref` cell** | A GC-managed shared mutable cell (`ref T`). The only way two names can observe each other's mutations (ADR 0063). |
| **Region inference** | Compiler inference of arenas (regions) for groups of allocations with a shared lifetime. See ADR 0012. |
| **Rule ID** | A stable identifier for a normative spec rule, formatted `<chapter>.<section>.<n>` (e.g. `plat.ptr.1`). See ADR 0002. |
| **Runtime dispatch** | Choosing among code paths at run time based on detected CPU features. |
| **Safepoint** | A point where a thread can pause for the GC. Polls cost a little. Loops that provably don't allocate can omit them. |
| **Second-class borrow** | Temporary access to an argument that lasts only for a call and can't be stored or returned, so no lifetimes are needed (ADR 0062). |
| **SeqCst** | Sequentially consistent ordering: all threads observe atomic operations in a single global order. |
| **SIMD** | Single Instruction, Multiple Data: vector instructions (NEON and SVE on ARM; SSE, AVX2 and AVX-512 on x86). |
| **Structured concurrency** | Tasks can't outlive the block that spawned them unless placed in an explicit `TaskGroup` (ADR 0096). |
| **SVE / SVE2** | Scalable Vector Extension (ARM): vector instructions whose width is chosen by the hardware. Optional for AIL (ADR 0006). |
| **Target** | An OS + ISA combination the compiler generates code for, e.g. `linux-arm64`. See spec `plat.target.1`. |
| **Task-local heap** | A heap private to one task, collected independently with no synchronisation or write barriers. See ADR 0012. |
| **TBI** | Top Byte Ignore (ARM64): the hardware ignores the top 8 bits of an address, so software can store tags there for free. |
| **Tier 1 target** | A fully supported target: it gates CI, receives full optimisation, and ships official releases. |
| **Trait** | A named set of method signatures that types implement explicitly with `impl Trait for Type` (ADR 0073). |
| **Trap** | The response to a bug (overflow, out-of-bounds access, failed assertion). It stops the current task, runs its deferred cleanup, and is reported to the parent task. In the main task it ends the program (ADR 0035, ADR 0047). |
| **TSO** | Total Store Order: x86's relatively strong memory-ordering model. |
| **Warning** | A diagnostic that doesn't stop a program from building or running, such as a `_` arm that hides enum variants (`expr.match.3`). Errors stop it. |
| **Weak memory model** | A model (as on ARM) in which loads and stores can be observed out of order unless barriers or ordered atomics are used. |
| **Whole-program optimisation** | Optimising across all modules at once (specialisation, devirtualisation, cross-module constant propagation). See OQ-017. |
| **Write barrier** | Code the GC requires on certain pointer stores so it can track references between heap regions or generations. |
| **x86-64-v3** | x86-64 microarchitecture level including AVX2, FMA and BMI1/2. AIL's x86 baseline (ADR 0005). |
