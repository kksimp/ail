# 0592. Transcendental float methods, with a one-ulp bound

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** Designer (Kaleb), choosing option C of OQ-118 on 2026-10-21
- **Topic:** 11 — Standard library (float methods); native backend (ADRs 0384–0393)
- **Spec rules:** `type.num.16` (new); `native.subset.1` lists the new methods (amended)
- **Resolves:** [OQ-118](../open-questions.md)
- **Extends:** [ADR 0591](0591-native-float-round.md), [ADR 0130](0130-floats.md)

## Context

The language had `abs`, `floor`, `ceil`, `round` and `sqrt` on `float`, and no
other transcendental function. `std.tensor`'s `sigmoid` and `tanh` (ADR 0507)
used the platform's `exp` and `tanh` on the interpreter only. The native
backend has no C library, so it can't call libm.

Two facts bound the design. Platform libms differ from each other in the last
bit, so no implementation can match them all bit for bit. And the native
runtime is a standalone AIL program linked into every executable, so its
functions must be written in AIL.

## Decision

Add these float methods (`type.num.16`): `sin`, `cos`, `tan`, `atan`,
`atan2(y)`, `exp`, `ln`, `log10`, `pow(y)`, `hypot(y)` and `cbrt`. The
interpreter calls the platform's libm (Rust's `f64` methods). The native
backend has its own versions in `compiler/rt/native/math.ail`, and they agree
with the platform libm to within one unit in the last place. `log10` is the
one exception the spec allows to differ by two.

The native algorithms follow Sun's fdlibm (permissive licence, credited in
[NOTICE](../../NOTICE)), rewritten in the native subset:

- `exp` and `ln`: fdlibm's reductions and polynomials.
- `sin`, `cos`, `tan`: fdlibm's kernels. Arguments up to 2^20·π/2 are reduced
  with three-part π/2; larger ones use Payne–Hanek reduction with 2/π to 1200
  bits, generated with mpmath and stored as 40 limbs of 30 bits.
- `atan`, `atan2`: fdlibm's `s_atan` and `e_atan2`.
- `pow`: `exp(y·ln x)`, with `ln x` in double-double (about 2^-60) so that the
  product keeps its accuracy. The special cases follow C99 Annex F.
- `hypot`: an exact sum of squares (Dekker products) and a corrected square root.
- `cbrt`: `exp(ln(x)/3)` and two Newton steps.
- `log10`: the double-double `ln` divided by ln 10, split in two parts.

Measured on 30,000 random arguments with magnitudes from about 10^-8 to 10^7
(so trigonometry also runs through the Payne–Hanek range), and on special and
edge values, every function is within 1 ulp of the interpreter's libm. On macOS the platform libm's `tanh` also differs from the fdlibm one in the last digit, so `tanh` is within 1 ulp of the platform libm there, not bit-identical (the native test `tensor_tanh` is skipped on macOS). The one measured `log10`
exception is 2 ulp: there glibc's result is 1.9e-16 from the true value, and
ours is 6e-17.

## Options considered

- **Option a — `exp` and `tanh` only.** Enough for native tensor activations,
  but no program can then compute an angle or a logarithm. Rejected by the
  designer's choice of option c.
- **Option b — call libm from the native backend.** Rejected: the native
  runtime has no C library, and linking one breaks the standalone executables
  (ADR 0387).
- **Option c — AIL versions of the whole set, within one ulp (chosen).** Each
  function has a measured bound, and the interpreter and native agree on the
  special values exactly.
- **Option d — bit-identical results on every platform.** Rejected: it isn't
  achievable, because the platform libms themselves disagree in the last bit.

## Consequences

- `type.num.16` fixes the special values and the bound. `native.subset.1` now
  lists the functions.
- The interpreter stays on the platform libm, so its last bits can differ
  between operating systems, as `std.tensor.13` already says.
- Not added: `float32` versions, `asin`, `acos`, `sinh`, `log2` and `exp2`. Each
  is a later decision that needs the same measurement.
- Tests: `bootstrap/tests/native/run/math_float.ail` (values to nine decimals,
  built and compared with `ail run`), `math_special.ail` (special values, exact),
  and `bootstrap/tests/native/math.mir` (the lowering).
- The native runtime's file list in `compiler/main.ail` includes `math.ail`.
