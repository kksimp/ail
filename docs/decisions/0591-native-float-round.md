# 0591. The native backend lowers `x.round()` to a runtime function

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 11 — Native backend (ADRs 0384–0393); standard float methods (`type.num.9`)
- **Spec rules:** `type.num.9` unchanged; `native.subset.1` lists `round` (amended)
- **Extends:** [ADR 0384](0384-mir-shape-and-native-subset.md), [ADR 0387](0387-static-executables-and-ail-build.md)

## Context

`type.num.9` already gives `x.round()` (halves away from zero, sign of zero kept).
The native subset had `abs`, `floor`, `ceil` and `sqrt` but no `round`, so a
program that used it could not build natively. Both x86-64 and AArch64 have
no single instruction for half-away-from-zero rounding (`roundsd` and `frint*`
round half to even, or only truncate in the other directions).

## Decision

`x.round()` lowers to `call rt.float_round`, a function in
`compiler/rt/native/math.ail` (linked into every executable under the runtime
exception). It is written with `floor` and `ceil`:

    t := if x >= 0.0 then floor(x) else ceil(x)   # truncation, exact
    d := x - t                                     # exact
    if d >= 0.5 then t + 1.0; if d <= -0.5 then t - 1.0; else t

Every step is exact, so the result agrees with `type.num.9` on signed zero,
NaN and the infinities. No backend encoder or MIR op is added, so the x86-64
and AArch64 generators are unchanged.

## Options considered

- **Option A — a runtime function over `floor`/`ceil` (chosen).** No new MIR
  op or encoder, so no change to two backends; one function, testable with the
  existing `executables_match` harness.
- **Option B — a new `round` MIR op with an encoder per ISA.** Faster in
  principle, but it needs an encoder sequence for each target, and the
  sequence is `trunc` plus a compare and select anyway. Rejected for now: the
  call is cheap next to the output it feeds, and the backend's op set is
  lane 120's.
- **Option C — `roundsd` mode 0 (ties to even) and a fix-up.** Rejected: it
  needs the same compare and select as A and is harder to check by reading.

## Consequences

- `native.subset.1` now lists `round`.
- Tests: `bootstrap/tests/native/run/math_round.ail` (built and compared with
  `ail run`, on x86-64 here; AArch64 under qemu is pending the emulator) and
  `bootstrap/tests/native/round.mir`.
- The runtime's file list in `compiler/main.ail` now includes `math.ail`.
