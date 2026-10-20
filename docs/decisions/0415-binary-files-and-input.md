# 0415. Binary file and standard-input functions: `read_bytes`, `write_bytes`, `append_bytes`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 17 — Binary data
- **Spec rules:** [std.fs.13–14](../spec/17-std.md#files-stdfs), [std.io.5](../spec/17-std.md#standard-input-and-output-stdio)


## Context

`fs.read_text` fails for a file that isn't UTF-8, and `fs.write_text` cannot
write one. Binary formats need the bytes themselves.

## Options considered

### Option A — `_bytes` twins of the `_text` functions (chosen)
- **Tokens:** same length as the text form; the pairing is guessable.
- **AI understanding:** each function does one thing; the types say which.

### Option B — one `read`/`write` that takes a mode or a type
- **Cons:** a mode argument or generic return makes the type depend on a
  value; two names are simpler to check and to explain.

## Decision

Option A. `fs.read_bytes(path) bytes ! IoError`, `fs.write_bytes(path, data)`,
`fs.append_bytes(path, data)`, and `io.read_bytes() bytes` (all of standard
input). Their failures, `is_dir` reporting and effects (`fs`, `io`) are those
of the text functions, and `--trace effects` records them under their own names.

## Consequences

- `process` and `net` bodies as `bytes` follow in ADRs 0416–0417.
