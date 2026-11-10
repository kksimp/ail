# 0968. Native: the aarch64-windows target (windows-arm64)

- **Status:** Deferred past 0.1.0 (experimental: compile-only, unverified)
- **Date:** 2026-11-05
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 24 — Cross-platform
- **Spec rules:** plat.isa.2, native.build (new rule for the target)
- **Builds on:** [ADR 0031](0031-hosts-and-cross-compilation.md), [ADR 0388](0388-arm64-code-generation.md), [ADR 0597](0597-pe-executables-for-windows.md), [ADR 0598](0598-windows-imports-and-system-calls.md), [ADR 0925](0925-windows-release-gate.md)

## Context

`windows-arm64` is Tier 1 in spec 01 and ADR 0031 but `ail build` cannot
make it. Kaleb's targets are arm64 and x86-64 only (2026-11-05); this is the
one missing combination.

## Decision

1. `--target aarch64-windows` = `a64.generate(m, a64.WINDOWS)` plus the PE
   writer with machine `0xAA64`. `a64.WINDOWS` is 3, so `target()` answers 3
   as on x86-64 Windows and the runtime layer (`win*.ail`) is shared.
2. Windows-specific ARM64 code (entry, import calls, stack probes) lives in
   its own file `compiler/a64/win_call.ail` (ADR 0823); `a64/gen.ail` gets at
   most one dispatch line.
3. PE relocation patching is shared with `obj/link.ail` (`b26`, `adrp`,
   `add12`) instead of a second copy.
4. x18 is never allocated or written (already true).
5. Work items and staging: [design/windows-arm64.md](../design/windows-arm64.md).

## Test platform (ADR 0925)

**Experimental and unverified (Kaleb, 2026-11-05).** Until it is verified it is excluded from the ADR 0925 release requirement, and no seed or release notes claim it is tested; `ail build --target` help marks it experimental. There is no Windows ARM64 hardware, and
no VM will be set up (no space). ail-win is x64 and cannot run ARM64 PE. So
the target is compile-only: `ail build --target aarch64-windows` writes a PE
that has never run. It is experimental and unreleased, and no release may list
it until the `native`, `examples` and `run` suites are green on real Windows
ARM64 or an emulator (ADR 0925). A Windows 11 ARM64 VM on the designer's Apple
Silicon Mac is the cheapest route if space appears.

## Rejected alternatives

- **x64 build under Windows' x64 emulation or ARM64EC:** not native, and hides
  ARM weak-ordering bugs.
- **A second Windows runtime for ARM64:** the runtime is ISA-independent via
  `target()`; a copy would drift.
- **A hosted ARM64 Windows runner:** no hosted CI (ADR 0085).
