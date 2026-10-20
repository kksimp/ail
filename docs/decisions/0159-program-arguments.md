# 0159. `ail run FILE ARGS...` gives the program its arguments and its path

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 12 — Modules and programs; tooling
- **Spec rules:** [std.env.1, std.env.4](../spec/17-std.md)

## Context

`env.args()` needs a source. The `ail` command line accepted only
`run FILE`, with `--json` anywhere. A tool also often needs its own location
(to read data next to it, or itself), which `argv[0]` gives in C.

## Decision

Every word after `ail run FILE` belongs to the program, including words that
look like `ail`'s own options; `ail`'s options go before `FILE`.
`env.args()` returns those words. `env.program()` returns `FILE` as given.

## Deciding criterion

Tokens **tie** (no program text changes). **AI understanding** decides: one
rule with no exceptions (what follows the file is the program's), the same
as `python FILE ARGS` and `cargo run -- ARGS` without the `--`.
`env.program()` as a separate function keeps `args()` free of the
off-by-one that `argv[0]` causes.

## Consequences

- `ail run FILE --json` now passes `--json` to the program; write
  `ail run --json FILE` for JSON diagnostics. No test or script relied on
  the old order. `--deny LIST` (ADR 0165) also goes before FILE.

## Rejected alternatives

- **`--` before program arguments:** an extra token on every call, and
  forgetting it silently changes meaning.
- **Path as `args()[0]`:** the familiar off-by-one.
