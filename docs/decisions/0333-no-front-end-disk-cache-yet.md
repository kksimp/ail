# 0333. Don't cache the parsed prelude or check results on disk yet

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Tooling speed (Stage 2)
- **Spec rules:** none

## Context

Lane 81's brief proposed caching the parsed prelude and the standard modules
on disk, keyed by the binary's hash, under `$XDG_CACHE_HOME/ail` or
`~/.ail-cache`, with `--no-cache` and `--deterministic` turning it off. The
profile ([interpreter-performance.md](../design/interpreter-performance.md#front-end-lane-81))
measured what that would save first.

## Options considered

### Option A — no disk cache now; make the passes cheaper (chosen)
- **Pros:** no stale-cache failure mode, no new flags, no file system writes
  from a read-only command; the measured wins (ADR 0332, ADR 0334) apply to
  every run, edited or not.
- **Cons:** an unchanged program is checked again from scratch.

### Option B — cache the parsed prelude and std modules
- **Pros:** what the brief proposed.
- **Cons:** lexing and parsing the prelude, `std.types` and `std.ui.widgets`
  costs about 0.3 ms of a 1.7 ms `ail check hello` (process start is 1.3 ms),
  and under 7% of the editor check; reading and decoding a serialized syntax
  tree costs a similar amount, and the AST has no serializer.

### Option C — cache whole `ail check` results by the contents of every input
- **Pros:** about 1.5 ms for a repeated check of an unchanged program.
- **Cons:** the inputs include every imported module, each module directory's
  file list and every `ail.pkg` manifest; missing one gives a stale answer,
  the worst failure for an agent's main feedback loop. Agents mostly check
  after an edit, when the cache misses.

## Decision

No on-disk cache for now. Revisit Option C when `ail check` has a daemon or
watch mode that already tracks its inputs.

## Rationale

Criterion 2 (AI understanding: a check result must never be stale) outweighs
criterion 4 when Option B's measured saving is under 0.5 ms.

## Consequences

- No `--no-cache` flag and no cache directory.
- The editor and agents get the speed-up on every check, edited or not.

## Rejected alternatives

- **Prelude cache:** saves under 0.5 ms; costs a serializer and invalidation.
- **Result cache:** a stale answer is worse than a slower one; most checks
  follow an edit.
