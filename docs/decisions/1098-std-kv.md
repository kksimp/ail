# 1098. `std.kv`: a key-value store in one file

- **Status:** Accepted
- **Date:** 2026-11-13
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 17 — Standard library
- **Spec rules:** std.kv.1-9 ([17-std](../spec/17-std.md))

## Context

The AI-first review (item 17) found that programs that need to keep a few
settings, a cache or a small table between runs hand-write a file format, or
write a whole JSON file on every change and lose it when stopped half way.
Streaming file handles (ADR 1071) and `fs.write_at` / `fs.truncate` (ADR 0977)
give the pieces for a crash-safe log.

## Decision

`std.kv`, written in AIL (`std/kv/kv.ail`), imported with `use std.kv`:
`open(path) Store ! fs.IoError | KvError`; `Store` is `unique` with `get`,
`has`, `keys`, `len`, `set`, `remove`, `compact` and `close`. Keys are `str`,
values `bytes`. The data lives in a `Map[str, bytes]` in memory; the file is an
append-only log of records (lengths, key, value, CRC-32) after a 7-byte header.
Each change is one write at the end of the intact log; `open` replays the log and
cuts off an unfinished or corrupt tail, so a stop loses at most the change in
progress. `compact` writes a new log beside the file and renames it over, and
`set` and `remove` do it once the waste passes 1 MiB and the live data. Pure AIL
over `std.fs` and `std.hash`: no lowering or runtime change, the same on macOS,
Linux and Windows.

## Rejected alternatives

- **Rewrite the whole file on each change** (temp file and rename). Safe but
  every `set` costs the whole store; the log makes it cost one record.
- **A B-tree or hash file read in place.** Needs page management and no
  in-memory copy; far more code for data sized for a config or a cache. A later
  module can add it.
- **Syncing on every `set`.** Makes each change power-safe but costs a disk
  flush per call; an explicit `sync()` lets the program choose. Cost: a program
  that forgets it may lose the last changes in a power failure (std.kv.4).
- **Syncing the directory too.** Needs a directory handle on Windows and extra
  per-OS code; a new file may be missing after a power failure until then.
- **Text values (`Map[str, str]`).** Forces an encoding step for binary data;
  `bytes` loses nothing and `text()` / `to_bytes()` convert.
- **Replacing a corrupt record in the middle and reading on.** Cannot tell a
  stop from damage, and the later records may depend on the lost one; the log
  ends at the first bad record.
- **`Store` as a copyable struct.** Two copies would diverge from the file.

## Measurements

None: no twin exists in tokbench.
