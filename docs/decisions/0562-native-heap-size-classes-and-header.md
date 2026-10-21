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
