# 1057. `ail help`: the command list, one command's usage, and JSON for agents

- **Status:** Accepted
- **Date:** 2026-11-11
- **Decided by:** AI (delegated, ADR 0076). Reasoning: AI understanding (an agent's first call must answer "what can I run"), then human readability. Slice G1 of `lanes/final-gaps-2026-11-10.md`.
- **Topic:** 22 — Tools
- **Spec rules:** [tool.help.1, tool.help.2, tool.help.3](../spec/22-tools.md)

## Context

`./ail --help` said "has no native form" and a bare `./ail` printed a usage line from the bootstrap
days (`ail run compiler (check PROGRAM.json... )`). The first thing a person or an agent types had
no answer, and the release plan's wiki check against `ail --help` could not run.

## Decision

1. `ail help`, `ail --help`, `ail -h` and `ail` with no arguments print every user command, one
   line each, in five groups, and exit 0 (tool.help.1).
2. `ail help CMD` and `ail CMD --help` print the command's usage line and the rule to give
   `ail explain` (tool.help.2). `--help` is help only as the one argument after the command, so
   `ail run FILE --help` still passes `--help` to the program.
3. `ail help --json` prints the table as one JSON array; `ail help --json CMD` one object
   (tool.help.3).
4. `ail help --keywords [--json]` prints the reserved words, so editors (ail-edit) read them from the
   toolchain instead of keeping a copy (tool.help.3).
5. The table is one file, `compiler/help/help.ail`. Internal commands are not listed. The `cli`
   gate suite fails when a listed command has no route in `ail` or in `ail.ps1`.
6. An unknown command prints "unknown command X; run `ail help`" and exits 2.

## Rejected alternatives

- **Help text in the launchers.** Two copies (`ail`, `ail.ps1`) that drift: RC1 to RC3 went red on
  routes one launcher lacked.
- **Generating the text from the spec.** The spec describes rules, not a command summary; the
  generated list would need a second table for grouping and one-line summaries anyway.
- **`ail --help` printing the whole quick reference.** About 1,500 tokens to answer a question the
  list answers in about 600.

## Consequences

`ail help` needs the native CLI, so the first call on a fresh checkout builds it, like every command.
A command added to a launcher must be added to the table, or the `cli` suite is red.
