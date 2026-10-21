# 0783. Std error families: a closed kind list per module, as spec rules

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076), on the coordinator's direction
- **Topic:** 17 — Standard library (`std` errors, `err.type.3`)
- **Spec rules:** `std.fs.20`–`22` (this change); the family rule for each module follows in later batches

## Context

`err.type.3` requires a public function to name its error type. Every fallible
`std` function already does (`fs.IoError`, `proc.ProcError`, `net.NetError`,
`json.JsonError`, `csv.CsvError`, and the rest). What each module still lacked is
a closed list of the causes its `kind` can take, with a rule and an `ail explain`
text per member. `net` and TLS have that shape now; the other modules describe
their kinds in prose.

## Decision

Each `std` error family keeps its `kind` as a `str`. The closed list of `kind`
values is written as spec rules, one per member, in the module's own chapter,
and every member has an `ail explain` text and a checked or `norun` fence.

## Rejected alternatives

- **A real enum for `kind`.** It would be the cleaner type, but every program that
  compares a kind string would break, and it changes the language's error model.
  That is a language-level decision, so it needs the designer's approval first.
- **One rule for the whole list.** Each member then has no text to explain, and a
  diagnostic citing a single member could not point at its cause.
- **Leaving the kinds in prose.** The rules-audit cannot check prose, so a kind with
  no rule would never be noticed when it is added or removed.

## Consequences

- `std.fs` gains `std.fs.20`–`22` for `exists`, `permission` and `other`.
- `std.time/date`, `std.json`, `std.regex`, `std.http`, `std.zip/gz` and `std.csv`
  follow the same shape in later PRs.
