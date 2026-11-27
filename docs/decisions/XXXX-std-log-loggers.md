# TBD. `std.log` loggers: fields, JSON lines, rotating file target

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 17 — Standard library
- **Spec rules:** std.log.3-14 ([std-log](../spec/std-log.md)); extends [0628](0628-std-log.md)

## Context

ADR 0628 left structured fields out until the corpus showed the need. Services
and long-running tools need fields per request, machine-readable lines, a file
target and bounded file size (std gap plan, lane 19).

## Decision

Add a `Logger` value (`log.named`, `with`, `child`, `with_level`, `json`,
`text`, `with_time`, `to_file`, `to_stderr`, `*_kv` methods). Text lines are
`[ts ]LEVEL [name: ]message key=value`; JSON lines hold `ts`, `level`,
`logger`, `msg` and the fields. Rotation is by size (`path.1` ... `path.keep`).
`LOG_FORMAT=json` joins `LOG_LEVEL`. The plain functions keep their exact output.
Write failures are dropped, so logging never traps.

## Rejected alternatives

- **Global `set_level` or `set_output`:** AIL has no mutable globals.
- **Timestamps on the plain functions:** it would change output scripts match.
- **Rotation by age, compression:** size covers the common case.
- **Typed field values:** a string map is what call sites have.
