# 0890. Optimizer roadmap: speed passes to match C at -O2 and -O3

- **Status:** Accepted
- **Date:** 2026-10-24
- **Decided by:** AI (delegated, ADR 0076; criterion: speed is the goal here, per ADR 0874)
- **Topic:** 23 — Implementation (native optimizer)
- **Spec rules:** none (every pass keeps observable behaviour, traps included)

## Context

The designer wants AIL to compete hard with C at `-O2` and `-O3`. ADR 0874 limits the
optimizer to passes that increase speed; binary size and compile time (within reason) don't
count. Pure-call CSE and loop hoisting landed in `compiler/mir/purecse.ail`; memoization is
a separate PR. Tensors (packed f32, SIMD, multi-core) belong to the tensor lane, and RR-3
owns pure CSE and memoization, so this ADR doesn't touch them.

Known gaps, from [native-performance.md](../design/native-performance.md) (the last full
table against `gcc -O2`; a fresh table comes from the baseline job below):

| Program class | Gap vs gcc -O2 | Cause |
|---|---|---|
| binarytrees | 5x | allocation per node, values live across calls spill |
| fasta | 2.7x | list and `str` allocation, `rt.alloc` call per object |
| fannkuch | 2.5x | a list per permutation, loop scalars spilled (5 callee-saved regs) |
| matmul | 2.3x | `b[k][j]` row check, `n` spilled, no unroll or vector code |
| collatz, mandelbrot, spectralnorm | 1.2–1.4x | loop structure, copies, divide by 2 |
| wordfreq | 6.5x | hashing, probe not inlined |
| nbody, sieve | 1.1–1.4x, 1.0x | near parity |

Mechanisms AIL has that C compilers lack, and which make passes simpler or stronger:

- **Value semantics:** a `List` value has no aliases, so every load and store is
  alias-free without `restrict`. Vectorization and load/store motion need no alias proof.
- **Effect-proven purity** (ADR 0874): calls can be CSE'd, hoisted, reordered.
- **Checked arithmetic and bounds** are explicit trap sites, so range facts from a guard
  hold on the whole dominated region.

## Options considered

### Option A — ranked passes by measured gap, one PR each (chosen)
- **Pros:** each PR has a perfbench number; small diffs fit the merge queue.
- **Cons:** slower to reach a "complete" optimizer than a big-bang rewrite.

### Option B — SSA rewrite with a general sparse-conditional pipeline first
- **Pros:** clean base for later passes.
- **Cons:** a rewrite that gains nothing until later; MIR is already SSA with block params.

### Option C — only vectorization and unrolling (the `-O3` flags)
- **Pros:** visible wins on matmul and float loops.
- **Cons:** leaves binarytrees, fasta and wordfreq (the biggest gaps) untouched.

## Decision

Build the following passes, in this order, each as its own file `compiler/mir/opt_*.ail`
(or a target file for allocator and codegen work) with at most one dispatch line in the
shared pass list. Each lands with a perfbench before/after and a golden test.

1. **Allocation.** (a) Inline bump-pointer allocation with the slow path out of line
   (today `rt.alloc` is a call plus checks per object). (b) Escape analysis for lists and
   records: a value that never leaves its function is built on the stack or in a region, and
   a constant-length small list becomes scalars (scalar replacement). Value semantics makes
   the escape proof local. Targets: binarytrees, fasta, fannkuch.
2. **Register allocation.** Live-range splitting at calls (a value that lives across a
   call is split so only the call-crossing part is in a callee-saved register or slot, the
   rest stays in a caller-saved register), spill-slot coalescing, and better loop-carried
   scalar handling. Targets: fannkuch, binarytrees, matmul's spilled `n`.
3. **Inlining.** Raise it from leaf-only (`INLINE_LIMIT` 40) to a call-graph cost model:
   inline non-leaf small functions and single-call-site functions, bottom up, with a
   size budget per function. Targets: callback-heavy and method-heavy code.
4. **Scalar loop optimizations.** General LICM (beyond pure calls), induction-variable
   strength reduction, division and modulo by constants as multiply-shift, signed divide by a
   power of two as shift plus fix, unrolling of small counted loops, and loop peeling for
   the first check. Targets: collatz, mandelbrot, spectralnorm.
5. **Range analysis.** Extend `nonneg.ail` and `bce.ail` into one interval analysis over
   MIR so overflow checks inside guarded loops disappear and row checks (`b[k][j]`) hoist.
6. **Auto-vectorization** of counted loops over `List[float]`, `List[int]`, `List[u8]`
   with no loop-carried dependence, using the alias-free guarantee, for SSE/AVX2 on
   x86-64-v3 and NEON on ARM64 in lockstep (ADR 0008). Reductions use the existing
   exactness rules (no float reassociation; integer reductions only, unless a `par`/tensor
   op already permits it). Coordinated with the tensor lane's SIMD kernels so the code
   generator support is written once.
7. **Maps.** Cached hash in the string object, combined find-or-insert, and inlined probe.
8. **Tail calls** to self and mutual tail calls as jumps.

Every pass keeps traps and their order. A pass that can't prove that skips the code.

## Pass 1a: inline allocation fast path

`inline_module` (`compiler/mir/inline.ail`) inlines `rt.alloc_z`, `rt.alloc_raw`, `rt.alloc`
and `rt.obj_new` although they call `rt.alloc_slow`: their fast path (free list or bump
pointer, no collection or time check due) is straight-line code, and a constant size folds the
size class away. The slow path stays a call. `rt.list_new` and the other list makers stay
calls because the list passes find them by name. The change is in MIR, so x86-64 and ARM64
get it together (ADR 0008). Measured on binarytrees, fasta and fannkuch on ail-linux (A/B,
same checkout with and without the commit). Result (2026-10-25, Mac): no change, instruction
counts within 0.1M on all six programs. The hot functions of binarytrees, fannkuch and fasta
have no allocation calls: binarytrees' time is the recursive `make`/`check` and two
`list_push` per node. The inlining was removed again (2026-10-25, #880): it gained nothing and
enlarged every function that builds a struct or list, and the gate got slower. Measured on
ail-linux, stage 1 and stage 2 of the self-host, alone: with it 127.9 s (71.8 s and 56.1 s) and
a 9,674,760-byte stage-1 binary; without it 58.7 s (35.7 s and 23.0 s) and 7,924,864 bytes.
The recursion pass (below) costs no compile time: `AIL_OPT_OFF=unroll` gives 58.8 s and
7,898,288 bytes. Lesson: a pass that inlines a large body at every call site needs a
compile-time and size check as well as a speed check. Pass 1 as a
speed lever is replaced by the codegen items below; escape analysis (1b) waits for a profile
that shows allocation.

## Pass 2 first slice: ARM64 call and global overheads

Read from the ARM64 disassembly of binarytrees: every recursive call pays a five-instruction
stack check (`adrp`, `add #0`, `ldr`, `cmp sp`, `b.lo`) and every call saves and restores
the callee-saved registers with one `str`/`ldr` each. (a) `adr_sym` of `globals` skips its
`add`: the globals start on a page in every object writer, so the add is always `#0`.
(b) The callee-saved registers are saved and restored in pairs with `stp`/`ldp`.
x86-64 already checks the stack with one `cmp rsp, [mem]` and has no pair form to use, so
nothing is owed there (ADR 0008).

## Pass 3 first slice: self-recursive calls inlined once

`compiler/mir/opt_unroll.ail`: a user function of at most 60 instructions with one to three
direct self calls gets a copy of its own body at each of those calls (the copy's own
recursive calls stay calls). A recursion of depth d then makes about d / 2 calls, and each
pair shares the frame, the callee-saved saves and the stack check. gcc -O3 does this
(binarytrees is 2.3x of -O3 against 1.6x of -O2). Trap sites keep their frames, so traces
are unchanged. `AIL_OPT_OFF=unroll` turns it off, so one binary gives the A/B.

## Pass 4 (amendment, 2026-10-26): loop-invariant list headers

Measured with `perf stat` on ail-linux (main 66224b97, gcc 13 `-O2`): AIL runs 21.8 G
instructions on lzr against gcc's 8.75 G, 26.5 G against 10.2 G on vm and 33.3 G against
19.4 G on matmul 1400, with the same branch misses. The gap is instruction count. Every
`xs[i]` reloads the list's length (for the check) and its element pointer (for the
load) from the header, two loads before the element, and a `while` loop gets no help
from the counted-loop versioning of ADR 0577.

`compiler/mir/hoist.ail` (`AIL_OPT_OFF=hoist` turns it off): in a loop whose only uses
of a list are `get`, `set` and `len`, the list a parameter or a call's result and defined
outside the loop (never a field or an element, which another holder can push to), the
length and element pointer are read once in the loop's preheader. A `get` becomes the new
op `bounds index, len` followed by `ldx`; a `set` becomes `bounds` and `stx`; `len`
becomes a copy. `bounds` compiles to a compare and a branch to the same trap stub, which
is given the length instead of reading it from the list, so the message, index, length and
site are unchanged (native.bce.11). At most four lists per loop (two registers each).
Loops of more than twelve blocks are skipped: on the vm drag-race program (an
interpreter dispatch loop, each list touched in a few arms) the two held registers per list
cost spills on x86-64, 1.4% to 3.0% slower than main over two rounds of five runs, so
the pass leaves dispatch loops alone.
The element pointer is an interior pointer, which keeps the block alive for the collector.

Rejected: interchanging loops for matmul (it changes which overflow traps first); hoisting
for every list value (a list built by `push` in the loop, or reached through a field,
can change under the loop).

## Pass 5 (amendment, 2026-10-27): `match` as a decision tree

The vm drag-race program dispatches with a 10-arm `match op`; the arms were tried in
order, so the last arm cost ten compares and the average about five. gcc uses a jump table.
`compiler/mir/lower_matchtree.ail` (`AIL_OPT_OFF=matchtree` turns it off) lowers a match
of five or more int-literal or variant arms, with no guard or range, to a binary search over
the sorted values, so a dispatch costs about log2(arms) compares and needs no backend
change, so x86-64 and ARM64 get it together (native.match.1). Rejected for now: a jump
table, which needs a new terminator and an indirect branch in both backends and an
`unreachable` bound check; reconsider if the tree's measured gain is small.

## Dropped leftovers (2026-11-09)

- **SROA constructor filter and size cap** (`claude/project-thread-bi6mi0-sroa2`): measured on
  2026-10-27 at -1.5% size and +6% raytrace time, and it conflicts with the joins, late inline and
  sret work in ADR 0923. Dropped; `AIL_SROA_STATS` is not ported.
- **Constant-displacement folding** (`lane/235-disp-fold`, `mir/disp.ail`): `fold_prim_addresses`
  (`mir/addr.ail`) folds the same `load64`/`store64` addresses and also scaled indexes. Superseded.

## Rationale

The listed gaps point at allocation and register pressure first, not at `-O3` loop
transformations: the program classes furthest from C are bound by allocation and spills,
so vectorization alone would leave AIL slower on four of nine programs. The alias-free and
purity facts give AIL a path past C, not just to parity, on loops where C must emit
runtime alias checks or can't prove a call pure.

## Measurement protocol

- On ail-linux only. Baseline (item 0): `tools/perfbench` programs at the large sizes
  in `tools/perfbench/large.py` against `gcc -O2`, `gcc -O3`, `clang -O2`, `clang -O3`
  (all `-ffp-contract=off`), min of 7, plus instructions retired (callgrind) at the
  `.args` sizes. The table goes into `docs/design/native-performance.md`.
- A pass lands only with a before/after on the programs it targets and no regression
  over 3% elsewhere.
- Tensor programs and parallel runs stay with their lanes.

## Rejected alternatives

- **PGO or runtime profiling:** needs a profile step in every build; revisit after the
  static passes.
- **A second IR (a full SSA/LLVM-style pipeline):** ADR 0010 forbids LLVM, and the MIR is
  already SSA.
- **Float reassociation for vectorized reductions:** changes output bits and breaks the
  interpreter-parity tests.
