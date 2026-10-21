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
   (2^20 words, 2^23 bytes). sieve 3,000,000: 124.4M to 79.4M instructions.

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
