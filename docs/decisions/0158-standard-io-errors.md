# 0158. Standard failures are `fs.IoError` and `proc.ProcError` with a short `kind`

- **Status:** Accepted; `kind` is an enum since [ADR 1068](1068-std-error-kinds-are-enums.md)
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 8 — Errors; standard library
- **Spec rules:** [std.err.1](../spec/17-std.md)

## Context

The standard modules' failures need a type an AI can match on and print. Rust's
`io::ErrorKind` has dozens of kinds, and platforms differ: removing a
directory with `unlink` is `EISDIR` on Linux but `EPERM` on macOS.

## Options considered

### Option A — one error struct per module with a `kind` string (chosen)
`error struct IoError { path str, kind str }`, `kind` one of eight short names.
- **Pros:** prints as `fs.IoError(path: "a.txt", kind: "not_found")`, which
  says everything; matched with literal patterns (`match e.kind`); adding a
  kind later breaks no program.
- **Cons:** a misspelt kind in a pattern isn't caught.

### Option B — an `error enum IoError` with a variant per kind
- **Pros:** exhaustive matching, misspellings caught.
- **Cons:** the path must be repeated in every variant; adding a kind is a
  breaking change; programs mostly test one kind and print the rest.

### Option C — the OS error number and message
- **Cons:** differs by platform, so output isn't reproducible (wishlist 6.3).

## Deciding criterion

Tokens **tie** (`e.kind == "not_found"` and `IoError.NotFound` cost about the
same). **AI understanding** decides: the text form is complete and
self-explaining, and the kinds are the same on every platform, so a program's
output is too. The bootstrap checks for a directory itself where platforms
disagree.

## Decision

`fs` failures are `fs.IoError(path, kind)` and a process that can't start is
`proc.ProcError(cmd, kind)`. `kind` is one of `not_found`, `permission`,
`exists`, `is_dir`, `not_dir`, `not_empty`, `invalid_utf8` and `other`.

## Consequences

- The brief's four kinds (`not_found`, `permission`, `is_dir`, `other`) are
  extended by four that `make_dir`, `remove` and `read_text` produce.
- If enums with shared fields arrive, Option B can be revisited.

## Rejected alternatives

- **Variant per kind:** repeats the path, and every new kind breaks matches.
- **OS error numbers:** platform-dependent output.
