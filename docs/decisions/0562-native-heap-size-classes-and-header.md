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

## Amendment (2026-10-21): the page size is 16,448 bytes

Lane 252 bisected a native `matmul` slowdown to this heap: cachegrind D1 read
misses at size 200 went from 1.10 M (bump allocator) to 9.66 M. Cause: with
16,384-byte pages every page of a size class has the same layout, and 16,384
bytes is a whole number of cache-set periods (256 lines, 4 times the 64 sets
of a 48 KiB 12-way L1). The rows of a table (`List[List[int]]`, one block
each, 4 or so per page) therefore fell on the same few sets in every page, and
a column walk (`b[k][j]` over `k`) evicted its own lines: 200 rows landed on 9
sets, 23 lines per set against 12 ways. Padding the class sizes did not help,
because the repeat comes from the page, not the stride inside it.

The page is now 16,448 bytes (257 lines): each page starts one cache set
later than the one before, so consecutive pages spread a table's rows over all
sets. The division by the page size in the page-table lookups is by a
constant, which the backend turns into a multiply, and the block reciprocal
stays exact (offsets are below 2^15). Large spans are whole pages, so their
stride is 257 lines per page, also spread.

Results (D1 read misses, cachegrind, `matmul`):

| size | bump allocator | 16 KiB pages | 16,448-byte pages |
|---:|---:|---:|---:|
| 100 | 146 K | 820 K | 143 K |
| 150 | 465 K | 3.64 M | 456 K |
| 200 | 1.10 M | 9.66 M | 1.45 M |
| 250 | 20.6 M | 16.1 M | 2.09 M |
| 400 | 121 M | 135 M | 104 M |

`matmul 400` runs in 0.076 s (was 0.164 s). Rejected: odd multiples of 16 for
the class sizes (no change in misses), separate header and data arenas (not needed: the
footprint of a column walk fits once the pages stop repeating), and a random colour per page (the slack
in a page is too small to colour).

**Amended 2026-11-18 by [ADR 1158](1158-heap-page-of-265-cache-lines.md):** the page is 16,960 bytes (265 lines). Pages still start on every cache set in turn; the step of 9 lines keeps neighbouring blocks from aliasing in the low 12 address bits.

## Addendum 2026-10-21: list growth and bulk copy

Profiling `binarytrees` (334 M instructions against 116 M for the C twin) put the collector at about 0% and list growth by `push` at 21%. Two runtime-wide changes follow, both in `core.ail`:

- **First grow to 16 elements.** A list that fills a block of fewer than 8 elements grows to 16 elements; from 8 it doubles as before. Cost: a list that reaches 5 to 8 elements holds up to 64 bytes more than before (a `List[int]` block of 128 bytes instead of 64), and for `List[bool]` 16 bytes. It saves one or two small grows per list. `binarytrees` 327.9 M to 316.7 M instructions, `fasta` 70.4 M to 68.7 M. Capacity is not observable (native.bce.3).
- **`copy_big`.** List growth and `extend` copy blocks of 256 bytes or more 64 bytes per round with two moving pointers (about 3 instructions per word, was about 5). `copy_bytes` itself is unchanged: it must stay under the inlining limit (ADR 0391), and wrapping it in a call made `fasta` 12% slower.

Rejected: `rep movsq` through a new prim (needs both back ends and a `lower.ail` slot, and gains nothing for the small copies that dominate string code); a 64-byte round inside `copy_bytes` (breaks inlining); starting every list at 8 elements (costs memory on the many one-element lists).

## Addendum 2026-10-21: the scan loop filters before it looks up

`gc_scan_range` tests each word against the heap bounds (read once per range) and calls `gc_mark` only for a word inside the heap. Before, every word of a `List[int]` block, which is scanned like any `KIND_LDATA` block, cost two calls (`gc_mark`, `gc_find`) to learn it was not an address. A program that keeps a 1 M-element `List[int]` live through 40 collections went from 190.3 M to 86.2 M instructions; `wordfreq` -1.4%, `binarytrees` and `fasta` unchanged within 0.2%. The collector's behaviour is the same: a word outside the heap was never marked.
