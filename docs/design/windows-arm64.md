# windows-arm64 target: design and gap list

Status: proposed, **unverified** (2026-11-05). No test platform and no VM (Kaleb, no space): compile-only stage done, runtime stages not started. Lane E (TARGETS). Kaleb ruled out RISC-V and all
32-bit targets (2026-11-05), so the only missing Tier 1 target is
`windows-arm64` (spec 01-platforms, ADR 0031). `x86_64-macos` stays out
(ADR 0004). ADR number: to be allocated by the integrator.

## Shape

`aarch64-windows` = the existing ARM64 generator (`compiler/a64/gen.ail`) +
the existing PE writer (`compiler/obj/pe.ail`) + the existing Windows runtime
layer (`compiler/rt/native/win*.ail`, selected by `target() == 3`). No new
instruction encoder and no new runtime layer. The work is the join.

`a64.generate(m, os)` gets `a64.WINDOWS := 3`, the same value x64 uses, so
every `target() == 3` branch in the runtime (path, heap, net, tz, tls) applies
unchanged. `target()` then answers the OS, as it does today; the ISA never
reaches the runtime.

## Work items

| # | Item | Where | Notes |
|---|---|---|---|
| 1 | `--target aarch64-windows`, `host_target()` for Windows on ARM, `exe_ext`, cache key | `compiler/main.ail` | `uname` is not used on Windows; read `PROCESSOR_ARCHITECTURE` (`ARM64`) |
| 2 | Entry | `a64.generate` | Loader calls the entry as a C function: no argc/argv on the stack. Pass empty argv like x64 `generate_windows` (zeroed argv/envp above main, which is also the GC stack top); end with `ExitProcess(x0)` via the import slot instead of `rt.exit` -> syscall |
| 3 | `libsys.NAME` calls | `a64.gen` libsys branch | Today: `adr` + `ldr` of the `imp:` slot + `blr`; Linux/macOS filters by name. Windows: no name filter, DLL grouping by `pe_dll_of`. Windows ARM64 passes args in x0-x7 / d0-d7 with the same stack-argument rule as AAPCS64, so the existing argument moves apply. No shadow space |
| 4 | Relocation kinds in PE | `obj/pe.ail` `pe_resolve` | Accepts only `rel32` today. Needs `b26`, `adrp`, `add12`, and the import-slot reach (adrp+ldr), shared with `obj/link.ail`; factor the patching into one function |
| 5 | PE header | `obj/pe.ail` | Machine `0xAA64` (43620), `IMAGE_FILE_LARGE_ADDRESS_AWARE`, 4 KiB pages kept (Windows ARM64 uses 4 KiB). Section and import layout unchanged |
| 6 | Reserved registers | `a64.registers()` | x18 already excluded (macOS). On Windows x18 is the TEB: also never write it, and x29 stays the frame chain. Confirm no kernel uses x18 |
| 7 | Stack probing | `a64.gen` prologue | Windows commits stack on guard-page touches. Frames over 4 KiB must touch each page in order. x64 Windows has no probe either (verify, may be a latent bug for both): emit a probe loop for frames over 4 KiB, or rely on the pool stack that AIL threads already allocate |
| 8 | `syscall`, `clone` prims | `a64.gen` | `svc` is not allowed on Windows. Runtime must route to `win.ail` (`import_call`) wherever `target() == 3`, already true for x64; audit the prims that a64 emits directly (`clone` -> `CreateThread`, futex -> `WaitOnAddress`/`WakeByAddress*`) |
| 9 | Atomics, SIMD kernels, AES | `a64/kernels*.ail` | Reused as is. LSE (ARMv8.1) is present on every Windows 11 ARM64 device. CPU-feature dispatch (plat.isa.3) must use `IsProcessorFeaturePresent` instead of Linux auxv/`mrs` |
| 10 | Icache | n/a | No JIT, no cost. Note for later: `FlushInstructionCache` if a JIT ever lands |
| 11 | Unwinding / SEH | none | AIL traps use their own table (`trace.tab`, `codegen.add_call_table`). No `.pdata`; a debugger cannot unwind AIL frames until DEBUGINFO lane emits CodeView/PDB and ARM64 `.pdata` |
| 12 | Gate wiring | `tools/gate`, ADR 0925 | `native`, `examples`, `run` suites must be green on a Windows ARM64 machine before a release, same rule as x64 (ADR 0925); lockstep (ADR 0008) means no release without it |
| 13 | Docs | spec 01, 23, status, roadmap, native-abi, glossary | Add `native.build` rule for `aarch64-windows`; ABI table row in `design/native-abi.md` §1 |

## Machine-readable diagnostics (AI-first)

- An unsupported combination reports `error[native.build.3]` with the list of
  supported targets, as JSON when `--json` is given (field `supported`).
- `ail build --target aarch64-windows` on any host works (ADR 0031): the
  compiler cross-builds, no SDK.
- A program started on a CPU below ARMv8.2-A exits with the existing
  plat.isa.3 message naming the missing feature.

## Testing

All builds and tests run on Kaleb's machines. There is no Windows ARM64
hardware among them (ail-win is x64, and x64 Windows cannot run ARM64 PE).
Options, recommended first:

1. **Windows 11 ARM64 VM on the Mac** (UTM or Parallels, Apple Silicon host).
   Microsoft publishes a free Windows 11 ARM64 evaluation/Insider VHDX; no
   licence purchase needed for testing, only the download. Runs natively on
   the Mac, so it is fast, and the Mac gate harness can drive it over SSH.
   Needs Kaleb's OK for the download and a VM on his Mac.
2. QEMU system emulation (aarch64, TCG) on ail-linux: same image, very slow
   on a 4 GB VM with an i5-7500T. Fallback only.
3. A physical Windows on ARM device: costs money, needs Kaleb's decision.

Until one exists, steps 1-5 can be written and checked by building
(`--target aarch64-windows` produces a PE; a parser check of the headers
and imports runs on Linux without executing it), as `pe_sample` is today.

## Staging

1. Build-only: items 1, 4, 5, 13 + `win_sample` for ARM64; header check on Linux.
2. Hello world on a VM: items 2, 3, 6.
3. Full runtime: items 7, 8, 9 and the `native`/`examples`/`run` suites.
4. Release rule: add to the ADR 0925 pre-release checklist.

## Rejected alternatives

- **ARM64EC / running the x64 build under Windows' x64 emulation.** Not a
  native target; slower, and hides ARM weak-ordering bugs (ADR 0008).
- **A separate `rt/native/winarm.ail`.** The runtime is ISA-independent
  through `target()`; a copy would drift.
- **Waiting for hardware before any code.** Build-only stage needs none.
