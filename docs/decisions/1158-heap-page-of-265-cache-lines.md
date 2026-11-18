# 1158. A heap page of 265 cache lines

- **Status:** Accepted
- **Date:** 2026-11-18
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked on 2026-11-17 for lanes on the speed gaps still open
- **Topic:** 25 — Optimization strategy
- **Spec rules:** none (layout of the native heap is not observable)
- **Amends:** [ADR 0562](0562-native-heap-size-classes-and-header.md) (its 2026-10-21 amendment: a page of 257 lines)

## Context

With the four-element passes of [ADR 1144](1144-four-element-vector-passes.md)
`c[i] = a[i] * k + b[i]` over lists of 1000 floats took 87 ms on ail-linux
where a C copy of the same loop took 70 to 80 ms, or 100 to 110 ms depending
on where its arrays lay (`vwide-diag-linux.txt`, `vwide-diag2-linux.txt`).
The counter `ld_blocks_partial.address_alias` showed 200 million events.
An x86-64 core compares a load with the stores still in flight by the low 12
bits of their addresses first. A load from `a` that follows, in those bits, a
little behind a store to `c` waits for it. Moving `c` in the C copy by its
offset from `a` modulo 4096:

| `c - a` mod 4096 | 0 | 32 | 64 | 128 | 256 | 384 | 512 | 1024 | 2048 | 3072 | 4064 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| ms | 79.9 | 105.4 | 107.9 | 107.8 | 110.3 | 85.9 | 79.2 | 79.0 | 81.8 | 86.1 | 82.0 |

So 32 to 256 bytes ahead costs 30%, and 0 or anything from 512 up is free.

The heap put lists exactly there. A page is 16,448 bytes, 257 cache lines,
so that each page starts one cache set after the one before (ADR 0562). A
block of 8192 bytes sits two to a page, and a larger one starts its own
pages: lists made one after the other are 64 bytes apart modulo 4096 for
each page between them. The three lists of the loop were at 0, 0 and 64
(1000 elements) and 0, 128 and 256 (4000 elements).

## Options considered

### Option A — a page of 265 cache lines, 16,960 bytes (chosen)

Each page then starts 9 lines (576 bytes modulo 4096) after the one before.
Either list may be the one written, so both `576 k mod 4096` and its
negative must stay out of 1 to 511 for pages `k` apart: 576, 1152, 1728,
2304, 2880, 3456 are clear both ways, and 7 pages apart is the first that is
not (64 ahead when the list written was made first). 9 is odd, so pages
still start on all 64 sets in turn, which is what ADR 0562 needs for the
rows of a table.

- Pro: one constant. Small blocks and large spans are both covered, and no
  reader of the page table changes.
- Con: a page is 3.1% larger, and an 8192-byte block's page has 576 unused
  bytes (it had 64).
- Con: lists exactly 7 or 14 pages apart, with the later one read and the
  earlier one written, still alias. No step is free of such pairs.

### Option B — start a large block at an offset inside its first page

The header of a span moves by a multiple of 64 bytes chosen from its page
number.

- Con: the 1000-element case is a small block (8192-byte class), which this
  does not move.
- Con: six places find a span's header at its page start (the sweep, the
  verifier, the census, `gc_find`); each must learn the offset.

### Option C — pages of 4096-byte multiples, every block at the same offset

Distance 0 is free too, and it is what a C program gets from `mmap`.

- Con: it brings back the repeat ADR 0562 removed (a column walk over a
  table's rows evicting its own lines).

### Option D — fix it in the loop

Nothing in the loop can change how far apart two lists are.

## Decision

Option A: `PAGE := 16960` in `compiler/rt/native/heap.ail`. Every size
derived from it uses the name, the block reciprocal stays exact (offsets
are below 2^15), and the division by the page size is still by a constant.

## Consequences

ail-linux (x86-64, 2026-11-18, `vwide3-job-linux-a426371.txt`), the change
against the main it merged, best of 5, ms. `address_alias` is
`ld_blocks_partial.address_alias` for the whole run.

| program, N | 16,448 | 16,960 |
|---|---:|---:|
| axpy 1000 | 87.4 | 86.5 |
| axpy 2000 | 254.6 | 237.5 |
| axpy 4000 | 509.9 | 471.5 |
| axpy 8000 | 999.5 | 959.1 |
| vint (int lanes) 1000 | 124.1 | 117.8 |
| vint 4000 | 538.7 | 520.3 |
| mapcomp 1000 | 157.0 | 160.5 |
| mapcomp 4000 | 530.6 | 535.9 |
| axpy 1000 `address_alias` | 200.8 M | 11.8 M |
| axpy 4000 `address_alias` | 797.8 M | 12.7 M |

The alias events are gone and the loops that stream two or three lists gain
4 to 8%, less than the 30% of the C copy: at 1000 elements the loop was not
held back by them. `mapcomp` is within 2% either way. Mac (arm64): level
within the 3% noise of the run (axpy 4000 255.3 against 253.4), as
expected, since the effect is the x86-64 store buffer's. Peak memory of the
compiler building itself is unchanged (32 MB). 47 heap, list and collector
programs match their goldens, also under `AIL_GC_VERIFY=1`, on both machines.

## Rejected alternatives

Options B, C and D above. Other steps: of the odd steps below 64, 9 and 55
keep the most neighbours clear both ways (6 pages); 21 (277 lines) is clear
for 42 pages when the list written is the later one and aliases at 3 pages
when it is the earlier one. An even step never aliases at a multiple of 8
lines, but pages then start on 8 sets or fewer and tables of rows of 2048
bytes fall back on a few sets (ADR 0562).
