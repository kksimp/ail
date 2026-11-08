# 0924. Ship the IANA tz database with AIL

- **Status:** Accepted (amends [ADR 0448](0448-time-zones-from-the-platform-tz-database.md) and [ADR 0840](0840-native-std-tz-in-the-runtime.md))
- **Date:** 2026-10-27
- **Decided by:** Designer (2026-10-27, "1A")
- **Topic:** 17 — Standard library
- **Spec rules:** [std.tz.1–3](../spec/17-std.md#zones-stdtz)

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
is still found from the platform. On Windows it is `TZ` when that names a zone,
else `UTC`; mapping the Windows zone ID to its IANA name is deferred (see
Implementation).

## Implementation

- **Where:** `tools/tzdata/zones/` holds one slim TZif file per name (598
  files, 342 KB; links are copies, and slim files rely on the POSIX footer
  rule for later years). `rt.tz_offset` (`compiler/rt/native/tz.ail`) looks in
  `AIL_ROOT/tools/tzdata/zones` first, then the system directories, which
  Windows lacks. `ail` and `ail.ps1` set `AIL_ROOT`; a program run without it
  falls back to the system copy.
- **Regenerate:** `tools/tzdata/build.sh` runs `zic -b slim` over
  `tools/tzdata/tzdata.zi`. To update, replace `tzdata.zi` with the new
  release's file and run it. The release is recorded on the first line of
  `tzdata.zi` (`# version 2026e`) and nowhere else.
- **Spec:** `std.tz.1` and `std.tz.3` now say the bundled copy is read first.
- **Windows `tz.system()`:** the Windows-to-IANA mapping needs the CLDR
  `windowsZones` table (about 140 rows, a few times a year) and a reader for
  the active zone ID (`tzutil /g` or the registry). Neither is in the tree, and
  no test needs it, so it stays `TZ` else `UTC`. Open as a follow-up if a
  program needs the local Windows zone.

## Rationale

Same results on every OS is worth more than the freshest rules: it makes tests
and recordings portable, and it removes a Windows-only code path. The data is
public domain, so it adds no licence condition.

## Consequences

- Follow-up, done in [ADR 0972](0972-windows-system-zone.md): the Windows zone-ID mapping for `tz.system()`; the CLDR table it needs is under the Unicode License v3, so the change adds a `NOTICE` entry.
- A new IANA release means a new AIL release; the data version is recorded next
  to the data.
- `std_tz`, `examples/calendar.ail` and the `17-std.md` zone examples can then
  pass on Windows.

## Rejected alternatives

- Platform data (option B): answers differ between machines, and Windows needs a second reader and a name mapping.
- Windows icu.dll (`ucal` functions): needs a five-argument native call the
  runtime's `import_call` does not have, and its data follows the OS release.
- The Windows registry as the data source: a second reader and a name mapping,
  with answers that differ between machines.
