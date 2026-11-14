# Perfbench drag-race programs

Ten programs, each written four times with the same algorithm and the same
output: `NAME.ail` (AIL), `NAME.c` (C11, libc only; `sqrt` from libm), `NAME.rs`
(safe Rust, std only, ported line for line from the C twin) and `NAME.py`
(CPython 3, no numpy). `NAME.out` is the expected output (they agree on Linux
x86-64). `NAME.args` holds the size argument, which each program reads from
`env.args()`, `argv`, `std::env::args()` or `sys.argv`; without it the program
uses its built-in default. The larger programs in `big/` have C and Rust twins.

Build and run by hand:

    ./ail run tools/perfbench/programs/NAME.ail $(cat tools/perfbench/programs/NAME.args)
    gcc -std=c11 -O2 -ffp-contract=off -o NAME NAME.c -lm && ./NAME $(cat NAME.args)
    rustc -C opt-level=3 --edition 2021 -o NAME NAME.rs && ./NAME $(cat NAME.args)
    python3 -I NAME.py $(cat NAME.args)

`-ffp-contract=off` keeps C from fusing multiply-adds, so floats match the
other two (IEEE double everywhere).

## Programs

| Program | What it does | Pure subset |
|---|---|---|
| collatz | total and longest Collatz chain below N | yes |
| nbody | five-body gravity, energy before and after | yes (flat `List[float]`) |
| spectralnorm | power iteration on the infinite matrix A(i,j) | yes |
| mandelbrot | ASCII Mandelbrot, N rows of 2N characters | no: builds `str` rows with `join` |
| fannkuch | fannkuch-redux checksum and max flips, N = 9 | yes |
| binarytrees | arena-allocated trees, check sums, depth N | yes (index lists) |
| fasta | LCG-weighted DNA, 60-column lines | no: `char` lists and `join` |
| wordfreq | word counts over a generated text | no: `Map[str, int]` |
| matmul | N x N integer matrix product | yes (`List[List[int]]`) |
| sieve | primes below N: count, sum, largest | yes (`List[bool]`) |

wordfreq's text is generated in memory by the same LCG in all three, not read
from a file: the brief asked for a generated text, and the three languages
would otherwise need a shared data file.

## Measurements (2026-10-20, Linux x86-64, min of 3 runs; wall time)

`ail` is `ail run` on the release bootstrap (includes check and startup).
Token counts are cl100k on each file.

| Program | size | ail s | py s | C s | ail/py | tokens ail | tokens py |
|---|---|---|---|---|---|---|---|
| collatz | 100000 | 0.529 | 1.140 | 0.027 | 0.46 | 245 | 196 |
| nbody | 50000 | 1.223 | 0.501 | 0.011 | 2.44 | 1432 | 1225 |
| spectralnorm | 250 | 0.915 | 0.701 | 0.008 | 1.31 | 463 | 336 |
| mandelbrot | 240 | 0.785 | 0.308 | 0.012 | 2.54 | 327 | 258 |
| fannkuch | 9 | 1.435 | 0.908 | 0.025 | 1.58 | 514 | 414 |
| binarytrees | 14 | 2.600 | 0.786 | 0.021 | 3.31 | 363 | 270 |
| fasta | 500000 | 0.229 | 0.169 | 0.009 | 1.36 | 328 | 246 |
| wordfreq | 1000000 | 0.333 | 0.247 | 0.009 | 1.35 | 261 | 185 |
| matmul | 200 | 1.345 | 0.606 | 0.012 | 2.22 | 326 | 222 |
| sieve | 3000000 | 1.392 | 0.765 | 0.021 | 1.82 | 264 | 208 |

AIL is slower than CPython on nine of ten and uses more tokens on all ten.
These numbers are the baseline for the native backend (lane 120); lane 132
runs the race in the harness.

## Noise thresholds (day 3, 2026-10-21)

Measured on the batch head 72849752 (Linux x86-64, 4 cores, shared VM). Source:
`/mnt/project-files/reports/perfbench-noise-day3.md`, Part 1 (callgrind) and Part 2
(wall clock).

Instruction counts (callgrind, `instr.sh`, 5 runs each):

- `ail` (`ail run` and native): deterministic within 97 instructions across runs
  (nbody and wordfreq vary by 97 of 3.7 and 1.9 billion). Call a before/after
  instruction change of **0.01%** or more real.
- CPython `-I`: near-deterministic, 0.0007–0.06% on nine programs. `wordfreq`
  varies 0.29% because its word dict is hash-seed sensitive. Call a change under
  **0.5%** noise. Set `PYTHONHASHSEED=0` for a fixed Python baseline.
- Instruction counts miss cache, branch and allocator effects. Confirm a small
  change in wall time with a timed run.

Wall time (medians of 10 runs, `race.py` helpers, not `run.sh`):

- Spread is large on this VM: 14–167% for C and native runs of 5–80 ms, and
  37–99% for `ail run`. Short runs cannot resolve small changes.
- Adopted by the integrator (2026-10-21): treat a change under **10%** of the
  median as noise, for runs of 0.3 s or more. Report the spread with any
  wall-time claim, and back it with an instruction-count change where one exists.
## NumPy twins
`numpy/` holds three `std.tensor` programs with NumPy twins (`f32_chain`,
`f32_matmul`, `softmax`). `numpy/run.sh [RUNS]` checks that the outputs match
and prints wall time, tokens and whether `ail build` accepts the program. The
results are in `docs/design/automatic-parallelism.md` section 6.4.
