# 0929. `fs.read_range` and `fs.trash`: partial reads and recoverable delete

- **Status:** Accepted
- **Date:** 2026-11-04
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** Standard library (`std.fs`)
- **Spec rules:** std.fs.25, std.fs.26

## Context

Finding duplicates cheaply means hashing the first block of same-size files before whole files, and `read_bytes` reads everything. A tool that deletes the user's files should be able to do so recoverably; `fs.remove` is permanent.

## Options considered

### Option A — `read_range` and `trash` in std.fs (chosen)
- **Pros:** reads cost bytes asked for; the trash call is correct per OS (volume trash, put-back info, name collisions).
- **Cons:** `trash` has three OS arms.

### Option B — `read_bytes` then slice; `fs.rename` into `~/.Trash`
- **Pros:** no new API.
- **Cons:** reads whole files; a rename fails across volumes, skips put-back info and collisions.

## Decision

Add `fs.read_range(path, offset, len)` (`std.fs.25`) and `fs.trash(path)` (`std.fs.26`). `trash` never deletes permanently and never copies; `fs.remove` remains the permanent call.

## Rationale

Recoverable by default is the safe behaviour for any app that removes user data, so no permanent-delete default is offered.

## Consequences

- Husk's only delete path is `fs.trash`.
- Native only (ADR 0865); the macOS arm first.

## Rejected alternatives

Option B is wrong on other volumes and loses put-back.
