# 0927. `fs.scan`: one bulk walk with allocated size, inode and kind

- **Status:** Accepted
- **Date:** 2026-11-04
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** Standard library (`std.fs`)
- **Spec rules:** std.fs.23

## Context

Husk, a disk-space analyzer written in AIL, must size millions of entries quickly. `fs.walk` lists file paths only, and `fs.metadata` opens and `fstat`s each path, follows links, and has no allocated size or inode.

## Options considered

### Option A — `fs.scan` returning entries with metadata (chosen)
- **Pros:** one pass; the runtime can use bulk calls (`getattrlistbulk` on macOS, `getdents64` plus `statx` on Linux) and read subtrees in parallel; allocated size and inode give honest sizes and hard-link dedup.
- **Cons:** the result is held in memory (about 100 bytes per entry).

### Option B — `fs.walk` then `fs.metadata` per path
- **Pros:** no new API.
- **Cons:** an open, a stat and a close per file; follows links; no allocated size, inode or link count.

### Option C — a visitor callback walker
- **Pros:** streams without holding the list.
- **Cons:** callbacks fit AIL badly (no closures across the walk, ordering under parallel reads).

## Decision

Add `fs.scan(root, same_device)` (`std.fs.23`).

## Rationale

Allocated size is what a delete frees, and a deterministic, link-safe tree is what an analyzer and an AI agent both need. Parallel reads stay invisible to the program.

## Consequences

- A scan of a home folder drops from a stat per file to a bulk read per directory.
- Native only (ADR 0865): macOS first, then Linux and Windows arms; the Windows arm must pass before a release (ADR 0925).
- Follow-up: a streaming variant if memory proves a problem.

## Rejected alternatives

Option B is too slow and lacks the data; option C is awkward to specify.
