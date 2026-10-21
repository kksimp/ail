# 0626. `std.csv`: CSV as rows and records

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 17 — Standard library
- **Spec rules:** std.csv.1-5 ([17-std](../spec/17-std.md))

## Context

Lane 192 compared 25 everyday agent tasks in Python and AIL. Reading CSV is
the most common: Python has `csv.DictReader`; AIL had only `line.split(",")`,
which breaks on quoted fields, so an agent either gets quoting wrong or
writes a 25-line state machine.

## Decision

`std.csv`, written in AIL (`bootstrap/src/std/csv.ail`), imported with
`use std.csv`: `parse`, `parse_sep`, `records`, `quote`, `write`,
`write_records`, and `CsvError` (`why`, `row`). RFC 4180 quoting; `\n` and
`\r\n` rows; blank lines add no row (std.csv.1-5). Standard modules written
in AIL may now have two-part paths (`std.csv`), as well as three-part ones
(`check/stdlib.rs`, `compiler/check`).

## Token measurement

`tools/tokbench/corpus/agent-r4/csv_manual.ail` (a hand-written quoted-field
splitter and a sum of one column) against `csv_std.ail`, tokbench mean over
the five tokenizers: **236.4 → 69.6 tokens (-71%)**.

## Rejected alternatives

- **Native (Rust) functions.** Cost: a runtime and a native-backend arm for
  each. A module in AIL runs on every tier and builds natively unchanged.
- **`parse` without errors.** An unterminated quote would silently swallow
  the rest of the file; a result costs one `?`.
- **Typed rows by derive (`FromCsv`).** Wanted eventually; `records` plus
  `to_int` covers the common case now (open for a later lane).
- **Blank lines as `[""]` rows.** Python's reader gives `[]`; agents filter
  them. Skipping them is the shorter program.
