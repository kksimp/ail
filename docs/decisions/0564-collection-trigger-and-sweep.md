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
