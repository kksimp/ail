# 0448. Time zones: UTC, fixed offsets and the platform tz database through `std.tz`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 17 — Standard library
- **Spec rules:** [std.tz.1–3](../spec/17-std.md#zones-stdtz)

## Context

A calendar needs zones and daylight saving. The IANA database is the
shared source: Linux and macOS both ship TZif files in `/usr/share/zoneinfo`.
Modern "slim" files hold transitions only until a past year and then rely on
a POSIX rule in the footer for every later year.

## Options considered

### Option A — read the platform's TZif files in the runtime (chosen)
- **Pros:** no data shipped with AIL, so zones follow the platform's updates;
  about 300 lines of Rust with no crate (ADR 0009); the file format is stable
  (RFC 9636).
- **Cons:** a machine without the database has only UTC and fixed offsets;
  Windows has no such files.

### Option B — embed the database in the toolchain
- **Pros:** the same answers everywhere.
- **Cons:** a copy that goes stale and must be released; the rules are political
  and change several times a year.

### Option C — a crate (`chrono-tz` or `tz-rs`)
- **Cons:** an external dependency for 300 lines (ADR 0009), and one more thing
  to remove before the toolchain is free of Rust.

## Decision

Option A. Two natives in `std.tz` (not in `std.time`, because their effects
are `fs` and `env`, not `clock`): `tz.offset(zone, secs) int?` and
`tz.system() str`. The reader takes the version 2+ block, the transitions, and
the footer rule (`Mm.w.d`, `Jn` and `n` days, with times) for later instants;
zone names are limited to letters, digits and `/_+-` with no `..` so a name
cannot name another file. Parsed zones are cached for the run. `std.time.date`
turns `offset` into local fields, and into instants for local times.

**Windows (ADR 0004):** no TZif files, so `tz.offset` answers `none` for named
zones and `tz.system()` is `"UTC"`; the registry reader (`TimeZoneInformation`
and the CLDR name table) is a follow-up that fits behind the same two
functions. UTC and fixed offsets work on every target.

## Consequences

- Cited stub in `std.tz.3`; Linux and macOS are tested in lockstep.
- Leap-second ("right/") tables are not read; time is POSIX seconds.
- The self-hosted checker lists `tz` among the standard modules.
