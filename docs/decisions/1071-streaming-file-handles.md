# 1071. Streaming file handles: `std.file` and `io.read_exact`

- **Status:** Accepted
- **Date:** 2026-11-12
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** Standard library (`std.file`, `std.io`)
- **Spec rules:** std.file.1-9, std.io.7 ([17-std.md](../spec/17-std.md))
- **Resolves:** OQ-141

## Context

`std.fs` reads and writes whole files, plus `read_range`, `write_at` and `map`. There is no open handle, no line reader and no buffered writer: tailing a log needs hand-kept offsets, and writing a large CSV needs `append_text`, which reopens the file on every call. `std.io` has no exact-count read (OQ-141): `ail lsp` frames messages by `Content-Length` and cuts the body from a line read.

## Options considered

### Option A: handles in AIL over `std.fs` (chosen for files)
`std/file/file.ail` holds a path, a position and a buffer, and reads with `read_range`, writes with `write_at` or `append_bytes`.
- **Pros:** the same source on macOS, Linux and Windows, no runtime, lowering or checker change, no new global slot, no new shared scratch state (OQ on the two-thread scratch buffers of `os.ail` stays as it is). A handle holds no descriptor, so it cannot leak one and the file can be renamed or removed while a handle names it.
- **Cons:** one open per 64 KiB chunk or flush, not one per handle; a reader sees the file as it is now, not as it was when opened.

### Option B: descriptor handles in the runtime
- **Pros:** one open per handle, snapshot-like reads of a replaced file.
- **Cons:** three OS arms (open flags differ between macOS and Linux, Windows needs `CreateFileW`), a lowering entry, a record/replay entry, a leak path if a trap skips a close, and none of it measured to matter.

### Option C: a `File` that is a plain value closed with `defer` (as `std.term`)
- **Pros:** needs no parser change.
- **Cons:** a program that forgets the `defer`, or returns before it, loses buffered data silently; copies of the value diverge.

## Decision

Option A, with `Reader` and `Writer` as `unique` types (`own.uniq.1`): never copied, so one stream has one position, and the `Writer`'s `Drop` flushes on every path out of its scope, a trap included (`own.uniq.3`). A `drop` cannot report a failure, so `close()` and `flush()` are the calls that do (std.file.9). This is the scoped form that cannot leak: leaving the scope is the close.

Misuse by the program (negative offset or count, `seek` on an appending writer) traps through contracts; I/O failures are the `fs.IoError` of `std.fs`, so the error kinds follow whatever `std.fs` uses (the string kinds today, the enum when the error-kind lane lands it) without this module changing.

Reading: `read(n)`, `read_exact(n)` (position unchanged on `none`, so a follower can retry), `read_line()` (last unterminated line included) and `read_complete_line()` (a line only when its `\n` is written, for tailing). `tell`, `seek`, `size`. Writing: `create`, `append` (each flush is one `O_APPEND`-style append, safe with several writers), `update` (patch in place), `write`, `write_line`, `write_bytes`, `flush`, `tell`, `seek`, `close`.

`for line in r` is not offered: `for` takes a `List`, `Set`, range or `Channel` (generators are ADR 0965, 0.2). A `while` with `line := r.read_line()? else break` is the loop today, the form `net.Conn.read_line` already uses.

`io.read_exact(n int) bytes?` (std.io.7) is a runtime function: it reads through the stdin buffer that `read_line` and `read_bytes` share, so it needs no platform code of its own. It returns `none` when input ends first and consumes what it read; a negative `n` or one above 1 GiB traps. `ail lsp` can drop its line trick after the next seed (the seed compiler lacks the mapping).

The parser's `pub` rule now accepts `unique` (`pub unique struct`), the follow-up ADR 1012 left open.

## Rationale

AI understanding first (ADR 1065): one handle, one position, no way to forget a close, errors in the type. Human readability next: the same names as `std.fs` and `net.Conn`. Speed: a chunked reader and a 64 KiB write buffer remove the per-call open that was the complaint; Option B would save an open per chunk, unmeasured.

## Consequences

- `std.file` is a new AIL std module (registered in `load.ail`, `lower_mod.ail`, `lower_err.ail`, `names.ail`).
- Native-only (ADR 0865). Programs using `std.file` build with the tree's compiler until a seed with it is published.
- `io.read_exact` is not recorded by `--record`; a replay stops at it (`replay.cap.2`) until a wrapper is added.

## Rejected alternatives

Options B and C above. Also: an iterator type for `for` (needs generators); whole-file `lines(path) List[str]` (not streaming; `read_text` plus `split` does it).
