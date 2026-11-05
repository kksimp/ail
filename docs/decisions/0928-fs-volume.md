# 0928. `fs.volume`: total and free bytes of a volume

- **Status:** Accepted
- **Date:** 2026-11-04
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** Standard library (`std.fs`)
- **Spec rules:** std.fs.24

## Context

An analyzer must show how full the disk is. No std call reports it.

## Options considered

### Option A — `fs.volume(path)` (chosen)
- **Pros:** one `statfs`/`GetDiskFreeSpaceEx` call; portable; no process spawn.
- **Cons:** macOS purgeable space is not counted as free.

### Option B — run `df` through `proc`
- **Pros:** no runtime change.
- **Cons:** needs the `proc` capability, output differs per OS, slow.

## Decision

Add `fs.volume(path)` returning `Volume { total, free }` (`std.fs.24`).

## Rationale

A small, exact, capability-light call.

## Consequences

- Purgeable-space accounting is a possible later field.
- Native only (ADR 0865).

## Rejected alternatives

Option B is not portable and costs a spawn.
