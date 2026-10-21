# 0446. Zip archives: store and deflate, deterministic, no zip64, no escaping the target

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 21 — Standard library architecture
- **Spec rules:** [std.compress.5](../spec/17-std.md#compression-stdcompress)

## Context

Zip is the common archive format for sharing a directory. Reading untrusted archives is a classic attack ("zip slip", bombs).

## Options considered

### Option A — store and deflate, no zip64, deterministic writer, strict reader (chosen)
- **Pros:** covers real archives under 4 GiB; same directory gives the same bytes (diffable, cacheable); the reader checks every CRC-32 and rejects hostile names before writing.
- **Cons:** archives over 4 GiB or 65535 files fail with `too_big`; no encryption, bzip2 or LZMA entries.

### Option B — add zip64 now
- **Cons:** rarely needed for the targets; more format surface to test. Additive later.

### Option C — tar.gz instead
- **Cons:** not what the brief or most users exchange; may follow.

## Decision

Option A. Writer: names sorted, fixed 1980-01-01 timestamp, UTF-8 flag, Unix mode 644, symbolic links skipped. Reader: from the central directory, name check (`unsafe_path`), `too_big` against the 2^30 total cap, `unsupported` for encryption and other methods. Cross-checked with `unzip -t`.

## Consequences

- Permissions and timestamps of files are not preserved (a deliberate loss for reproducibility).
- `zip_dir` reads every file into memory before writing; large trees wait for `bytes` and streaming.

## Rejected alternatives

- Zip64 now (option B): rejected because it is rarely needed for the targets and adds format surface to test. It can be added later.
- `tar.gz` instead (option C): rejected because it is not what the brief or most users exchange. It may follow.
