# 0901. `std.toml`: TOML read into the `Json` value

- **Status:** Accepted
- **Date:** 2026-10-25
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 17 — Standard library
- **Spec rules:** std.toml.1-8 ([17-std](../spec/17-std.md))

## Context

The adoption-gaps review (2026-10-24) listed TOML, YAML and XML as missing
std formats. TOML is the config format of Python (`tomllib`), Rust and most
new tools, and AIL's own `ail.pkg` manifest is TOML-shaped. A program that
needs a config file today hand-writes a parser.

## Decision

`std.toml`, written in AIL (`std/toml/toml.ail`), imported with `use std.toml`:
`parse(text) Json ! TomlError`. The result is the existing `Json` value, so the
accessors agents already know (`get`, `at`, `str`, `num`, `as_arr`) read it
and `to_text` prints it as JSON. Read only, like `tomllib`. Dates and times stay
`Str`; integers are `Num` (std.toml.1-8). The module is pure AIL: no lowering,
runtime or Rust change, so it builds natively and on every target unchanged.

## Rejected alternatives

- **A separate `Toml` value enum.** Cost: a second set of accessors and a
  converter to `Json` for every program that feeds config to JSON code. Gain:
  an `Int` variant and date variants. Add them if integer precision matters.
- **A writer (`toml.write`).** Rarely needed by programs; deferred.
- **YAML first.** Its grammar is far larger and has no single spec to cite.
- **A Rust implementation.** Against ADR 0873 (native first, no Rust-only features).

## Measurements

None yet: tokbench needs a hand-written parser twin; open for a later change.
