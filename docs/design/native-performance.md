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

## After bounds-check elimination and list sizing (ADRs 0577–0579)

Lane 176. Counted loops are versioned with guards so their list accesses
lose their checks, row loads are hoisted into the guard, two-pointer loops
(`while i < j`) are covered, accesses that an earlier access or a `push`
vouches for lose theirs, and a list filled by a counted loop is sized first.

| program | size | native | clang -O2 | gcc -O2 | python | native/clang | native/gcc | native/python |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| collatz | 1000000 | 0.347 | 0.158 | 0.244 | 10.612 | 2.19 | 1.42 | 0.03 |
| nbody | 1000000 | 0.091 | 0.054 | 0.058 | 6.351 | 1.68 | 1.57 | 0.01 |
| spectralnorm | 1000 | 0.076 | 0.052 | 0.054 | 8.080 | 1.46 | 1.41 | 0.01 |
| mandelbrot | 1000 | 0.108 | 0.083 | 0.079 | 4.263 | 1.30 | 1.36 | 0.03 |
| fannkuch | 10 | 0.609 | 0.234 | 0.220 | 9.157 | 2.60 | 2.76 | 0.07 |
| sieve | 30000000 | 0.245 | 0.237 | 0.239 | 5.890 | 1.03 | 1.02 | 0.04 |
| matmul | 400 | 0.138 | 0.049 | 0.069 | 4.673 | 2.83 | 2.01 | 0.03 |
| binarytrees | 17 | 0.789 | 0.179 | 0.135 | 5.453 | 4.41 | 5.84 | 0.14 |
| fasta | 5000000 | 0.175 | 0.060 | 0.069 | 1.176 | 2.91 | 2.53 | 0.15 |

Instructions retired at the `.args` sizes (callgrind), before → after:
fannkuch 442M → 272M, matmul 223M → 142M, sieve 144M → 133M, binarytrees
381M → 375M, fasta 75M → 74M, collatz 188M (unchanged). Wall times on this
shared VM vary by 10–20% between runs: fannkuch ran 0.84–0.94 s before and
0.51–0.61 s after, matmul 0.20 s before and 0.13–0.14 s after.

What is left against C: (the row check of `b[k][j]` in matmul went away
with the row-length floor, ADR 0611), the elements' address is
kept in a register only inside versioned loops (`data`, `ldx`, `stx`),
`n` still spills in matmul's inner loop, and binarytrees and fasta are
allocation and call bound, not check bound. `ail build` takes about 0.1 s
longer per program.

## Range guards and smaller wins (ADR 0611)

Wall time, best of 7, against `gcc -O2 -ffp-contract=off` at the large sizes;
the "before" column is the batch head with #181. The VM varies 10-20%.

| program | before | now | gcc -O2 | before/gcc | now/gcc |
|---|---:|---:|---:|---:|---:|
| collatz 1000000 | 0.315 | 0.297 | 0.218 | 1.45 | 1.36 |
| mandelbrot 1000 | 0.109 | 0.097 | 0.077 | 1.42 | 1.26 |
| spectralnorm 1000 | 0.074 | 0.062 | 0.051 | 1.45 | 1.21 |
| matmul 400 | 0.125 | 0.117 | 0.050 | 2.51 | 2.35 |
| fannkuch 10 | 0.804 | 0.551 | 0.220 | 3.65 | 2.50 |
| sieve 30000000 | 0.237 | 0.245 | 0.199 | 1.19 | 1.23 |
| binarytrees 17 | 0.665 | 0.674 | 0.135 | 4.94 | 5.01 |
| fasta 5000000 | 0.166 | 0.159 | 0.059 | 2.80 | 2.68 |

What is left: matmul's `b[k][j]` keeps its row check (the rows' lengths
aren't known), collatz and the float kernels pay for loop structure and
copies, and binarytrees and fasta are bound by allocation and memory.

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

`loops`, `wordfreq` and the tensor programs are outside the native subset
(maps, interpolation into a value, comprehensions, `std.tensor`).

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

1. **Bounds checks in loops** (matmul, fannkuch): done in ADR 0577. Left:
   keep the data pointer and length in registers across the loop.
2. **List allocation** (fannkuch, binarytrees): sized before counted loops
   in ADR 0579. Left: free short-lived lists once a collector exists.
3. **Callee-saved registers** (binarytrees): save only the ones a function
   uses on the paths that need them.
4. **Division by a power of two** as a shift with a sign fix (collatz).
5. Cache the runtime's machine code (compile time).

## Pushes with room, sized lists, loop rotation (ADR 0611)

fannkuch at N = 9, instructions: 274M → 218M (gcc -O2: 103M); wall 0.58 s →
0.49 s against 0.22 s. What is left is the allocation of a list per
permutation (about half of the gap: `rt.alloc` runs three times each with
the checks `a == 0 or b > c` materialised as booleans, `list_new`,
`list_reserve`), `n` spilled to the stack by the register allocator, and
the 8 to 10 instructions per swap against gcc's 6.

## Row-length floors (ADR 0611)

matmul at 400: 1,106M → 979M instructions (callgrind), 0.128 s → 0.098 s on
this VM. `rt.list_rows_min(b)` runs once before the outermost loop nest that
leaves `b` alone; the inner loop's guard tests `j` against it.
