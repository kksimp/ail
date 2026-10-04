# Glossary

Shared terminology for AIL. Use these terms consistently in docs, code,
diagnostics, and AI interfaces. Entries are alphabetical.

| Term | Definition |
|---|---|
| **ABI** | Application Binary Interface: calling conventions, data layout, and symbol rules that let separately compiled code interoperate. Windows x64 and the System V ABI (Linux, macOS) differ. |
| **ADR** | Architecture Decision Record: one file per design decision in `docs/decisions/`. See ADR 0001. |
| **Arena (scoped allocation)** | A region where many objects with a shared lifetime are allocated, then freed together in one operation. |
| **Baseline (ISA)** | The minimum CPU feature level the compiler may assume without runtime checks. See ADR 0005 and ADR 0006. |
| **Bootstrap backend** | An existing code generator (e.g. LLVM) used temporarily, before AIL's custom backend exists. |
| **Conformance test** | A test, usually extracted from a spec example, that checks an implementation follows a normative rule. |
| **Escape analysis** | Compiler analysis that determines whether a value can outlive or leave its creating scope. A value that does not escape can be stack-allocated. |
| **ISA** | Instruction Set Architecture, e.g. x86-64 or ARM64 (AArch64). |
| **LSE** | Large System Extensions (ARMv8.1+): single-instruction atomic operations (CAS, LDADD, SWP…). Guaranteed by AIL's ARMv8.2-A baseline. |
| **MTE** | Memory Tagging Extension (ARMv8.5+): hardware tags on memory and pointers that detect memory-safety errors. |
| **Normative** | Part of the binding specification. Contrast with non-normative notes and rationale. |
| **PAC** | Pointer Authentication (ARMv8.3+): cryptographic signatures stored in a pointer's unused bits. |
| **Pinning** | Preventing the GC from moving an object, e.g. while foreign code holds its address. |
| **Rule ID** | A stable identifier for a normative spec rule, formatted `<chapter>.<section>.<n>` (e.g. `plat.ptr.1`). See ADR 0002. |
| **Runtime dispatch** | Choosing among code paths at run time based on detected CPU features. |
| **SeqCst** | Sequentially consistent ordering: all threads observe atomic operations in a single global order. |
| **SIMD** | Single Instruction, Multiple Data: vector instructions (NEON and SVE on ARM; SSE, AVX2 and AVX-512 on x86). |
| **SVE / SVE2** | Scalable Vector Extension (ARM): vector instructions whose width is chosen by the hardware. Optional for AIL (ADR 0006). |
| **Target** | An OS + ISA combination the compiler generates code for, e.g. `linux-arm64`. See spec `plat.target.1`. |
| **TBI** | Top Byte Ignore (ARM64): the hardware ignores the top 8 bits of an address, so software can store tags there for free. |
| **Tier 1 target** | A fully supported target: it gates CI, receives full optimisation, and ships official releases. |
| **TSO** | Total Store Order: x86's relatively strong memory-ordering model. |
| **Weak memory model** | A model (as on ARM) in which loads and stores can be observed out of order unless barriers or ordered atomics are used. |
| **x86-64-v3** | x86-64 microarchitecture level including AVX2, FMA and BMI1/2. AIL's x86 baseline (ADR 0005). |
