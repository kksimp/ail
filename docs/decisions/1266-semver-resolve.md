# 1266. `semver.resolve`: a dependency resolver in `std.semver`

- **Status:** Accepted (number pending; the integrator allocates it)
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 17 — Standard library; AI understanding
- **Spec rules:** std.semver.13 and std.semver.14 in [std-semver.md](../spec/std-semver.md)
- **Extends:** [ADR 1219](1219-std-semver.md)

## Context

ADR 1219 gave versions and ranges. Programs that handle packages (a build tool, a plugin loader, `compiler/pkg/`) also
need to pick one version of each package from many candidates, where each version has requirements of its own. Agents
write this search badly: they take the highest version of each requirement in turn and never undo a choice.

## Decision

`semver.resolve(roots, packages) List[Pin] ! SemverError`, pure AIL in `std/semver/semver.ail`. Inputs are data, with
no registry or file access: `Dep` (name, range text), `Package` (name, version, deps) and the result `Pin` (name,
version).

- One version per package (the Cargo and npm-flat model), so a conflict is a failure, not two copies.
- Depth-first search, highest fitting version first, packages in the order first required, undoing a choice by trying
  the next lower version. The answer is deterministic; the pins are in name order.
- Ranges are parsed once, up front; a bad range fails with `parse_range`'s own error.
- Failures reuse `SemverError`, with two new kinds, `NoPackage` and `NoSolution`; the text names the package and the
  ranges (or the two requirements) of the last failed attempt.
- Prerelease versions follow std.semver.10, so `*` and `^1.2.3` never pick one.

## Rejected alternatives

- **A SAT or PubGrub solver.** Better conflict messages and no exponential worst case, but far more code for the
  sizes programs resolve; the search can be replaced behind the same call.
- **Several versions of one package (npm nesting).** Hides conflicts and needs a tree result type.
- **Reading a registry or files inside `resolve`.** Would need capabilities; the caller supplies the candidates.
- **Preferring the lowest version (Go's minimal version selection).** Predictable, but surprises a reader who expects
  `^1.2` to give the newest 1.x.
- **A separate `SolveError` type.** A second error type for one call; the kinds say what failed.
