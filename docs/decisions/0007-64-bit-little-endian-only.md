# 0007. AIL targets 64-bit little-endian machines only

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 3 — Type system; 5 — Memory model; 24 — Cross-platform ABI strategy
- **Spec rules:** [plat.ptr.1, plat.ptr.2, plat.endian.1](../spec/01-platforms.md)
- **Refined by:** [ADR 0014](0014-compressed-managed-references.md): the 64-bit rule applies to raw pointers; managed reference representation is implementation-defined

## Context

If the specification has to allow other pointer widths or byte orders, it
needs target-dependent integer sizes, portability caveats on data layout,
and endian-conversion rules. Every in-scope target (x86-64, ARM64 on
Linux/Windows/macOS) is 64-bit and little-endian. The question was which
future targets the language design should stay open to.

## Options considered

### Option A — 64-bit native only; no future target is guaranteed (chosen)
- **Pros:** The simplest spec. Pointer width and `usize`-equivalent sizes are
  fixed at 64 bits. Little-endian layout can be guaranteed. Simpler GC,
  atomics and layout rules.
- **Cons:** Adding web or embedded targets later may need a spec revision.

### Option B — Stay open to RISC-V 64
- **Pros:** A growing open ISA.
- **Cons:** None for the language itself. RV64 is 64-bit little-endian, so
  this decision doesn't block it (see Consequences).

### Option C — Stay open to WebAssembly (wasm64)
- **Pros:** Browsers and sandboxed plugins.
- **Cons:** Its GC and stack-scanning model conflicts with a custom runtime,
  and threading is limited.

### Option D — Stay open to 32-bit (wasm32, ARMv7)
- **Pros:** The widest embedded and web reach.
- **Cons:** Breaks the fixed 64-bit pointer assumption everywhere in the spec.

## Decision

The AIL specification assumes a **64-bit address space and little-endian byte
order**. No targets other than the current five are planned or designed for.

## Rationale

This removes a whole category of portability rules and ambiguity, for both
humans and AI. All Tier 1 targets satisfy it.

## Consequences

- Spec rules `plat.ptr.*` and `plat.endian.*` are normative and unconditional.
- Any 64-bit little-endian ISA (e.g. RISC-V RV64) could still be added later
  without changing the language. Only toolchain work would be needed.
- 32-bit and big-endian targets would require a new ADR that supersedes this one.
- Explicit-endianness APIs remain necessary in the stdlib for file and network
  formats. That's a library concern, not a language one.

## Rejected alternatives

- **wasm32 / 32-bit:** too much spec complexity for markets not in scope.
- **Designing for wasm64 now:** its runtime model conflicts with the custom GC.
  Revisit later if web deployment becomes a goal.
