# 0388. ARM64 code generation: AAPCS64, the same allocator, checks through the flags

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.build.1, native.build.3](../spec/23-native.md), expr.arith.1, expr.arith.3, expr.arith.5, expr.bit.2, type.coll.4, type.num.9
- **Builds on:** [ADR 0385](0385-x86-64-code-generation.md) (x86-64), [ADR 0386](0386-linear-scan-register-allocation.md) (allocator),
  [ADR 0006](0006-arm64-baseline-v8-2.md) (ARMv8.2-A), [ADR 0008](0008-isa-priority-policy.md) (lockstep)

## Context

Lane 120's third slice targets ARM64 (macOS, and Linux for testing) in
lockstep with x86-64 (ADR 0008). The encoder is AIL
(`compiler/a64/asm.ail`, ADR 0010); the MIR, the allocator, the runtime and
the trap protocol are shared.

## Decision

1. **Convention.** AAPCS64 as it stands: x0–x7 and d0–d7 for arguments,
   x19–x28 and d8–d15 callee-saved. x18 is never touched (the platform
   register on macOS).
2. **Registers.** x16, x17, x8, d30 and d31 are the code generator's scratch;
   x0 is the return register and scratch between calls. The allocator gets
   x9–x15 and x1–x7 (caller-saved), x19–x28 (callee-saved), d16–d29 and
   d0–d7, d8–d15.
3. **Frame.** `stp x29, x30` then one `sub sp`; saved registers and spill
   slots are at fixed positive offsets from `sp`, so every access is one
   scaled `ldr`/`str`.
4. **Checks.** Overflow uses the flag-setting forms (`adds`, `subs`, `negs`)
   and `b.vs`; a product checks `smulh` against the sign of the low half.
   `sdiv` doesn't trap, so a zero divisor is a `cbz` to the trap, and
   `MIN / -1` is caught on a `negs`; `msub` gives `MIN % -1 = 0` by itself.
   A float to `int` conversion saturates, so it is checked by converting
   back and comparing with the truncated value, and by excluding the
   largest `int`, which no float converts to exactly. All of them branch to
   the same trap stubs as on x86-64.
5. **Addresses.** Data and globals are reached with `adrp` and `add`, so
   code is position-independent (macOS loads executables at a random slide).
   Float constants are built in x8 with `movz`/`movk` and moved with `fmov`.
6. **Testing.** `bootstrap/tests/native.rs` builds Linux ARM64 executables
   and runs them under qemu's user-mode emulator where it is installed;
   macOS ARM64 runs them natively in the designer's gate (ADR 0085).
   The encoder was checked against clang's assembler on every instruction
   it emits (clang is allowed for differential testing, ADR 0010).

## Options considered

- **Frame addressed from x29 with negative offsets.** The unscaled
  `ldur`/`stur` reach only 256 bytes; positive `sp` offsets reach 32 KiB.
- **A literal pool for float constants.** One load instead of up to four
  moves, but needs a pool per function and `ldr` literal relocations;
  rejected until a measurement shows constant loads in a hot loop.
- **Checked conversion through the FPSR exception flags.** Exact, but needs
  reading and clearing FPSR around each conversion, which is slower than
  the round trip.

## Consequences

- One `ail build --target aarch64-linux` path exists for testing on any
  machine with qemu; it is also usable on Linux ARM64 hosts.
- The code generator is `compiler/a64/gen.ail`, about the size of the
  x86-64 one; anything else target-specific is in the encoder.
