# ADR 0842: Gate on the designer's Mac, no Linux VM

- Status: Accepted (amends [ADR 0085](0085-interim-local-testing-gate.md)); amended by [ADR 0865](0865-no-double-work.md) and [ADR 0925](0925-windows-release-gate.md)
- Decided by: Designer (2026-10-21 18:43 and 2026-10-22 13:10: "default to testing on my mac and not Linux", "no more in your VM with Linux")
- Date: 2026-10-22

## Context

ADR 0085 had the AI run `./ail selftest` on Linux x86-64 and the designer run
`tools/verify.sh` on macOS ARM64. Linux runs cost tokens (long suites, repeated
fixes), and the Mac is where the first-class ARM64 target runs. A Mac-only run
found a bug the Linux suite could not: the Windows runtime arms became dyld
imports in every macOS binary (all native programs aborted at load).

## Decision

- **The gate is the Mac.** One command from the repository root:
  `tools/gate/mac-gate.sh`. It runs the full selftest with `--no-fail-fast`, the
  native suites, and starts one native binary; it prints one line per failure
  and ends with `GATE GREEN` or `GATE RED`.
- The integrator may run cheap mechanical checks locally (`ail check compiler`,
  `ail fmt --check`, doc checks, token budgets), because they cost little and
  catch merge damage early. No Linux VM runs of the suites, fuzz, x86-64 native,
  qemu or callgrind.
- Lanes do not open their own Mac sessions; the integrator carries each chain
  through one Mac run. A change to a runtime arm for one target (Windows, Linux)
  is built for `aarch64-macos` and one binary is started on the Mac before merge.
- The x86-64 half of strict lockstep (ADR 0008) is checked by the Mac's x86-64
  encoder tests (`native_asm`, objdump) and the Linux CI the designer may add
  later. x86-64 callgrind figures in reports are historical until then.
- Windows stays best-effort (Wine, when the designer asks).

## Consequences

- Fuzz including `#523`, bcprogs and the callgrind regression bar are not run
  routinely; the day-3 reports list them as unverified.
- Benchmarks that need valgrind use wall time on the Mac (`tools/perfbench`).

## Rejected alternatives

- Keep the Linux gate as well: doubles token cost; Linux-only checks found
  nothing the Mac did not in chains J to O, while the Mac found a blocker.
- Hosted CI: still out of scope (ADR 0085).
