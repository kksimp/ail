# 0974. `std.fs` attributes: mode, modification time, symbolic links, truncate

- **Status:** Accepted
- **Date:** 2026-11-07
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** Standard library (`std.fs`)
- **Spec rules:** std.fs.27, std.fs.28, std.fs.29, std.fs.30, std.fs.31, std.fs.32, std.fs.33

## Context

An archiver (Clove) must put files back as they were: executable bits, modification times and symbolic links. Extracting without a second copy of the data on disk means shrinking the archive as its blocks are consumed. `std.fs` could read the size and time of a file but set nothing, had no view of links beyond `scan`'s `is_link`, and could not cut a file short or write into the middle of one (extracting a block from the end of an archive into the middle of its file).

## Options considered

### Option A — six small calls in `std.fs` (chosen)
- **Pros:** each is one system call per platform; a program says what it needs and nothing more; `metadata` keeps its shape.
- **Cons:** three OS arms each; Windows has no clean mode or link model, so those arms are reduced (read-only bit; links need a privilege).

### Option B — a `fs.Attrs` struct with `get_attrs` / `set_attrs`
- **Pros:** one pair of calls.
- **Cons:** every set writes every field, so a caller that wants the time alone must read first; a partial failure is unreportable; the struct grows with every platform detail.

### Option C — leave it to `proc.run("chmod")`, `touch`, `ln`
- **Pros:** no runtime code.
- **Cons:** a process per file; different tools per platform; no Windows.

## Decision

Add `fs.mode`, `fs.set_mode`, `fs.set_modified`, `fs.read_link`, `fs.symlink`, `fs.truncate` and `fs.write_at` (`std.fs.27` to `std.fs.33`), in `compiler/rt/native/fsattr.ail`. Modes are the twelve permission bits as an `int`; times are the milliseconds `metadata` reports. Windows: `mode` reports 438 or 292 from the read-only attribute and `set_mode` changes only that attribute; `symlink` uses the unprivileged flag and fails with `permission` without developer mode; `read_link` fails with `other` until written.

## Rationale

Small named calls cost fewer tokens at the call site than a struct round trip (ADR 0076, criterion 1), and each fails on its own with an `fs.IoError` the caller can read.

## Consequences

- Clove stores and restores mode, mtime and links, and extracts in place by truncating the archive.
- Native only (ADR 0865). All three targets have arms; the Windows arms are the reduced ones above.

## Rejected alternatives

Option B writes what it did not mean to; Option C has no Windows arm and costs a process per file.
