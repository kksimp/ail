# XXXX. `std.log`: fields, JSON lines and a file target

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 17 — Standard library
- **Spec rules:** std.log.3-8 ([17-std](../spec/17-std.md))
- **Amends:** [0628](0628-std-log.md) (its rejected "structured fields" and "no timestamps" now have an opt-in form)

## Context

ADR 0628 deferred structured fields until the corpus showed the need. Services
and tools that ship logs to a collector need `key=value` fields, one JSON
object per line, and a file. The std gap plan (lane 19) lists all three.

## Decision

- `debug_kv`, `info_kv`, `warn_kv`, `err_kv` take a `Map[str, str]` (insertion order).
- `log.fields(map)` and `log.to_file(path)?` give a `Logger` with the same methods;
  `l.fields(more)` adds fields. `with` is a keyword, hence `fields`.
- Environment variables, like `LOG_LEVEL`: `LOG_FORMAT=json`, `LOG_FILE=path`,
  `LOG_TIME=1` (UTC timestamp). With none set, output is unchanged and deterministic.
- A failed file write falls back to standard error rather than trapping: a logger must not crash the program.

## Rejected alternatives

- **Global `log.set_file` / `set_format`.** No mutable globals (as in 0628); a `Logger` value and the environment cover it.
- **Typed field values (`Map[str, Json]`).** More to write at each call; strings suffice for log lines. Add if needed.
- **Timestamps on by default.** Breaks the deterministic golden output programs and tests rely on.
- **A variadic `log.info(msg, k, v, ...)`.** AIL has no variadics.
