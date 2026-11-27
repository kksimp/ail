# 1231. `std.args`: subcommands, environment fallbacks and `--version`

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 17 Standard library
- **Spec rules:** [std.args.9-11](../spec/17-std.md)
- **Amends:** [ADR 0627](0627-std-args.md) (adds to it; nothing in it changes)

## Decision

- `command(name, help, specs)` and `parse_commands(name, version, about, cmds, words)` give `git`-style subcommands with
  `Parsed.command` and `Parsed.opts`; `help`, `help CMD` and `CMD --help` print usage.
- `Spec.from_env(key)` makes an environment variable the fallback of an option or switch: command line, then
  variable, then default. `parse_in` and `parse_commands_in` take the environment as a map, so tests need no real one.
- A non-empty `version` adds `--version` and `-V`.

## Rejected alternatives

- Nested subcommands: one level covers the tools in the corpus, and a command's handler can parse again.
- Reading the environment always: `_in` forms keep tests deterministic.
- `-v` for version: it is commonly verbose.
