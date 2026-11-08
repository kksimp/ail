# 0840. `std.tz` in native builds: a TZif reader in the runtime

- **Status:** Accepted; amended by [ADR 0924](0924-ship-the-tz-database.md)
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; approved by the integrator for the native self-compile lanes)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.subset.1](../spec/23-native.md), std.tz.1-3
- **Builds on:** ADR 0448 (`std.tz`), [ADR 0772](0772-native-std-modules-in-ail.md) (std modules written in AIL)

## Context

`std.time.date` is written in AIL and builds as a module of the program
(ADR 0772), but every function failed natively on one name, `tz`: its
zone functions `tz.offset(zone, secs)` and `tz.system()` were implemented
only by the interpreter (`bootstrap/src/interp/tz.rs`).

## Decision

1. **The runtime implements `tz.offset` and `tz.system`** in
   `compiler/rt/native/tz.ail`: a TZif reader (RFC 9636, version 2 or later
   block, falling back to version 1), the POSIX rule of the file's footer
   for instants after the last transition, the same four zoneinfo
   directories in the same order, the same safe-name check, and
   `/etc/localtime` read with `readlink` for the system zone. It follows
   `tz.rs` step by step; the test compares both on the same zones and
   instants.
2. **The lowering maps the two calls to `rt.tz_offset` and `rt.tz_system`**
   (`lower_os.ail`), and `tz` joins the standard modules the runtime
   implements. `tz.offset` returns an optional integer, none for an unknown
   or unsafe name.
3. **Windows** has no TZif files: `tz.offset` is none and `tz.system()` is
   `UTC` (std.tz.3), as in the interpreter.
4. **No global slots, no cache.** Each call reads the file again; the
   zone files are small and a call is dwarfed by the work of formatting a
   date.

## Rejected alternatives

- **Running the interpreter's reader through a helper process.** A native
  executable would depend on `ail` at run time.
- **Embedding a tz database in the runtime.** It would need updating with
  every tzdata release and would differ from the platform's zones, which
  `ail run` reads.
- **Writing `std.tz` in AIL as a standard module** (ADR 0772). The module
  would pull the byte-level reading into every program's lowering; the
  runtime's dead stripping drops it when unused.

## Consequences

- `tests/native/run/tz_zones.ail` and `date_native.ail` match the
  interpreter on x86-64 and aarch64.
- `std.time.date` builds natively on Linux and macOS.
- A cache is easy to add (a global slot per zone) if a measured program
  needs one.
