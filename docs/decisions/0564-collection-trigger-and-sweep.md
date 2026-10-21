# 0564. Collection trigger, sweep and measurements

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by: AI (delegated, ADR 0076)**
- **Topic:** Native heap and collector (lane 171)
- **Spec rules:** [native.build.1](../spec/23-native.md)
- **Design note:** [native-abi.md](../design/native-abi.md)

## Decision

A collection runs when the bytes allocated since the last one exceed
`max(8 MiB, live bytes after the last)`, so the heap is about twice the live
data plus 8 MiB. Marking uses an explicit stack of block addresses (a 512 MiB
reservation, committed as used; a block is queued once, by its mark bit), so
deep structures do not overflow the machine stack. Sweep walks the pages in
address order: a small page with no marked block is freed whole; otherwise
its unmarked blocks are cleared and rebuilt into the class's free list, lowest
address first. Large spans are freed or kept whole.

## Measurements (Linux x86-64, min of 3)

| program | size | before s | before MiB | after s | after MiB |
|---|---:|---:|---:|---:|---:|
| fannkuch | 10 | 1.078 | 885 | 0.889 | 11 |
| binarytrees | 17 | 1.025 | 896 | 0.503 | 11 |
| mandelbrot | 1000 | 0.142 | 33 | 0.138 | 11 |
| fasta | 5000000 | 0.189 | 87 | 0.173 | 11 |
| collatz, nbody, spectralnorm, sieve, matmul | | unchanged | | within 3% | |

`binarytrees 20` runs in 85 MiB (its live arena). `bootstrap/tests/native/heavy/gc_10gb.ail`
allocates over 10 GB of short-lived lists and stays at 10 MiB.

## Rejected alternatives and criterion

Criterion: speed (4) after memory. A generational or incremental collector was left
until stack maps (ADR 0563) make roots cheap to find. A fixed trigger (say 64 MiB) wastes
memory in small programs and thrashes in large ones.

## Consequences

The pause is proportional to the heap in pages plus live objects; programs with
very large live data see long pauses until a generational design (a later ADR).

## Allocator speed-up (amendment, 2026-10-21)

Lane 191 measured `binarytrees` at 7.8x gcc and suspected the allocator's
call cost. A profile (callgrind, `binarytrees 14`) showed `rt.alloc` at 3% of
instructions and the page search in `heap_pages` at 25%: the arena lists of
`binarytrees` grow to many pages, and each growth scanned the page table from
the start and zeroed pages that `list_grow` then overwrote. Changes:

- `heap_pages` steps over whole large spans by their page count, keeps the
  hint at the lowest free page, and remembers the smallest run length known
  not to fit until the next sweep.
- `alloc_raw(n, kind)` skips zeroing for blocks the caller fills before
  reading (list element blocks, string objects, the string builder, the sort
  buffer). The collector scans a list's element block only up to the list's
  length, so stale words past it keep nothing alive.
- `alloc` keeps a straight-line fast path for small objects (free-list pop or
  bump), calling `alloc_slow` otherwise. It is cheap but was not the
  bottleneck.

Results (Linux x86-64, min of 5): `binarytrees 17` 0.357 to 0.320 s with 16%
fewer instructions at depth 14; `binarytrees 20` 7.0 to 3.8 s and 85 to 61 MiB;
the 10 GB test 9.8 to 3.3 s; `fannkuch`, `sieve`, `mandelbrot` 3-10% faster;
others unchanged. A bump-pointer fast path in the generated code itself, and a
`list_new_for` that allocates header and first block together, are left until
the code generator folds constant offsets into addressing: today the fast path
spends most of its instructions on `add` and `imul` for addresses.

## Cheaper collections for string-heavy programs (amendment, 2026-10-21)

Lane 235's `lines_split` (200,000 log lines split into fields) spent about a
quarter of its instructions in allocation and collection. Changes:

- A small page's table entry holds the block size and the reciprocal
  `ceil(2^32 / size)`, so the collector finds a block start with a multiply
  instead of a division and a call.
- The sweep makes one pass per page (clear marks, chain free blocks, count
  live) instead of two.
- The trigger is `max(16 MiB, 2 x live bytes)`: the heap grows to about three
  times the live data before a collection, in exchange for fewer of them.
- A small page is no longer zeroed when taken; a page that was used before is
  marked dirty and zeroed block by block, only when a zeroed block is asked
  for. Pages taken from the top of the heap are clean.
- The fast path indexes its per-class slots with one shift.

Measured with callgrind on a native `lines_split` (strings, `lines`, `split`;
no map): 2.91 G to 2.23 G instructions (-23%), 0.265 s to 0.20 s. The
collector went from about 600 M to 235 M instructions. The rest is the string
functions and the address arithmetic in the generated code of `alloc` (about
100 instructions per call), which lane 235's displacement folding should
shrink. Resident size grows by the larger trigger: the 10 GB test runs in
16 MiB (was 10), `binarytrees 20` in 80 MiB (was 61).
