# 0562. Native heap: reserved region, 16 KiB pages, size-class free lists, header word

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by: AI (delegated, ADR 0076)**
- **Topic:** Native heap and collector (lane 171)
- **Spec rules:** [native.build.1](../spec/23-native.md)
- **Design note:** [native-abi.md](../design/native-abi.md)
- **Supersedes the allocator of:** [0407](0407-bump-allocator-no-header.md)

## Context

The native runtime bump-allocated and never freed (ADR 0407): `binarytrees` at
depth 17 used 896 MiB, `fannkuch` 885 MiB. A collector needs object starts and
kinds, and the allocator must reuse freed memory.

## Options considered

A: keep bump chunks, add per-object headers, find objects by walking chunks.
B: a reserved region cut into 16 KiB pages, a page table, size classes for
small objects, page spans for large ones (chosen). C: a bitmap of object
starts beside the heap.

## Decision

B. At start `rt.init` reserves up to 16 GiB of address space with `mmap` (read-write,
no-reserve, halving until it succeeds; only touched pages cost memory), plus an
8-byte-per-page table and a mark stack. Pages are 16 KiB. Blocks up to 8192 bytes
(header included) come from 36 size classes (16 to 256 in steps of 16, then four
per doubling); a page holds blocks of one class, and free blocks are linked
through their first payload word. Larger objects take a span of whole pages.
Every object has the header word at pointer minus 8 fixed by the lane rules
(bits 0-7 kind, bit 8 mark, bits 16-63 payload bytes). `alloc(n, kind)` returns
zeroed memory; a page or block that was used before is zeroed on reuse. Kind 11
(pointer-free words) is added for `List[bool]` data and any scalar block.
Pages freed by a collection return to the page pool and serve any class or span.

## Rejected alternatives and criterion

Criterion: speed (4); freeing is required. A rejected: finding an object from an address
would be a linear walk. C rejected: a separate bitmap costs a second memory access per
allocation; the page table gives the same answer from an address in two loads.
A fixed region limits the heap to 16 GiB (smaller where `mmap` refuses);
exceeding it traps `run.limit.2`.

## Consequences

Allocation is a free-list pop or a bump within a page (about 20 instructions
plus the zeroing of reused blocks). `binarytrees` 17 runs in 11 MiB instead of
896 MiB and in half the time. Interior pointers resolve to their object through
the page table. Windows (lane 187) needs `VirtualAlloc` in place of `mmap` in `heap_reserve`.

## Addendum 2026-10-21: list growth and bulk copy

Profiling `binarytrees` (334 M instructions against 116 M for the C twin) put the collector at about 0% and list growth by `push` at 21%. Two runtime-wide changes follow, both in `core.ail`:

- **First grow to 16 elements.** A list that fills a block of fewer than 8 elements grows to 16 elements; from 8 it doubles as before. Cost: a list that reaches 5 to 8 elements holds up to 64 bytes more than before (a `List[int]` block of 128 bytes instead of 64), and for `List[bool]` 16 bytes. It saves one or two small grows per list. `binarytrees` 327.9 M to 316.7 M instructions, `fasta` 70.4 M to 68.7 M. Capacity is not observable (native.bce.3).
- **`copy_big`.** List growth and `extend` copy blocks of 256 bytes or more 64 bytes per round with two moving pointers (about 3 instructions per word, was about 5). `copy_bytes` itself is unchanged: it must stay under the inlining limit (ADR 0391), and wrapping it in a call made `fasta` 12% slower.

Rejected: `rep movsq` through a new prim (needs both back ends and a `lower.ail` slot, and gains nothing for the small copies that dominate string code); a 64-byte round inside `copy_bytes` (breaks inlining); starting every list at 8 elements (costs memory on the many one-element lists).
