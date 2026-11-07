# 0972. Windows system zone: map the Windows zone ID to its IANA name

- **Status:** Accepted (completes the follow-up in [ADR 0924](0924-ship-the-tz-database.md))
- **Date:** 2026-11-07
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 17 — Standard library
- **Spec rules:** [std.tz.2, std.tz.3](../spec/17-std.md#zones-stdtz)

## Context

ADR 0924 ships the IANA database, so named zones work on Windows. `tz.system()`
(and so `date.now_local()`) still answered `TZ` else `UTC` there, because
Windows names its zones by its own IDs (`Eastern Standard Time`), not IANA names.

## Decision

On Windows, when `TZ` doesn't name a safe zone, `tz.system()` reads the active
zone ID with kernel32 `GetDynamicTimeZoneInformation` (one pointer argument;
`TimeZoneKeyName` is the UTF-16 string at byte 172 of the 432-byte struct) and
looks it up in `tools/tzdata/windows-zones.txt`. That file has one
`Windows ID<TAB>IANA name` line per ID (142 lines), the territory-001 rows of
CLDR's `windowsZones.xml`; `tools/tzdata/windows-zones.sh` regenerates it. Any
failure (no `AIL_ROOT`, unknown ID, unsafe name) gives `UTC`, as before. The
table is read at call time from `AIL_ROOT`, like the zone files, so no runtime
global slot or hot file changes. CLDR is under the Unicode License v3, which
`NOTICE` records.

## Rationale

`GetDynamicTimeZoneInformation` fits the runtime's four-argument `import_call`
and needs no registry reader. The territory-001 row is CLDR's default zone for
an ID, which matches what `tzutil /g` reports.

## Rejected alternatives

- Registry read (`RegGetValueW`): six arguments, beyond `import_call`.
- Spawning `tzutil /g`: needs a process spawn in the runtime, slow, and depends on PATH.
- Embedding the table in the runtime image: grows every program, for a rarely used lookup.
