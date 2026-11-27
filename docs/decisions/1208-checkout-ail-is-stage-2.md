# 1208. In a checkout, `./ail` runs a stage-2 compiler built from the working tree

- **Status:** Accepted
- **Date:** 2026-11-26
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); requested by the designer's coordinator as developer friction
- **Topic:** 23 — Compiler architecture
- **Spec rules:** native.build.3 (no behaviour change for programs)
- **Amends:** [ADR 0855](0855-retire-the-rust-bootstrap.md) (the published seed builds the native CLI)

## Context

`./ail` built `.native/ail` once, with the published seed (stage 0), and rebuilt it when `compiler/` or `std/` had a newer
file. So `ail build`, `check` and `run` in a checkout ran stage 1: the tree's source, compiled by the seed's code
generator and runtime. When the tree used something the seed didn't know (the first case: `TaskGroup[int]()`, ADR 1204),
stage 1 differed from what the tree would build, and lanes had to build stage 2 by hand to test their work. Several gate
runs were lost this way.

## Decision

- In a checkout, `./ail` makes `.native/ail` in two steps when it is stale: the seed builds stage 1 (a temporary file),
  stage 1 builds `compiler/main.ail` again, and that stage 2 becomes `.native/ail`. Stage 1 and stage 2 build the same
  executable (the fixed point, ADR 0610), so what changes is only which compiler built the CLI.
- If the seed can't build the tree but a stage 2 is already cached, that stage 2 plays stage 1's part, so a tree ahead of
  the seed keeps building. With no cached stage 2 it stays the existing error.
- `AIL_USE_SEED=1` keeps stage 1 as before. `.native/ail.stage` records which stage is cached, so switching rebuilds. The
  name is not `AIL_SEED`: that is the random seed of `--seed` (std.random.3).
- Staleness is unchanged: any `.ail` under `compiler/` or `std/` newer than `.native/ail`, skipped under
  `AIL_NATIVE_FRESH=1`. A gate's first build brings stage 2 up to date; the suites then reuse it, so every suite runs the
  same compiler (deterministic).
- Releases and installed `ail` are unchanged: this is the launcher script of a checkout only. `ail.ps1` (Windows)
  stays as it is until parity resumes.
- Cost: a rebuild after a `compiler/` change takes about twice as long (two builds).

## Rejected alternatives

- **Rebuild from the previous stage 2 only, no seed step.** Faster, but a build then depends on the cache's history, so two
  checkouts of one commit could differ. The seed chain is the same everywhere.
- **`AIL_SEED=1` as the switch.** Clashes with the `--seed` variable.
- **Keep stage 1 and add a `./ail stage2` command.** Leaves the trap in place for every lane that doesn't know about it.
