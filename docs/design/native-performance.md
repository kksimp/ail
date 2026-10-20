# Native performance: the first drag race

Non-normative. Lane 120's measurement of `ail build` output (ADRs 0384–0390)
against `clang -O2`, `gcc -O2` (both with `-ffp-contract=off`), CPython and
the AIL interpreter, on 2026-10-20. Every native column printed the
interpreter's bytes (drag.out.1).

## Environment

- Linux x86-64, Intel Xeon @ 2.10 GHz, 4 logical cores (a shared cloud VM:
  expect a few percent of noise).
- clang 18.1.3, gcc 13.3.0, CPython 3.12.
- The backend as of lane 120 slice 4: SSA MIR, linear-scan allocation,
  checked arithmetic and bounds, no optimisation passes yet.

## Large sizes (best of 3, seconds)

Run with `python3 -I tools/perfbench/large.py tools/perfbench/programs DIR`
after building each program into `DIR` with `ail build`. Sizes are chosen so
C runs for 40–240 ms and process start-up stops mattering.

| program | size | native | clang -O2 | gcc -O2 | python | native/clang | native/gcc | native/python |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| collatz | 1000000 | 0.409 | 0.152 | 0.234 | 9.171 | 2.69 | 1.74 | 0.04 |
| nbody | 1000000 | 0.084 | 0.051 | 0.047 | 5.628 | 1.65 | 1.79 | 0.02 |
| spectralnorm | 1000 | 0.119 | 0.047 | 0.051 | 5.985 | 2.54 | 2.32 | 0.02 |
| mandelbrot | 1000 | 0.133 | 0.078 | 0.080 | 3.538 | 1.72 | 1.66 | 0.04 |
| fannkuch | 10 | 0.853 | 0.237 | 0.215 | 8.108 | 3.60 | 3.97 | 0.11 |
| sieve | 30000000 | 1.158 | 0.174 | 0.197 | 5.420 | 6.64 | 5.87 | 0.21 |
| matmul | 400 | 0.234 | 0.052 | 0.064 | 3.670 | 4.53 | 3.64 | 0.06 |
| binarytrees | 17 | 0.765 | 0.142 | 0.109 | 3.920 | 5.41 | 7.03 | 0.20 |
| fasta | 5000000 | 0.209 | 0.042 | 0.063 | 0.980 | 4.95 | 3.30 | 0.21 |

**Against Python:** native AIL wins every program, by 5x (fasta, sieve,
binarytrees) to 50x (nbody, spectralnorm).
**Against C:** native AIL loses every program, by 1.65x (nbody) to 7x
(binarytrees). The float kernels are closest; list-heavy code is furthest.

## After the first optimisations (ADR 0391)

The same race after lifetime holes, MIR folding and CSE, inlining, inline
`push` and byte-sized `List[bool]`:

| program | size | native | clang -O2 | gcc -O2 | python | native/clang | native/gcc | native/python |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| collatz | 1000000 | 0.261 | 0.171 | 0.200 | 9.551 | 1.53 | 1.30 | 0.03 |
| nbody | 1000000 | 0.103 | 0.053 | 0.054 | 5.364 | 1.93 | 1.91 | 0.02 |
| spectralnorm | 1000 | 0.073 | 0.051 | 0.043 | 5.585 | 1.42 | 1.71 | 0.01 |
| mandelbrot | 1000 | 0.095 | 0.074 | 0.081 | 3.188 | 1.28 | 1.17 | 0.03 |
| fannkuch | 10 | 0.847 | 0.211 | 0.177 | 7.840 | 4.02 | 4.78 | 0.11 |
| sieve | 30000000 | 0.209 | 0.174 | 0.204 | 5.718 | 1.20 | 1.02 | 0.04 |
| matmul | 400 | 0.186 | 0.044 | 0.046 | 3.418 | 4.29 | 4.02 | 0.05 |
| binarytrees | 17 | 0.595 | 0.172 | 0.106 | 3.644 | 3.47 | 5.63 | 0.16 |
| fasta | 5000000 | 0.183 | 0.055 | 0.061 | 1.079 | 3.33 | 3.01 | 0.17 |

Instructions retired at the `.args` sizes (callgrind), before → after:
collatz 326M → 188M, sieve 367M → 144M, binarytrees 644M → 381M, fannkuch
635M → 442M, matmul 285M → 223M, fasta 151M → 75M, nbody 106M → 92M,
spectralnorm 151M → 95M, mandelbrot 74M → 44M. Wall times on this shared VM
vary by about 10% between runs; nbody's 1.93 is within that of its earlier
1.65.

**Against C:** sieve is level with gcc, and collatz, spectralnorm and
mandelbrot are within 1.7x. fannkuch, matmul, binarytrees and fasta stay
at 3–5.6x.

## perfbench sizes (`tools/perfbench/run.sh --all`, min of 5)

The `.args` sizes, now passed to every column. C finishes most in under
10 ms, so these ratios mostly measure start-up; the interpreter column shows
what `ail build` buys.

| program | ail run | native | clang | gcc | python | native/python | ail run / native |
|---|---:|---:|---:|---:|---:|---:|---:|
| collatz | 0.374 | 0.032 | 0.012 | 0.019 | 0.756 | 0.04 | 12x |
| parsum | 0.165 | 0.036 | – | – | 0.736 | 0.05 | 5x |
| nbody | 0.731 | 0.005 | 0.003 | 0.003 | 0.245 | 0.02 | 146x |
| spectralnorm | 0.576 | 0.007 | 0.004 | 0.004 | 0.338 | 0.02 | 82x |
| mandelbrot | 0.607 | 0.007 | 0.006 | 0.006 | 0.210 | 0.03 | 87x |
| fannkuch | 0.844 | 0.088 | 0.022 | 0.022 | 0.701 | 0.13 | 10x |
| sieve | 1.130 | 0.071 | 0.012 | 0.017 | 0.424 | 0.17 | 16x |
| matmul | 0.586 | 0.020 | 0.004 | 0.004 | 0.367 | 0.05 | 29x |
| binarytrees | 1.483 | 0.092 | 0.020 | 0.016 | 0.408 | 0.23 | 16x |
| fasta | 0.118 | 0.017 | 0.005 | 0.007 | 0.226 | 0.08 | 7x |
| race_smoke | 0.104 | 0.010 | 0.004 | 0.005 | 0.183 | 0.05 | 10x |

`wordfreq` builds natively since ADR 0568 (see below). `loops` needs string
interpolation into a value, and the tensor programs need comprehensions and
`std.tensor`; they are outside the native subset.

## Where the time goes against C

Read from the generated code (`ail run compiler mir FILE`, objdump):

1. **Every list access is a call.** `xs[i]`, `xs[i] = v` and `push` go
   through runtime functions with a bounds check each; C indexes a pointer.
   This dominates sieve, matmul, fannkuch and binarytrees. Fix: inline the
   load/store with the length check, then hoist the check out of loops whose
   range is the list's length.
2. **Checked arithmetic.** Each `+`, `-`, `*` tests overflow
   (expr.arith.1); C doesn't. Range analysis on induction variables removes
   most of these in loops; the rest cost one predicted branch.
3. **No constant folding or strength reduction.** `n % 2 == 0` divides;
   constants are re-materialised per use; `x / 2` divides (collatz).
4. **No inlining.** Small helpers (`escape`, `a(i, j)` in spectralnorm) pay
   a full call with spills around it.
5. **Allocation.** binarytrees and fasta allocate through the runtime's bump
   allocator, but every list starts at capacity 4 and grows by copying.

The float kernels (nbody, mandelbrot) are within 1.7x because their inner
loops are register arithmetic, which the allocator already does well.

## The compiler's own speed

`ail build` of a perfbench program takes 0.63–0.67 s on this machine, almost
all of it fixed: the self-hosted backend runs on the interpreter and
compiles the 800-line runtime (`compiler/rt/native/`) with every program.
`ail check` of the same program takes 5 ms and `clang -O2` 120 ms. Two
steps close the gap: cache the compiled runtime (it changes only with the
compiler), and build the compiler natively once the subset covers it
(structs, enums, maps, methods).

## Next steps, in order of expected gain

Done in ADR 0391: constant folding, the `% 2^k` mask, overflow checks ruled
out by a dominating bound, inline `push`, inlining of leaf functions. Left:

1. **Bounds checks in loops** (matmul, fannkuch): hoist the check over
   `0..xs.len()` and index without it; keep row pointers in registers.
2. **List allocation** (fannkuch, binarytrees): start lists at their
   literal or known size; free short-lived lists once a collector exists.
3. **Callee-saved registers** (binarytrees): save only the ones a function
   uses on the paths that need them.
4. **Division by a power of two** as a shift with a sign fix (collatz).
5. Cache the runtime's machine code (compile time).

## Maps: `wordfreq` (ADRs 0568–0570)

`Map[str, int]` counts 10 million words from a 12-word list (best of 3,
seconds; Linux x86-64, same VM): native 0.359, `gcc -O2` 0.055, CPython
1.876, so native is 5.2x faster than Python and 6.5x slower than C. Each
iteration hashes the key twice (`get`, then the assignment) and compares it
by pointer, then bytes. A cached hash in the string object, a combined
find-or-insert, and keeping `map_find` and `map_set` inline are the next
steps. At the `.args` size (1 million words) the run takes 0.07 s and
prints the interpreter's bytes on x86-64 and on ARM64 under qemu.

**Remembered entry (follow-up).** A table keeps the last key it found or added
and its entry, so the assignment after `get(k)` skips the hash and the probe
(a rebuild, removal or copy forgets it). 10 million words: 0.341 s -> 0.260 s
(gcc -O2 0.055 s, CPython 1.9 s). Still open: a cached hash in the string
object (needs a spare word; agree the layout with the strings and GC lanes),
and inlining `map_find` and `map_set` into the caller.
