# 0690. Fuzzers for std.csv and std.args, and the blank-row fix in `csv.write`

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 16 — Testing
- **Spec rules:** std.csv.1, std.csv.3, std.csv.5, std.args.1-6, fuzz.mod.1-7, fuzz.policy.1
- **Builds on:** [ADR 0461](0461-module-fuzzers.md), [ADR 0626](0626-std-csv.md), [ADR 0627](0627-std-args.md)

## Context

`std.csv` and `std.args` parse untrusted text. The module fuzzers (ADR 0461) take one template per module; neither had one.

## Decision

1. Two input kinds in `bootstrap/tests/fuzz/inputs.rs`. `csv` joins plain, quoted, doubled-quote, unterminated and multi-line fields with `,` and `\n`, `\r\n` or `\r`, then sometimes cuts or repeats the text. `args` makes a command line of flags, options, `--`, `-`, `=` forms and junk, one word per line.
2. Two templates. `csv.ail` checks that a failure is a `CsvError`, that `write` then `parse` returns the same rows, and that `records` agrees with `parse`. `args.ail` checks that a bad line is an `ArgsError` carrying the usage text.
3. **Finding:** `csv.write` wrote a row of one empty field as a blank line, which `parse` skips (found at seed 4 of 400). It now writes `""` for that row (std.csv.3, spec text updated). Case: `fuzz_csv_blank_row`.

Runs: 2,000 `csv` inputs and 3,000 `args` inputs, 0 failures after the fix.

## Rejected

- Having `parse` keep blank lines as `[]` rows: the spec says a blank line adds no row.
- Quoting every field in `write`: it changes every output for a single edge case.
