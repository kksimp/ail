# 0924. Ship the IANA tz database with AIL

- **Status:** Accepted (amends [ADR 0448](0448-time-zones-from-the-platform-tz-database.md) and [ADR 0840](0840-native-std-tz-in-the-runtime.md))
- **Date:** 2026-10-27
- **Decided by:** Designer (2026-10-27, "1A")
- **Topic:** 17 — Standard library
- **Spec rules:** [std.tz.1–2](../spec/17-std.md#zones-stdtz) (to be amended with the implementation)

## Context

ADR 0448 reads TZif files from `/usr/share/zoneinfo`. Windows has no such
directory, so on Windows every named zone is unknown: `std_tz`, the
`examples/calendar.ail` run and the `17-std.md` zone examples fail on ail-win
with `unknown zone: America/New_York`. Linux and macOS can also differ, because
each system installs its own release of the database.

## Options considered

### Option A — ship the public-domain IANA tz database with AIL (chosen)
- **Pros:** the same answer on every OS and every machine; Windows works; the data is public domain, so it fits the licence (ADR 0510).
- **Cons:** AIL must re-release when the rules change, a few times a year; the toolchain grows by the compiled data, a few hundred KB.

### Option B — use each platform's own data (Windows registry or ICU, `/usr/share/zoneinfo` elsewhere)
- **Pros:** the data stays as current as the OS.
- **Cons:** a second reader for the Windows registry format; answers differ between machines and OS versions; Windows zone names are not IANA names and need a mapping.

## Decision

AIL ships a copy of the IANA tz database. `std.tz` reads that copy on every OS,
so a zone name gives the same offsets everywhere. The system zone (`std.tz.2`)
is still found from the platform, and on Windows it is mapped from the Windows
zone ID to its IANA name.

## Rationale

Same results on every OS is worth more than the freshest rules: it makes tests
and recordings portable, and it removes a Windows-only code path. The data is
public domain, so it adds no licence condition.

## Consequences

- Follow-up: a tool that regenerates the bundled data from an IANA release, the
  runtime change to read it, the Windows zone-ID mapping, and the `std.tz.1`
  wording. These change the spec when they land.
- A new IANA release means a new AIL release; the data version is recorded next
  to the data.
- `std_tz`, `examples/calendar.ail` and the `17-std.md` zone examples can then
  pass on Windows.

## Rejected alternatives

- Platform data (option B): answers differ between machines, and Windows needs a second reader and a name mapping.
