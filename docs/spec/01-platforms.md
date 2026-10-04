# 01. Platforms and targets

Chapter key: `plat`

> **Status:** Drafted (partial). The target tier policy is pending
> [ADR 0008](../decisions/0008-isa-priority-policy.md).

## Targets

**plat.target.1** — A conforming implementation's Tier 1 targets are:

| Target ID | OS | ISA | Object format |
|---|---|---|---|
| `linux-x64` | Linux | x86-64 | ELF |
| `linux-arm64` | Linux | ARM64 | ELF |
| `windows-x64` | Windows | x86-64 | PE/COFF |
| `windows-arm64` | Windows | ARM64 | PE/COFF |
| `macos-arm64` | macOS | ARM64 | Mach-O |

There is no `macos-x64` target. *(ADR 0004)*

> **Note:** Target ID spelling is provisional and will be fixed when the build
> system is designed.

## Instruction-set baselines

**plat.isa.1** — Code generated for an x86-64 target MAY assume the
x86-64-v3 feature level. It MUST NOT assume any feature beyond x86-64-v3
unless that code is reached only through runtime feature dispatch or the
program was compiled with an explicit opt-in target flag. *(ADR 0005)*

**plat.isa.2** — Code generated for an ARM64 target MAY assume the
ARMv8.2-A feature level (including LSE atomics). It MUST NOT assume any
feature beyond ARMv8.2-A unless that code is reached only through runtime
feature dispatch or the program was compiled with an explicit opt-in target
flag. *(ADR 0006)*

**plat.isa.3** — A program started on hardware below its target's baseline
MUST terminate with a diagnostic naming the missing CPU feature level. It
MUST NOT run into undefined behavior first. *(ADR 0005, ADR 0006)*

> **Rationale:** A clear message ("this CPU lacks x86-64-v3") is far easier
> for users and AI agents to act on than an illegal-instruction crash.

## Pointer width

**plat.ptr.1** — Addresses and raw pointers are 64 bits wide on every target.
*(ADR 0007; refined by ADR 0014 to apply to raw pointers only)*

**plat.ptr.2** — The size and index integer type (name to be decided in the
Types chapter) is exactly 64 bits on every target. Its width is not
target-dependent. *(ADR 0007)*

**plat.ptr.3** — The in-memory representation of managed references
(references to objects whose storage is managed by the compiler or runtime)
is implementation-defined, and MAY be narrower than 64 bits. Safe code MUST
NOT be able to observe it. Converting a managed reference to a raw pointer
MUST be explicit, and is available only in `unsafe` code and at the FFI
boundary. *(ADR 0014)*

> **Rationale:** compressed references can shrink pointer-dense data by up
> to half and improve cache behaviour. See
> [performance-beyond-c.md §3.2](../design/performance-beyond-c.md#32-compressed-managed-references).

## Byte order

**plat.endian.1** — Every target is little-endian. The in-memory byte order
of multi-byte scalar values is little-endian, and programs MAY rely on this.
*(ADR 0007)*

> **Note:** Explicit-endianness conversion for file and network formats is
> provided by the standard library, not by the language.
