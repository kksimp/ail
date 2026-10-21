# 0772. Standard modules written in AIL compile natively as program modules

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — native backend
- **Spec rules:** std.ui.19, mod.use.1, native.subset.1
- **Builds on:** [ADR 0393](0393-native-modules.md)

## Context

Six standard modules are plain AIL source (`bootstrap/src/std`: `args`,
`csv`, `log`, `net.http`, `net.jar`, `time.date`; ADR 0626 and others). The
interpreter reads them from the bootstrap binary. `ail build` stopped at
`use std.log` with "`log` isn't in the native subset yet", though the
module is 35 lines of the subset. Writing each by hand as runtime code
(as ADR 0770 did for `proc`, which needs system calls) would duplicate the
source and let the two drift.

## Decision

- `mir.aliases_of` treats `use std.NAME` for the six names (`STD_AIL` in
  `lower_mod.ail`) as an import of module `std.NAME`, like a user's `use a.b`;
  the other standard modules stay runtime functions.
- `ail run compiler build` takes the standard-library directory as an
  optional seventh argument, and `ail build` passes
  `<compiler>/../bootstrap/src/std`. `lower_file` loads the module's one file
  (`std.csv` is `csv/csv.ail`, `std.net.http` is `net/http.ail`) like any
  imported module, so its functions are `std.log.info` and so on and the
  subset rules (and the `native.subset.1` reports, with that file's path)
  apply to it unchanged.
- The build cache key already covers the bootstrap binary, which embeds the
  same sources, so a change to a standard module changes the key.

## Result

`std.log` builds and matches `ail run` (stdout and stderr, with and without
`LOG_LEVEL`). The others report their own blockers in their own files:
`std.args`, `std.csv` and `std.time.date` need error structs with fields and
`?` on user error types (the optionals and lane 179 threads); `std.net.jar`
needs `.last()`, `.take()`, `.index_of()` on lists and `.to_int()` on `str`.
Each starts building when those land, with no further work here.

## Why not

- **Runtime code for each module:** duplicates AIL source, and the AIL
  version is the specification's reference behaviour.
- **Embedding the sources in the compiler:** the compiler would carry a
  second copy; the repository layout already guarantees the directory next to
  `compiler/`.
- **Putting the modules in the runtime's file list:** that list is
  runtime-only (`rt.` names, no module scope).
