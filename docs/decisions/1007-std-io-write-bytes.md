# 1007. `io.write_bytes`: raw bytes to standard output

- **Status:** Accepted
- **Date:** 2026-11-09
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 17 — Binary data
- **Spec rules:** [std.io.6](../spec/17-std.md#standard-input-and-output-stdio)

## Context

`io.read_bytes()` (std.io.5, ADR 0415) reads all of standard input undecoded,
but a program can only write text to standard output. A compressor or any
binary filter (`clove c FILE -c`, `clove d -`) needs both ends.

## Options considered

### Option A — `io.write_bytes(data bytes)` (chosen)
- **Tokens:** pairs with `fs.write_bytes` and `io.read_bytes`; guessable.
- **AI understanding:** one function, one job; the type says bytes.

### Option B — add `io.read_all_bytes()` as well
- **Cons:** an alias of `io.read_bytes`, which already reads all of standard input; two names for one behaviour cost tokens and confuse.

### Option C — make `print` accept bytes raw
- **Cons:** `print(b)` shows the literal form `b"..."` today (type.bytes); changing it breaks that and adds a newline.

## Decision

Option A. `io.write_bytes(data bytes)` writes the bytes to standard output,
undecoded, no newline, in order with `print`; `exit` flushes it. Effect `io`.
No `io.read_all_bytes`: `io.read_bytes` (std.io.5, ADR 0415) is that function and
the read half of the pair. Output is binary on every target: Windows writes
with `WriteFile` on the standard handle (ADR 0608), so `\n` is never turned
into `\r\n`.

## Consequences

- Native-only (ADR 0865); not a recorded call (output, like `print`).

## Rejected alternatives

- `io.read_all_bytes` (option B): rejected as a duplicate of `io.read_bytes`.
- Raw bytes through `print` (option C): rejected because it changes `print`'s output for bytes and appends a newline.
