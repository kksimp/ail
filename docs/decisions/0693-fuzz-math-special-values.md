# 0693. A fuzzer for `std.math` special values, and the fuzz budget

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 16 — Testing
- **Spec rules:** type.num.16, native.build.1, fuzz.mod.1-7
- **Builds on:** [ADR 0461](0461-module-fuzzers.md), [ADR 0592](0592-native-math.md)

## Decision

1. `math_special.ail` picks `x` and `y` from 21 special floats (nan, both infinities, both zeros, 1e308, 5e-324, 709.78, 710, -745.2, pi, 1e22, ...) and applies `sin cos tan atan exp ln log10 cbrt atan2 pow hypot`. It prints the class of each result (nan, +inf, -inf, +0, -0, neg, pos), never its digits, because libm results differ in the last bit across platforms (type.num.16). `fuzz-native: yes` makes the native build agree on every class. Oracles: `pow(x, 0)` and `pow(1, y)` are 1, `hypot` is not negative, and `hypot` with an infinity is infinite.
2. No change to `tools/fuzz/run.sh`: it already runs `fuzz_modules`, which picks up every template in `bootstrap/tests/fuzz/modules/`. The default budget stays the harness's 40 inputs per template for `./ail selftest`; `tools/fuzz/run.sh N` raises it.

## Results

1,200 inputs: 0 failures; 1,200 native programs built and agreed with `ail run`.

## Rejected

- Printing the digits of results: a one-unit difference between libms would fail the native comparison for no bug.
