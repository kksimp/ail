# 0357. `ail pkg update [NAME...]` re-resolves git dependencies

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 13 — Package management; 27 — Build system
- **Spec rules:** `mod.pkg.9`

## Context

With a lock (ADR 0356) a moved tag changes nothing, so a command must
refresh entries on purpose, one dependency at a time when needed.

## Options considered

### Option A — `ail pkg update [NAME...]` in the package root (chosen)
- **Pros:** names the action; per-dependency; leaves room for other `pkg`
  subcommands (a registry's, OQ-049).
- **Cons:** works on the current directory only.

### Option B — delete `ail.lock` (or its line) and run again
- **Pros:** no command. **Cons:** refreshes every entry; an agent must
  edit a generated file.

## Decision

`ail pkg update` refreshes every git dependency, `ail pkg update geo json`
only those; the others keep their locked commits. It fetches, rewrites the
lock and prints `locked NAME COMMIT` per git dependency. A name that isn't a
git dependency of the program is a `mod.pkg.9` error.

## Rationale

ADR 0076: AI understanding decided: one command with a stable output an
agent can check, and `mod.pkg.8`'s help names it exactly.

## Consequences

- `ail pkg` is a command group; `ail fetch` from ADR 0233 isn't needed,
  since any `run` fetches.

## Rejected alternatives

- **B:** coarse, and editing a generated file invites mistakes.
- **`ail update`:** too general a name next to `fmt`, `fix`, `test`.
