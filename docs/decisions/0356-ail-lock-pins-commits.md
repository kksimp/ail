# 0356. `ail.lock` pins each git dependency to a commit; `run` and `test` write it

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 13 — Package management; 27 — Build system
- **Spec rules:** `mod.pkg.8`

## Context

ADR 0233 designed `ail.lock` with a `commit` and a `sum sha256:...` per
git dependency, written by a future `ail fetch`. Implementing it needs a
format, a staleness rule and the commands that write it.

## Options considered

### Option A — commit only, written by `run`/`test` when stale (chosen)
- **Pros:** no extra command to learn; the commit hash already names the
  tree (git's hashes cover every file), and the cache is read-only.
- **Cons:** `run` writes a file next to the manifest.

### Option B — commit plus `sum` over the files (ADR 0233 as designed)
- **Pros:** detects a tampered cache. **Cons:** needs SHA-256 in the
  toolchain (no crate today) and hashes every file on every load.

### Option C — only `ail pkg update` writes the lock
- **Pros:** runs never change files. **Cons:** a first run fails until the
  agent runs another command.

## Decision

One line per git dependency of the whole graph, sorted by name, in the
manifest's word format: `dep NAME git URL rev|tag PIN commit COMMIT`, under a
generated-file comment. A matching line pins the commit. `ail run` and
`ail test` rewrite a stale lock once every fetch succeeded; `check`, `api`
and others only read it. `--deterministic` rejects a stale lock
(`mod.pkg.8`). Amends ADR 0233: `sum` is dropped.

## Rationale

ADR 0076: tokens equal (generated file, never read); AI understanding
decided A over C: the first run just works and the lock is reproducible from
then on, and the stale error names the fix. Speed decided A over B: no file
hashing on each load. The pin is in the line so editing the manifest makes
the lock stale without comparing anything else.

## Consequences

- A moved tag doesn't change a locked build; `ail pkg update` (ADR 0357)
  does.
- ADR 0233's cache keys use the commit where they said `sum`.

## Rejected alternatives

- **B:** cost on every load for a check the read-only cache mostly gives.
- **C:** an extra step on the common path.
- **TOML/JSON lock:** a second format for one file; the manifest's words parse it.
