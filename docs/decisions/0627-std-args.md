# 0627. `std.args`: flags, options and positionals with help

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 17 — Standard library
- **Spec rules:** std.args.1-6 ([17-std](../spec/17-std.md))

## Context

`argparse` is the first library an agent reaches for in a command-line tool.
AIL had `env.args()` only, so each tool hand-rolled `--out V`, `--out=V`,
`-v`, `--` and its usage text.

## Decision

`std.args`, in AIL: declare specs with `flag`, `opt`, `pos` and `rest`, then
`args.parse(name, about, specs, env.args())?` gives an `Args` with `flag(n)`,
`get(n)` and `rest`. `--help` and `-h` print generated usage and exit 0;
failures are an `ArgsError` carrying the reason and the usage text
(std.args.1-6).

## Token measurement

`agent-r4/args_manual.ail` against `args_std.ail` (`[-v] [--out FILE] INPUT`,
with errors): **240.0 → 113.0 tokens (-53%)**, and the std version also
answers `--help`, `--out=F` and `--`.

## Rejected alternatives

- **A builder (`p := args.parser(...)`, `mut p`).** More tokens (a variable
  and a call per argument) and needs `mut`; one list of specs reads as data.
- **Derive from a struct.** Attractive, but needs attribute design; revisit
  when derive generators cover it (ADR 0269 area).
- **`-abc` bundles and `-ofile`.** Rare in agent tools; a short flag is one
  letter and its value is the next word.
- **Exit inside `parse` for errors too.** A returned error lets the program
  choose the status and the stream.
