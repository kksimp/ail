# 0585. The self-compile survey is a script that lowers each module alone

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; lane 181 of the native self-compile push)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.subset.1](../spec/23-native.md)
- **Builds on:** [ADR 0384](0384-mir-shape-and-native-subset.md) (MIR and the native subset), [ADR 0022](0022-staged-self-hosting.md) (self-hosting)

## Context

`ail build compiler/main.ail` reports only the five native-subset problems
in `main.ail` itself. The entry point's `use a64`, `use mir` and similar
lines are not loaded yet, so the other 29 modules are never lowered. The
self-compile needs one number per construct, and that number has to come
from the whole compiler, not from its entry point.

## Decision

`tools/self-compile/run.sh` runs `ail run compiler mir FILE compiler/rt/native`
on every module under `compiler/` (except `main.ail` and the runtime), and
writes `problems.tsv` and `summary.md` to a directory of the caller's choice.
Each file is lowered alone, with the native runtime as its runtime. Problems
are grouped by file and by construct, with names in back-quotes normalised.

## Rejected alternatives


- **Wait for module loading and use `ail build compiler/main.ail`.** Accurate
  once it lands, but it gives no numbers now and hides which files block.
- **Concatenate all modules into one program.** Duplicate names across
  modules (for example `put` in `obj/link.ail` and `obj/macho.ail`) would
  collide, and the result would not match any real build.
- **Count problems from the source by grep.** Misses the lowering's real
  rules, which change with every lane.

## Known error of the method

A call of a function defined in another module is
reported as `calling X`, and a qualified constant (`codegen.SLOT_BASE`) is
reported as field access. The script can't tell these from real gaps, so
the status document marks them as cross-module. The script's output is
re-checked after module loading lands, and this ADR is superseded then.

## Consequences

- Anyone can rerun the survey in about 25 seconds on a Linux x86-64 cloud
  container, and its output is in `docs/design/self-compile-status.md`.
- The script writes only to the directory it is given. It needs no network
  and no hosted service.
