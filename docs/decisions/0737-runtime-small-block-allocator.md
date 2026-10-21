# 0737. `ail run` and `ail test` allocate small blocks from per-thread free lists

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 13 — Standard library (the interpreter's values)
- **Spec rules:** none (no language behaviour changes)
- **Builds on:** [ADR 0332](0332-front-end-allocator.md) (the front-end allocator), [ADR 0658](0658-native-json.md)

## Context

Every interpreter value is a small heap block: a `Str` is an `Rc<str>`, a
struct or list is an `Rc<Vec<Value>>`. After ADR 0658 moved JSON into Rust,
a callgrind profile of `json_roundtrip` showed `malloc` and `free` at 46% of
the instructions, and 20% of `lines_split`. glibc's allocator does
about 100 instructions per allocate/free pair and takes its arena lock when
the thread cache runs dry.

## Decision

In `ail run` and `ail test`, the global allocator (`bump.rs`) serves blocks of
up to 1,024 bytes (16-byte size classes, alignment up to 16) from a
per-thread free list, refilled from a per-thread 256 KiB chunk of one
16 GiB `MAP_NORESERVE` region. A free pushes the block on the freeing
thread's list, whichever thread allocated it. A thread that exits hands its
lists and the rest of its chunk to a shared pool. Larger blocks, higher
alignments, a failed reservation, and any block outside the region stay with
the system allocator, which also frees them. `AIL_SYSTEM_ALLOC=1` keeps
the system allocator for comparison. Memory in the region is reused by its
size class and not returned to the system before exit, like the front-end
mode of ADR 0332; the resident size of the benchmark programs is within 10%
of the system allocator's.

`str.split` and `lines` also size their lists once, split on a one-character
separator searches for the character, and `list.join` appends into one buffer
instead of making a string per element.

## Measured (Linux x86-64, best of 7, `tools/perfbench/race.py`)

| program | before | after | CPython |
|---|---|---|---|
| json_roundtrip | 0.218 s | 0.163 s | 0.083 s |
| map_ops | 0.100 s | 0.075 s | 0.073 s |
| print_many | 0.139 s | 0.095 s | 0.179 s |
| lines_split | 0.307 s | 0.295 s | 0.274 s |
| binarytrees | 0.726 s | 0.701 s | 0.358 s |
| wordfreq | 0.209 s | 0.254 s (noise: 0.27 s both ways in a direct 8-run average) | 0.124 s |

Instructions (callgrind, `json_roundtrip` with 2,000 records): 128.5 M to 77.2 M.
`./ail selftest` passes.

## Rejected alternatives

- **A different global allocator crate (mimalloc, jemalloc).** Custom tooling
  by default (ADR 0009), and the region scheme already exists for ADR 0332.
- **Return memory to the system.** It would need per-chunk accounting for
  little gain in a process that runs one program.
- **Intern or slice strings instead of allocating every piece.** It changes
  `Value::Str` everywhere; kept for a later step if allocation is still the
  cost.
