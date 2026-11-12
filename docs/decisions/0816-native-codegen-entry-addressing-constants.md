# 0816. Native code generation: parameters stay put, scaled addressing, held constants

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 23 — Compiler architecture
- **Builds on:** [ADR 0611](0611-overflow-checks-by-range-guards.md), [ADR 0625](0625-address-folding.md) (address folding)

## Context

After the bounds-check work, the integrator asked for the code generation
items that are not register allocation, in lockstep on both ISAs (ADR 0008):
the argument swap at function entry, scaled addressing, and loop-invariant
constants. Profiles of wordfreq, binarytrees and fannkuch (callgrind, per
instruction) showed 30% of executed instructions were register-to-register
moves.

## Decision

1. **Parameters stay in their arrival registers.** The allocator picked the
   first free register of its pool for a parameter, which put the first two
   arguments in `rsi, rdi` (arriving in `rdi, rsi`) and cost three moves at
   entry. A parameter no hint places now takes the register it arrived in
   when that register is allocatable and free (`Target.int_args`,
   `float_args`; both ISAs). Hints still win, since a move inside a loop
   costs more than one at entry.
2. **Scaled addressing.** `fold_prim_addresses` (MIR, `addr.ail`, run on
   every function including the runtime) folds `base + k` and
   `base + i * 8` (bytes: `base + i`) into the `load64`, `load8`, `loadf` and
   `store` primitives when the `add` and `mul` have no other use. The
   primitive then takes `base` (and `i`), with `imm` the offset, or
   `offset * 16 + log2(scale)` when indexed. Only forms both ISAs have: an
   offset 0 to 32760 (a multiple of 8 for words), scale 8 for words and
   floats, 1 for bytes; x86-64 uses `[base + i*8 + k]`, ARM64
   `ldr x, [base, i, lsl 3]` (with one `add` when there is also an offset).
   Also: an unchecked `n * k` on x86-64 writes over `n` (`imul d, n, k`),
   and on ARM64 an unchecked `n * 2^k` is one `lsl` instead of a constant
   load and a `mul`.

Instructions executed, before → after items 1 and 2 together: wordfreq
254.6M → 214.4M, binarytrees 412.2M → 368.3M, fannkuch 284.6M → 261.3M,
fasta 81.3M → 72.5M, sieve 128.8M → 124.4M, matmul 117.0M → 116.6M; collatz
and spectralnorm unchanged. Item 1 alone: wordfreq 240.9M, binarytrees
399.5M, fannkuch 274.8M. L1 data misses on matmul (9,813,919 → 9,813,745)
and fannkuch (1,247,171 → 1,247,781) are unchanged.

3. **Lists sized at creation have their element size and a byte limit.**
   `rt.list_new_for` took no element size and always made an 8-byte-element
   buffer of the collector's scanned kind, and both it and `rt.list_reserve`
   stopped at 2^20 elements. A `List[bool]` pushed 3,000,000 times (sieve)
   got an 8 MB buffer the collector scanned word by word
   (`gc_scan_range` calling `gc_mark` and `gc_find` for each, 34M of 124M
   instructions), and the range guard of the versioned loop (room for every
   push) failed, so the loop ran with a full-list test and a regrow per
   push. Now `list_new_for(lo, hi, per, size)` takes the size of the element
   (1 for `List[bool]`, else 8), allocates through `list_alloc` with that
   size (so the buffer has the kind `list_grow` gives it: no-scan for
   bytes, scanned for words), and both functions stop at 8 MiB of elements
   (2^20 words, 2^23 bytes). sieve 3,000,000: 90.3M to 79.4M instructions
   on the batch head (124.4M to 79.4M before chain E, which already
   removed part of the collector's scan); fannkuch 9 is unchanged.

4. **Push loops keep the length in a register.** In a versioned loop whose
   pushes have room guaranteed (native.bce.6), each push still loaded the
   list's length and element pointer and stored the length again: five
   instructions where a copy loop in C has one store. `keep_lengths`
   (`pushes.ail`, after `bce`) rewrites a loop that calls nothing but those
   pushes, has one latch and leaves only through its header, when no other
   instruction or edge in it mentions the list and every push runs once per
   iteration (its block dominates the latch). The element pointer (`data`)
   and the length are read in the preheader; the length is a block
   parameter of the header, each push becomes `stx data, length, v` and an
   add, and a new block on the loop's exit edge stores the length back. The
   loop allocates nothing, so the collector cannot see the stale length. A
   loop with a conditional push, a read of the list, or a call keeps the
   old code. sieve 3,000,000: 79.4M to 70.4M; fannkuch 9: 251.5M to
   244.6M; matmul 116.60M to 116.37M.

5. **Unchecked `add` and `sub` into any register are one `lea` on
   x86-64.** `Gen.arith` copied `x` into the result's register and added
   `y`, and when the result register was `y`'s it went through a scratch
   register and a third move (the `mov r11, base; add r11, idx; mov idx,
   r11` before every `base + i * 24` load in `rt.map_find_h`). An add with
   no overflow check (site -1: range guards, address arithmetic, loop
   counters) with `x` in a register is now `lea d, [x + y]` or
   `lea d, [x + k]`, and an unchecked `sub` of a constant `lea d, [x - k]`;
   `lea` reads both operands before it writes, so the result register can be
   either. Checked adds keep the old form because the overflow stub recovers
   operands from the flags. ARM64's three-operand `add` never needed it.
   Instructions (x86-64): wordfreq 214.4M to 209.1M, binarytrees 335.1M to
   323.8M, fannkuch 244.6M to 235.3M, fasta 70.2M to 66.9M, sieve and matmul
   unchanged.

6. **Constant indexes are checked against the pushes before them.**
   `dominated` (bce.ail) now also drops the check on `xs[c]` for a constant
   `c`, or for a block parameter that every edge passes a constant to
   (`base := 3`, then `base = 0` under `if`), when more than `c` pushes to
   `xs` are on every path to the access: the list has at least that many
   elements and, where no operation shortens a list, never fewer. fasta's
   `counts[base] += 1` and `letters[base]` lose their checks, as do the
   constant-index reads of a list literal after a loop. fasta 66.9M to
   65.9M instructions.

7. **The allocator puts values where their partners are.** About a fifth
   of the executed instructions in wordfreq and binarytrees were
   register-to-register moves. Three causes are fixed in
   `codegen/alloc.ail` and one in the x86-64 `add`: (a) the result of
   `add`, `sub`, `band`, `bor` and `bxor` is hinted to its operand's register
   (and, for the commutative ones, to the second operand's), so x86-64's
   in-place form applies; (b) a value whose hinted partner (that operand
   result, a block parameter) is placed later and lives across a call goes
   straight to a callee-saved register, so the partner takes the same one
   and no move is left (`mov %rax,%rsi; mov %rsi,%r14`); (c) a value
   computed within a few instructions of a call it dies at, and passed to
   it, prefers the argument register it is passed in; (d) `1 + n` (a
   constant first operand) is `add d, 1` in `n`'s register, with the
   overflow stub recovering `n` from the sum (`undo` 3). Instructions
   (x86-64): wordfreq 209.1M to 204.8M, binarytrees 323.8M to 318.8M,
   fasta 65.9M to 64.9M, nbody unchanged.

8. **Loads are forwarded, and constants are stored as immediates.** The
   dominator CSE of `fold.ail` leaves loads alone (a `push` changes `len`),
   so `xs[i]` read twice, a field read after a write to it, or a `len` read
   again each cost a load. `mir/forward.ail` (`forward_loads`, run after
   `bce` and the push-loop length pass, on user code and the runtime) keeps,
   along a block and the blocks below it that have one predecessor each,
   the values of the list headers (`len`, `cap`, `data`), the elements
   (`get`, `ldx`), the struct slots (`load`) and raw memory (`prim load64`,
   `load8`, `loadf`) it has read or written, and reuses them for an identical
   load. An element store forgets elements and raw memory; a slot store
   forgets slots, raw memory and headers (all of them when the object is a
   list); a raw store, a call, and a primitive with effects forget
   everything. A store remembers its value for the load that reads it back.
   A block with two predecessors, such as a loop header, starts empty, so
   nothing crosses a back edge. On x86-64 a store of a constant that fits
   32 bits (`stx`, `set`, `store`, `store64`, `store8`) is `mov [m], imm`
   (`mov8_mi` is new) in place of a load into `rdx` and a store. ARM64 has no
   store immediate; its stores are unchanged. Instructions (x86-64): sieve
   70.4M to 60.8M, fannkuch 234.0M to 231.1M, binarytrees 318.8M to 315.6M,
   wordfreq 204.8M to 203.8M, the other six unchanged.

9. **A sum used only as load and store addresses folds into each of
   them.** `fold_prim_addresses` (addr.ail) folded `base + k` into a
   `load64`/`store64` only when that sum had no other use, so the runtime's
   `m + 8 * H_ALLOCD`, read and then written, kept a register for the sum.
   It now folds when every use of the sum is the address of a load or store
   primitive; the sum then dies. (Folding when other uses remain was tried: it
   keeps both the sum and the base live and made `list_join_chars` spill.)
   Instructions (x86-64): wordfreq 195.5M to 192.5M, fasta 44.3M to 43.9M.

10. **A path that returns without calling holds no callee-saved
    register.** `split_for_calls` (prep.ail, ADR 0625) moves the values that
    cross calls into copies where the calls begin, so a path that returns
    first saves nothing, but only when what the first calling block
    dominates is closed. `make` in binarytrees ends in one block, `return
    node`, shared by the leaf path and the calling path, which left the
    region open and the five saves in the prologue of every call. When the
    region is open, each edge into a block that only returns (two to four
    of them) now gets a return of its own, and the function is split again;
    if that doesn't close the region the function is left as it was. The
    leaf path of `make` loses five saves, five restores and the frame's
    stack adjustment. Instructions (x86-64): binarytrees 247.0M to 228.1M.
    The same code runs for ARM64 (shared prep); its count needs the Mac.

11. **A loop that stores one unchanging value is a fill.** gcc -O2 turns
    sieve's `for _ in 0..n: is_prime.push(true)` into `memset`; here it was
    five instructions per element. `mir/fill.ail` (`fill_loops`, after the
    push-loop length pass) finds an innermost counted loop of step 1 whose
    body is constants, the steps of its one or two counters (the loop
    counter and the kept length) and a single `stx` of a loop-invariant
    value (or a constant) at a counter, and replaces it with a test of the
    trip count and a call to `rt.fill(p, n, v, size)` (`float_bits` first for
    a float). The loop's counters leave the loop as block parameters of its
    exit, holding the values the loop would have left. x86-64 assembles
    `rt.fill` as `rep stosb`/`stosq` above 64 bytes / 16 words and a loop
    below (x64/kernels.ail, the same mechanism as the tensor kernels);
    ARM64 runs the AIL body (words of the repeated byte). `rt.fill` is a
    runtime root (one line in `lower.ail`'s `RT_ROOTS`). Instructions
    (x86-64, callgrind counts each `rep` iteration as an instruction): sieve
    60.8M to 48.8M; the string instruction itself is far cheaper than that on
    hardware, but sieve at n = 60M is bound by page faults on the fresh
    list, so its wall time did not move. Nothing else among the ten changes.
    Copy loops (`memcpy`) are not done yet: they need the two lists proved
    distinct and a forward-overlap rule.

## Consequences

- A parameter's register is held from the entry, so a callee-saved register
  is not wasted on it; a parameter that lives across a call still goes to a
  callee-saved register, because the pool decides first.

## Rejected alternatives

- **Strength-reducing `n * 3` and `n * 24` to `lea`/shift pairs.** The
  `imul` is one instruction and the pair is two; the win would be latency
  only, with a second code path on each ISA.
- **Indexed forms with scale 1 for words on ARM64.** `ldr x, [n, m]` exists
  but the assembler only has the scaled `lsl 3` form; a scale of 1 on words
  is rare (a byte offset into a word array).

- **Preferring the arrival register over a hint.** A hint is a move on a
  loop edge or at a call; the arrival move happens once per call.
- **Fixing the pool order** (putting `rdi` before `rsi`). It would work for
  two parameters and swap again for three.
- **Hoisting loop-invariant wide constants into the preheader (tried,
  dropped).** A MIR pass moved every `const` that needs a 64-bit immediate
  out of call-free loops and kept it in a register (`name = "held"`, not
  rebuilt by the code generators). On x86-64 it removed nothing in the
  perfbench programs: the `movabs` instructions inside loops are the
  division-by-constant multipliers (`0xcccccccccccccccd`), which
  `magic_divide` emits itself, and `imul r/m` needs the multiplier in `rax`,
  so holding it in another register only changes `movabs rax, K` into
  `mov rax, r`. The remaining case is the hash multiplier in `rt.map_hash`
  and `rt.map_hash_str`; there the held constant took one of the few
  caller-saved registers, the leaf function then saved and restored two
  callee-saved registers, and wordfreq got worse (214.4M to 217.4M
  instructions) because the loops run about once per word. Without trip
  counts the pass cannot tell these cases apart. The case that would pay is
  ARM64, where a division multiplier costs `movz` plus three `movk` and
  `smulh` takes it in a register; that needs the multiplier as an extra
  argument of `div`/`rem` and a Mac measurement, so it is left until
  there is a way to measure ARM64 instruction counts.
- **Forwarding loads across loop back edges and joins.** It needs the
  memory state at a merge (every predecessor's), or a loop-invariance proof
  for the loop's stores and calls; the straight-line version already takes
  the repeated `len` and field reads of the programs measured, and the loop
  cases are hoisting, which `native.bce` row loads do for the common one.
- **Forwarding by alias analysis** (a store to a struct slot keeps the slots
  of another type). Every store here forgets a whole class instead; the loads
  that survive in the measured programs are in loops, where nothing carries
  over anyway.
- **Allocating `r10`/`r11` in functions that never use them as scratch.**
  `rt.alloc_raw` keeps three callee-saved registers because it needs eight
  values at once and the caller-saved pool has five. The scratch registers
  are used unconditionally by division, `neg`, `not` and compare-to-bool
  sequences and as operand reloads, so freeing one needs a per-function
  audit of every use; left for a later pass over the code generator.
12. **The map probe keeps no registers on its first-slot hit.** Profiling
    wordfreq (190 instructions per word, 3 map calls) showed `map_find` saving
    five callee-saved registers and reloading `m` from the stack around its
    probe loop, and `map_set` doing the same to store through the entry `get`
    had just found. `map_find` now hashes and calls `map_probe(m, k, h)`,
    whose first-slot hit makes no call (so ADR 0625 shrink-wrapping drops its
    saves) and which goes on in `map_probe_from` only on a collision or a
    key equal but not the same string; `map_set` stores through the
    remembered entry before calling anything. wordfreq 195.5M to 167.2M
    instructions (-14.5%); no other program of the ten uses a map.
    Rejected: folding `NAME.len()` of a constant list literal (so
    `seed % WORDS.len()` becomes a multiply, not an `idiv`). It works (a
    method arm in `lower_const.ail`, one dispatch line in `lower.ail`) but
    saved 1.0M of 167M instructions and no wall time I could measure on
    wordfreq, so it is not worth a slot in the lower.ail queue.
13. **`xs[y + k]` with `y` fixed in the loop loses its check.** nbody's
    `advance` indexes `s[bi + 1]` ... `s[bi + 6]` in an inner loop where
    `bi` belongs to the outer one. The existing versioning handled `xs[bi]`
    and `xs[j*7 + k]` but not a constant offset on a value computed outside
    the loop, so each of those accesses kept its check and re-read the list
    header after every store. The guard now tests `0 <= y` once and
    `y < len - k` for the largest `k` per (list, `y`) (native.bce.10); the
    in-loop `y + k` additions lose their overflow check because the guard
    bounds `y`. nbody 49.3M to 38.0M instructions (-22.9%; C 28.3M).
14. **An unchecked `mul` works in place.** `d = x * y` with `d` in `x`'s
    register and no overflow check (proved away, or a constant) emitted a
    copy to a scratch register, the `imul`, and a copy back. It is now one
    `imul d, y` (or `imul d, d, k`); a checked `mul` still needs the stub's
    recovery and is unchanged. spectralnorm 52.9M to 47.9M instructions
    (-9.5%; C 36.6M).
15. **`join("")` on characters is one pass.** `rt.list_join_chars` made one
    pass to OR the characters (to learn they are ASCII) and a second to store
    them. With no separator it now allocates `k` bytes, stores while it ORs,
    and returns if every character is below 128, else builds the string as
    before. fasta 36.1M to 33.4M instructions (-7.6%; matched C 29.1M).
16. **`/` and `%` by a power of two on a value that is not negative are a
    shift and a mask.** spectralnorm's `(i + j) * (i + j + 1) / 2` paid the
    four-instruction sign correction that a signed halving needs, though `i`
    and `j` are counters from 0. `mir/nonneg.ail` finds the values that cannot
    be negative: non-negative constants, `len`, a counted loop's variable when
    its start is, a checked `add` or `mul` of two such values (it is not
    negative or it traps; an unchecked one in the runtime wraps, so it proves
    nothing), `/`, `%`, `>>` and `&` of them. It runs before the bounds-check
    pass, while arithmetic still carries its trap sites, marks the `div` and
    `rem` by 2, 4, 8, ... and rewrites them last to `shr` and `band`, which
    both back ends already take. spectralnorm 52.9M to 45.4M instructions
    (-14.2%), before the in-place `imul` of item 14.
17. **A guard does not test what the loops around it already bound.** The
    inner loop of nbody's `advance` starts at `i + 1` for an outer counter
    over constants, and its guard still tested on every entry that the start
    was not negative and within +-2^30, eight compares per outer iteration.
    `iv_of` in `bce.ail` takes the range of a value from constants,
    `+ - *` of such ranges and counted loops (step 1, `p < hi`) whose bounds
    have ranges, and the guard leaves out the tests the range answers.
    nbody 38.0M to 35.5M instructions (-6.6%, on top of item 13).

## Amendment (2026-11-11)

The 8 MiB limit in item 3 is now 256 MiB of elements ([ADR 1064](1064-reserve-256-mib-ahead-of-push-loops.md)).
