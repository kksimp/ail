# 0332. Give the read-only commands a front-end allocator

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Tooling speed (Stage 2), [ADR 0023](0023-rust-bootstrap.md)
- **Spec rules:** none (no behaviour changes; output is byte-identical)

## Context

Agents and the editor run `ail check`, `ail api` and `ail effects` after
nearly every edit, so their wall time is criterion 4 at its most frequent.
Profiling `ail check tools/edit/edit.ail` (4,800 lines, callgrind, plan branch
at c9799b5; [interpreter-performance.md](../design/interpreter-performance.md#front-end-lane-81))
put a quarter of its 163M instructions in `malloc` and `free`, for 178,000
blocks and 34 MB in all, and a third of its wall time in the kernel, faulting
in 13 MB of heap one 4 KiB page at a time. These commands make one pass over
the program and exit.

## Options considered

### Option A — a front-end allocator for read-only commands (chosen)
`bootstrap/src/bump.rs`, the process's global allocator. Off by default; `main`
switches it on for `check`, `api` and `effects` before any work. It reserves a
256 MiB anonymous region (Linux asks for transparent huge pages), serves a block
of up to 1 KiB from a free list of its 16-byte size class and anything else by
bumping a pointer, and drops frees of large blocks. When the region is full,
or on Windows, blocks come from the system allocator; the address decides
which allocator frees a block.
- **Pros:** 1.2–1.3x on the editor's `check`, `api` and `effects`; page faults
  fall from about 3,350 to 150; one new file and four lines in `main.rs`; no
  crate.
- **Cons:** about 120 lines of `unsafe`; a large block freed early is not
  reused, so peak memory can rise (bounded by the region); `ail check hello`
  pays about 0.1 ms to map the region.

### Option B — a general-purpose allocator crate (mimalloc and similar)
- **Pros:** faster everywhere, `ail run` too.
- **Cons:** a C dependency, against ADR 0009 and ADR 0023's crate rule, for a
  bootstrap that Stage 3 replaces.

### Option C — arenas inside the front end (interned names, `Rc`-free AST)
- **Pros:** fewer allocations at the source.
- **Cons:** touches every pass in `lexer.rs`, `parser.rs` and `check.rs` that
  ten lanes edit at once; not local.

### Option D — use it for `ail run` and `ail test` too
- **Pros:** the interpreter allocates heavily as well.
- **Cons:** long-running programs need large frees returned; the interpreter is
  the oracle (ADR 0032) and its memory limit (`--max-memory`) counts real use.

## Decision

`check`, `api` and `effects` use the front-end allocator; every other command
keeps the system allocator.

## Rationale

Criterion 4 (speed): the largest measured cost that a local change can
remove, with no effect on output and none on `ail run`.

## Consequences

- `ail check` on the editor: 33.0 ms → 26.4 ms best of 15 (Linux x86-64).
- A derive generator (`ct.derive.2`) runs inside `ail check` under this
  allocator; its large frees are not reused until exit.
- The macOS gain is unmeasured here; the designer's `tools/verify.sh` run is
  the check.

## Rejected alternatives

- **Allocator crate:** a C dependency for a temporary bootstrap (ADR 0009).
- **AST arenas:** a cross-cutting rewrite of shared files.
- **All commands:** long runs need memory returned; the oracle stays plain.
