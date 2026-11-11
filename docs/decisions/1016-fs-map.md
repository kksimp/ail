# 1016. `fs.map`: a read-only memory-mapped view of a file

- **Status:** Accepted
- **Date:** 2026-11-09
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** Standard library (`std.fs`)
- **Spec rules:** std.fs.36

## Context

`fs.read_bytes` copies the whole file into the heap before the program sees a byte, and `fs.read_range` copies each window. A tool that scans a large file (Clove, Husk, a log search, an index builder) pays for the copy, for the heap it fills and for the collections it triggers. C `mmap` and Rust `memmap2` give a view the system pages in on demand. AIL had no equivalent (the runtime used `mmap` only for its own regions).

## Options considered

### Option A — `fs.map(path) bytes`, a private read-only mapping laid out as a string object (chosen)
- **Pros:** the result is an ordinary `bytes`, so every `bytes` operation works with no new type and no new checker rule. The collector already ignores addresses outside its heap, so the view needs no collector change. Reading is bounds-checked like any `bytes`, so safe code stays memory-safe.
- **Cons:** the mapping is never unmapped before the process ends. A file truncated by another process while mapped can fault, as in C.

### Option B — a new `MappedFile` handle type with `close()`
- **Pros:** unmaps deterministically.
- **Cons:** a second type with its own indexing and slicing API, and a use-after-close hazard that safe code would have to rule out with ownership rules (`unique` handles, ADR 0096). `bytes` views would need a lifetime tied to the handle, and borrows are second-class (ADR 0061).

### Option C — a writable mapping returning a raw pointer
- **Pros:** in-place edits, as C does.
- **Cons:** needs `unsafe` and the `std.mem` pointer API plus a length, a flush rule and a Windows backend. Left to a follow-up slice (below).

## Decision

`fs.map(path str) bytes ! IoError` (std.fs.36). On Linux (x86-64, ARM64) and macOS the runtime reserves an anonymous block of one 16384-byte head plus the file's size rounded up to 16384, then maps the file `PROT_READ`, `MAP_PRIVATE | MAP_FIXED` over the part after the head. The last 16 bytes of the head hold the string header (length, character count), so the value points at a valid `bytes` whose data is the mapping itself. 16384 is a macOS page and a multiple of the Linux page, so one layout serves both. A directory fails with `is_dir`; an empty file takes the `read_bytes` path. Record and replay (ADR 0873) record the contents as for `read_bytes` and replay a heap copy. On Windows the result is a `read_bytes` copy for now.

## Rationale

Option A costs the language nothing new and keeps every `bytes` use working, which is the AI-first criterion (ADR 0076, no new concept to learn). The leak is bounded by the file sizes a program maps, and a program that maps in a loop can call `fs.read_range` instead.

## Consequences

- Clove and Husk can scan large files without the up-front copy.
- Native only (ADR 0865).
- **Follow-ups:** a writable mapping under `unsafe` (Option C); a real Windows mapping through `CreateFileMappingW` and `MapViewOfFile` (it needs a view at a chosen address, or a header outside the file's pages); unmapping when the value is collected, if a leak shows up in practice.

## Rejected alternatives

Option B costs a second API and lifetime rules the language avoids; Option C is unsafe and larger than one slice.

## Amendment (2026-11-10): a real view on Windows

- **Decided by:** AI (delegated, ADR 0076). **Criterion:** feature parity of the three hosts for 0.1.0; speed and memory on large files.

`fs.map` on Windows now returns a view, not a copy (`fs_map_win` in `compiler/rt/native/os.ail`). The file gets a read-only mapping object (`CreateFileMappingW`, `PAGE_READONLY`). Views start on a 65536-byte boundary there, so the head block is 65536 bytes: the runtime reserves head plus the size rounded up to 65536 to find a free range, releases it, maps the view at `base + 65536` with `MapViewOfFileEx` and commits the head at `base` with `VirtualAlloc`. The string header sits in the last 16 bytes of the head, as on the other hosts. Windows cannot replace part of a reservation in one step with these calls, so another thread may take the range between the release and the two placements; the runtime then tries again, up to eight times, and only then falls back to the `read_bytes` copy. Windows refuses to truncate or delete a mapped file, so the truncation fault of the other hosts cannot happen there.

Rejected:

- **`VirtualAlloc2` and `MapViewOfFile3` with placeholders:** no gap between release and placement, but they need Windows 10 1803 and live in `kernelbase.dll`, a new import for every program that maps. The retry closes the same gap with `kernel32` alone.
- **A header outside the file's pages (a `bytes` that points elsewhere):** a second `bytes` layout that every `bytes` operation would have to know.
- **Keeping the copy:** a 1 GiB file costs 1 GiB of heap and a full read before the first byte.

