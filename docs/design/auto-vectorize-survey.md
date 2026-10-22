# Auto-vectorization survey for the ten native-vs-C programs

2026-10-21 (ADR 0845). Measured on the batch head plus PR #535, x86-64 Linux, gcc 13.3 and clang from this container. No code written. The policy is already [ADR 0246](../decisions/0246-simd-vectorisation.md) (vectorise independent loops, per-lane overflow check with a scalar redo, no implicit FMA); this note says what that buys on these programs and what the analysis needs.

## Bottom line

1. **The ten C twins are not fast because of vectorization.** `gcc -O2 -ffp-contract=off` (what perfbench uses) vectorizes no loop in any of them. It does two small things: SLP (two doubles in one 16-byte register) in mandelbrot, nbody, sieve and fasta, and it turns the sieve's fill loop into `memset`. Even `gcc -O3` and `clang -O3 -march=x86-64-v3` change the wall time by under 10% on nine of the ten (matmul is the exception, -20% to -36%; gcc -O3 binarytrees is faster for a reason unrelated to vectors, probably inlining). The native-vs-C gap is scalar code, allocation and runtime plumbing:

   | program | AIL instr | gcc -O2 instr | ratio |
   |---|---|---|---|
   | mandelbrot | 34.1M | 32.3M | 1.05 |
   | sieve | 60.8M | 45.3M | 1.34 |
   | spectralnorm | 52.9M | 36.6M | 1.44 |
   | collatz | 119.4M | 80.0M | 1.49 |
   | fannkuch | 148.3M | 92.1M | 1.61 |
   | nbody | 49.3M | 28.3M | 1.74 |
   | binarytrees | 228.1M | 116.2M | 1.96 |
   | matmul | 116.0M | 57.7M | 2.01 |
   | fasta | 43.9M | 18.2M | 2.41 |
   | wordfreq | 192.5M | 26.3M | 7.33 |

   (callgrind; fannkuch 9, matmul 200, the rest at their `.args`.)
2. **Vectorization would beat C, not catch up to it**, because C at -O2 doesn't do it. It pays on three shapes here: the sieve's fill loop (an idiom, not SIMD), mandelbrot (independent pixels), and spectralnorm (independent rows). Everything else is either strided, serial, or too short.
3. **Recommended order:** (a) loop idioms first (fill to memset, copy to memcpy): small, general, both ISAs, and it is what gcc -O2 does today; (b) outer-loop vectorization of independent float iterations with a bit-identical per-lane order (mandelbrot, spectralnorm); (c) integer lanes only behind a range guard. Do not start (b) before (a) and before the scalar gaps above shrink, since they are bigger and cheaper.

## Hot loops, program by program

"Share" is the share of the program's instructions, from callgrind where I profiled it.

| program | loop | vectorizable? | why / what it needs |
|---|---|---|---|
| sieve | `for _ in 0..n: is_prime.push(true)` | **yes, as a fill** | 15M of 60.8M (25%): 5 instructions per element (`movb`, two induction adds, `cmp`, `jl`). A constant push loop with room guaranteed (native.bce.6) is a memset. gcc does exactly this. Needs: counted loop, loop-invariant value, one list. |
| sieve | `while j < n: is_prime[j] = false; j += i` | no (practically) | 26.4M (43%), 4 instructions per store, already what C emits. Strided byte stores have no AVX2 scatter. A pattern-store trick for strides below 16 would be tailored to the sieve; I'd leave it. |
| sieve | `for k in 0..n: if is_prime[k]: count += 1; total += k; largest = k` | partly | 12M+ (20%). A conditional count/sum/last-index loop. Count and sum vectorize (byte compare, popcount, masked add with a range guard); `largest` is a last-true reduction. No compiler does this at -O2/-O3 here, so it is an upside only. |
| mandelbrot | `escape` per pixel, called from the `x` loop | **yes, outer loop** | 32.6M of 34.1M (96%) in the escape loop. Pixels are independent; a while loop with `break` and a different trip count per pixel needs lane masks (4 pixels per AVX2 register, run until all lanes are done). Every operation is `fmul`/`fadd`/`fsub`/compare, no reordering inside a lane, so results are bit-identical. The loop pushes one char per pixel, so the vector part is the escape counts into a small buffer. Needs: inline `escape`, independence of the `x` loop (conc.par.2-4 verdict from `ail effects --parallel`), masked control flow. gcc/clang do not do this. Likely 2-3x on this program. |
| spectralnorm | `mul_av`/`mul_atv`: `for i: sum = 0; for j: sum += eval_a(i,j) * v[j]` | **yes, over i** | The inner sum is a float reduction in a fixed order (ADR 0236, no reassociation), so it cannot be vectorized over `j`. It can over `i`: four rows at once, each lane doing its own sequential sum, bit-identical. The body has an integer division by 2 (a shift with a sign fix), `to_float` and a float divide. AVX2 has no `int64`-to-`double` convert (that's AVX-512DQ), so lanes need the values below 2^31 (guard) or a two-step convert. Gain bounded by the divide throughput. |
| nbody | `advance`: 10 pair interactions, inner loop `j in i+1..5` | no | Trip counts of 1 to 4, dependences through the velocities of body `i` and `j`. The SLP win gcc and clang get (3 doubles as 2+1) would be the only kind available here; a straight-line pair-packing pass is a different pass from loop vectorization. |
| matmul | `for k: s += a[i][k] * b[k][j]` (int) | not as written | The `b[k][j]` access is a column walk (stride n) and the reduction is `int` with overflow checks. gcc -O3 gets -20% by other means; clang -O3 gets -36%. Loop interchange (ikj) would make the inner loop an axpy over contiguous `j` and then vectorizable, but AVX2 has no 64-bit integer multiply (`vpmullq` is AVX-512), so int lanes need a 32-bit-range guard and `vpmuldq`, plus per-lane overflow detection on the add. Not first. |
| fannkuch | perm copy `for i in 0..n: perm[i] = perm1[i]` (n = 9) | copy idiom only | 9 elements: a `memcpy` call costs about what the loop does. The flip loop is data-dependent. |
| binarytrees | none | no | allocation and recursion. |
| fasta | the random generator, the table lookups | no | serial linear congruential state; output formatting. |
| wordfreq | none | no | hashing and map code. |
| collatz | `steps` | no (as a loop); possible across starts | the 64-bit `n % 2`, `n / 2`, `3 * n + 1` chain with an overflow check on `3 * n`; starts are independent, so 4 lanes with masks could run, but clang -O2 is already 33% faster than gcc here by scalar means (probably a branchless step), which is the cheaper thing to match first. |

## What the analysis needs, and what already exists

- **Independence.** conc.par.2-4 and `ail effects --parallel` (ADR 0236) already give a per-loop verdict and the first blocking write. Native vectorization should consume that verdict, not recompute it. Values are independent by mutable value semantics, so two `mut` lists never alias; the aliasing analysis that dominates a C vectorizer is nearly free here.
- **Trip counts.** Counted loops are found in `mir/loops.ail` (`counted`) and BCE already versions them with range guards (native.bce.1-8). A vector loop is the same shape: guard, vector body for `n / 4` iterations, scalar remainder.
- **Bounds checks.** Inside a versioned loop they are gone, so a vector load/store needs no per-lane check.
- **Overflow.** Every `int` `+ - *` traps (expr.arith.1). The rule from ADR 0246 stands: compute lanes, test, redo the chunk in scalar on a hit so the trap carries the right iteration. For `int` sums and products, a cheap exact guard exists: if every loaded element is below 2^31 in magnitude and the chunk is shorter than 2^31 elements, the 64-bit partial sums cannot overflow, so no per-lane test is needed. Floats have no overflow trap.
- **Float reductions.** Not reassociated, ever (ADRs 0236, 0524). A dot product (`sum += a[i] * b[i]`) stays scalar unless the tree order is fixed by the language (`sum()`, ADR 0239). Vectorizing across independent outer iterations, one reduction per lane, is exact.
- **No FMA.** ADR 0246: `a * b + c` stays two roundings; vector `mul` then `add`.
- **Hardware.** Baseline AVX2+FMA (ADR 0005); NEON on ARM64 (ADR 0006). Lockstep (ADR 0008) means every vector pass lands with both a 256-bit x86-64 path and a NEON path (two 128-bit lanes per op). #490 added the 256-bit VEX encodings to `x64/asm.ail` (`vmovupd`, `vaddpd`, `vsubpd`, `vmulpd`, `vdivpd`, `vbroadcastsd`, `vcvtpd2ps`, `vzeroupper`) and three hand-written kernels in `x64/kernels.ail`; those encoders are reusable as they are. Not there yet: integer vector ops (`vpaddq`, `vpcmpgtq`, `vpmuldq`, `vpmovmskb`, `vpand`), masked blends (`vblendvpd`), compares (`vcmppd`), `vsqrtpd`, gathers, and anything on the NEON side beyond the planned tensor kernels.

## Staged plan

1. **Idioms (small, both ISAs, no new SIMD).** A MIR pass recognises a push loop with a loop-invariant value on a list with room guaranteed (the shape native.bce.6 already produces) and emits a fill: `rep stosb`/`stosq` (fast on ERMS hardware) on x86-64 and a runtime fill on ARM64, and the same for a copy loop. Expected: sieve about -14M instructions of 60.8M (about -23%); nothing else among the ten moves much. This matches what gcc -O2 does, so it is catching up, not a trick.
2. **Outer-loop float vectorization.** For a counted loop the independence verdict accepts, whose body is float arithmetic, float compares and nested while loops, run four iterations per register with lane masks. Targets: mandelbrot, spectralnorm. This needs the missing encodings above (compare, blend, mask-move) and a NEON equivalent of every operation used. It is the only item that makes native faster than C on these programs by a wide margin.
3. **Integer lanes behind a range guard.** Sums, counts and element-wise integer loops with the 2^31 guard; then the sieve count loop and the ikj matmul axpy. Medium value, high encoder cost (no 64-bit multiply on AVX2).

## Fairness and risks

- Every item above is a general pass over independent loops; none keys on a program name, size or argument. The sieve strided-store pattern trick is the one candidate that would only help one benchmark shape, and I'd not build it.
- The size of the prize on these ten is smaller than the scalar gaps: wordfreq 7.3x, fasta 2.4x, matmul 2.0x, binarytrees 2.0x. If the goal is "beat C on the ten", scalar codegen, allocation and the map runtime come before stages 2 and 3. If the goal is "AIL uses the hardware automatically on user loops" (Kaleb's framing), stage 2 is the feature and the ten programs are weak evidence for it: mandelbrot is the one clear showcase.
- Compile time: a vectorizer adds a pass per counted loop; guard it behind the independence verdict so ordinary code pays nothing.

## Questions this raised

Answered in [ADR 0845](../decisions/0845-vectorization-order-and-idioms.md): the order of the stages, a runtime fill and copy on ARM64 until NEON lands, and loops with `break` in the independence verdict.
