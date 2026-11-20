# 1171. The language version becomes 0.2

- **Status:** Accepted
- **Date:** 2026-11-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked for the 0.2 release on 2026-11-19
- **Topic:** 13 — Package management; 4 — Versioning
- **Spec rules:** `mod.pkg.10`, `mod.pkg.11`, `mod.pkg.12`
- **Amends:** [ADR 1084](1084-language-version-line-in-ail-pkg.md)

## Context

0.2 adds forms that a 0.1 compiler rejects: functions that never return, one-line `elif`, `x = e else` blocks,
`T ! str` and `Fail`. The compiler still reported language version 0.1 and `ail new` wrote `ail 0.1`, so an
agent could not tell a tree that uses 0.2 forms from a 0.1 tree, and `ail fix --migrate` had no step.

## Options considered

### Option A — bump to 0.2; trees that say `ail 0.1` stay accepted (chosen)
- **Pros:** a manifest says which forms its sources may use; a 0.1 compiler refuses a 0.2 tree with `mod.pkg.11`
  instead of failing on the first new form; no 0.1 tree breaks, because 0.2 only adds forms.
- **Cons:** the migrate step is only a version-line rewrite.

### Option B — keep 0.1 because the changes are additive
- **Pros:** no code change. **Cons:** an old toolchain gives parse errors on 0.2 trees, not the version message.

### Option C — refuse `ail 0.1` trees
- **Pros:** one version in the field. **Cons:** breaks every 0.1 tree for no semantic reason.

## Decision

Option A. `LANG_MINOR` is 2, so `lang_version()` is `0.2`. `ail new` writes `ail 0.2`. A tree that says `ail 0.1` is
accepted and prints nothing. `ail fix --migrate [DIR]` rewrites a 0.1 tree's `ail` line to `ail 0.2` and prints
`migrated 0.1 to 0.2`; at 0.2 it prints `nothing to migrate from 0.2`. The test packages `version_ok` (now `ail 0.2`)
and `version_newer` (now `ail 0.3`) follow.

## Consequences

- Spec `mod.pkg.10-12`, `docs/packages.md`, the language card and the `ail new` text agree on 0.2.
- Generated files (the pkg card, `llms-full.txt`) change; the integrator regenerates them.
- The test package `version_older` (`ail 0.1`) is accepted; its api and fmt golden rows come with the batch regeneration.
