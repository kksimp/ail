# 0992. `fs.scan_count`: live progress for `fs.scan`

- **Status:** Accepted
- **Date:** 2026-11-08
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** Standard library (`std.fs`)
- **Spec rules:** std.fs.27

## Context

`fs.scan` (ADR 0927) returns only when the whole tree is read, which takes minutes for a whole disk. Husk's window could show only "scanning root 1 of 5". A disk analyzer needs a progress bar that moves: the bytes seen so far against the volume's used bytes.

## Options considered

### Option A — a process-wide counter, `fs.scan_count()` (chosen)
- **Pros:** `fs.scan` keeps its signature; any task reads the counter while the scan runs; one atomic add per directory per word, on the walker that read it, so parallel subtrees need no lock. The block is made by the first scan, so other programs pay nothing.
- **Cons:** counts every scan in the program, so a caller takes the difference from before its scan; the two words are not one snapshot.

### Option B — a callback or channel argument to `fs.scan`
- **Pros:** per-scan numbers.
- **Cons:** a second signature; a call per directory from pool threads into user code, or a channel `std.fs` would have to know about; the native subset has no fallible or generic function values (ADR 0284 limits).

### Option C — the app counts with `fs.walk` first
- **Pros:** no new API.
- **Cons:** reads the tree twice.

## Decision

`fs.scan_count() fs.ScanCount { entries int, bytes int }` (std.fs.27): the entries and allocated bytes every `fs.scan` has listed so far. Runtime global slot 65 holds the two-word block, made lazily (compare-and-swap) by the first scan. `sc_walk` adds a directory's entries and bytes after reading it, with atomic adds; the reader uses atomic loads (ADR 0025, SeqCst). Both arms, macOS, Linux and Windows, go through `sc_walk`. Recorded and replayed like `fs.volume` (ADR 0873).

## Rationale

Option A is the cheapest correct one: nothing changes for callers of `fs.scan`, and the cost is two atomic adds per directory. The words are read separately because a consistent pair would need a lock or a sequence counter on a hot path, and progress display tolerates a few bytes of skew; the spec says so.

## Consequences

- Husk's scanner task polls it to drive the progress bar.
- Native only (ADR 0865).

## Rejected alternatives

Option B costs a second API and calls into user code from pool threads; Option C doubles the I/O.
