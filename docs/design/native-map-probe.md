# Native Map probe cost: where wordfreq's remaining instructions go

Lane 235 (day 2, 2026-10-21). Report only, no source edits. For lane 173 (maps owner) as a day-3 item.

## Setup

`tools/perfbench/programs/wordfreq.ail` at 100,000 words, native build on bd982d24 plus #359 (list constants as globals), Linux x86-64, callgrind with `objdump` symbol mapping. Per word the program does `counts.get(w) else 0` then `counts[w] = c + 1` on a 12-key `Map[str, int]` whose keys are the same string objects every time.

## Where the 278 instructions per word go

| Function | Instr/word | Share | What it does |
|---|---|---|---|
| `rt.map_hash` | 64 | 23% | FNV-1a over 3.4 bytes on average |
| `rt.map_find_h` | 69 | 25% | one probe: index load, hash compare, `map_key_eq` inlined |
| `rt.map_find` | 68 | 24% | called twice (`get`, then `set`); the second call hits the last-key cache; the first calls hash and find_h |
| `main` | 53 | 19% | the loop itself |
| `rt.map_set` | 24 | 9% | |

Design-wise the map is already lean: one hash, one probe, the second lookup skipped by the last-key cache, pointer equality short-circuits the string compare. The cost is **code quality**, not algorithm. From the disassembly:

1. **The hash loop is 12 instructions per byte** (`map_hash`, 0x40146f-0x40149a). It recomputes `rdi + 16 + i` with three instructions, re-materialises the 10-byte `movabs $0x100000001b3` every iteration, and moves the product through a scratch register. Setup, finaliser and call overhead add about 24 more. 3.4 bytes x 12 + 24 = 65, which matches the 64 measured.
2. **A field load costs three instructions**, not one: `mov %rsi,%r9; add $0x18,%r9; mov (%r9),%r9` for `load64(m + 24)`, instead of `mov 0x18(%rsi),%r9`. `map_find_h` has about a dozen such loads and `map_find` three; the same pattern is in every `store64`, list index and struct field access in all native code.
3. **`map_find_h` pushes and pops five callee-saved registers and keeps four values in stack slots** (`-0x30..-0x48(%rbp)`) for one probe, because it holds m, k, h, mask, idx, ents, kind and the entry address live across the inlined `map_key_eq` loops. About 25 of its 69 instructions are prologue, epilogue and spill traffic on the common path (first probe, pointer-equal key).
4. `imul $8` and `imul $0x18` compute scaled addresses instead of shifts and `lea`.

## What the C twin does

`wordfreq.c` has 12 constant words and counts into `long counts[12]` with `seed % 12`: no hashing, no probing. It is a lower bound, not a fair map comparison. A hand-written C open-addressing map with FNV-1a would still take roughly 40-60 instructions per word, so the honest target for native AIL is about 100-130 instructions per word, 2-2.5x of that, not 1x of the array.

## Proposed changes, largest saving first

| # | Change | Where | Est. saving per word | Also helps |
|---|---|---|---|---|
| 1 | **Hash a string eight bytes at a time.** For `n <= 8` one `load64(s + 16)` masked to `n` bytes, multiplied and mixed; for longer strings 8-byte rounds plus a masked tail. The string object is 16-aligned and its allocation is rounded up to 16 bytes, so reading 8 bytes at +16 never leaves the object for a non-empty string (the empty string needs a special case); literals in read-only data need 8 bytes of padding. The hash value changes, so any code that stores or compares hashes (`Set` of str, `ail` runtime hash tables) must go through `map_hash_str` only, which it does today. | `compiler/rt/native/map.ail` `map_hash_str` | ~45 instr (16%) | every `Map[str, _]` and `Set[str]` |
| 2 | **Fold `base + constant` into the load/store displacement** (and `base + index*scale` into the addressing mode) in the x64 and a64 generators: `load64(p + 24)` becomes one `mov 0x18(%rdi),%rax` / `ldr x0,[x0,#24]`. Needs a peephole over `add` with a constant whose only use is the memory operand. Both arches, lockstep (ADR 0008). | `compiler/x64/gen.ail`, `compiler/a64/gen.ail`, or a MIR addressing pass | ~40 instr (14%) here; ~5-10% on any program that reads fields or list elements | everything |
| 3 | **Split `map_find_h` into a small first-probe function and an out-of-line slow path.** The fast path needs only m, k, h and two loads and keeps no callee-saved registers; the key-compare loop (`map_key_eq` words and bytes) moves to a separate function called only when hashes match but pointers differ. Saves the five pushes/pops and the four spills on the hit path. | `compiler/rt/native/map.ail` | ~20 instr (7%) | every map lookup |

Combined estimate: 278 -> about 170 instructions per word (-39%), i.e. native wordfreq from 0.037 s to roughly 0.024 s, about 2.6x of the C array twin and about 10x faster than CPython (0.236 s).

Smaller items, not worth a PR alone: hoist loop-invariant `movabs` constants out of loops (a MIR or codegen pass; lets loops like the hash loop drop 2 instructions per iteration), replace `imul $8/$0x18` by shift or `lea`, and skip the `map_find` call frame for the last-key-cache hit by testing the cache at the call site.

## Not recommended

- **Caching the hash in the string object.** The string header has no spare word (`+0 bytes`, `+8 chars`, `+16 data`, native-abi.md). Growing the header by 8 bytes changes the ABI, every literal, and the allocator size classes, to save a cost that change 1 mostly removes.

## How to reproduce

```
./ail build tools/perfbench/programs/wordfreq.ail -o wf
valgrind --tool=callgrind wf 100000        # then callgrind_annotate; symbols from objdump -d
objdump -d --no-show-raw-insn wf           # rt.map_hash, rt.map_find_h, rt.map_find
```

## Prototype of change 2 (displacement folding), 2026-10-21 11:30 UTC

Branch `lane/235-disp-fold` (WIP, no PR, not for merge tonight; lane 120 owns it from day 3).

`compiler/mir/disp.ail` adds `fold_displacements`: a `load64`/`loadf`/`store64`/`storef` prim whose address is `y + k` or `y - k` (constant `k`, `add` without an overflow check) becomes the existing `load`/`store` op with displacement `k`. Both generators already take `imm` on those ops, so x64 and a64 get it from one MIR pass (ADR 0008 lockstep holds without touching the generators). It only folds `k` that one a64 instruction can encode (-256..255, or a multiple of 8 below 32768). It runs on `rt.*` functions too, which skip the other late passes. Not folded: `load8`/`store8` (the hash loop's byte loads need base + index + 16, which needs an indexed addressing mode, a separate change).

Valgrind callgrind instruction counts, Linux x86-64, reduced arguments, before = bd982d24 + #359, after = plus the pass. Outputs are identical before and after on all nine programs.

| Program (args) | Before | After | Change |
|---|---|---|---|
| wordfreq (100000) | 27,829,087 | 25,328,160 | -9.0% |
| binarytrees (10) | 14,390,695 | 13,547,547 | -5.9% |
| print_many | 250,225,800 | 240,325,800 | -4.0% |
| fannkuch (8) | 20,792,326 | 20,590,661 | -1.0% |
| matmul (60) | 3,481,119 | 3,451,001 | -0.9% |
| spectralnorm (50) | 2,394,223 | 2,388,209 | -0.3% |
| nbody (5000) | 5,253,176 | 5,251,997 | 0.0% |
| collatz (20000) | 20,402,728 | 20,402,672 | 0.0% |
| sieve (300000) | 7,707,662 | 7,707,593 | 0.0% |

wordfreq saves 25 instructions per word, less than the 40 I estimated: the field loads are now one instruction, but `map_find_h` still spends its prologue and spills (change 3) and the hash loop is untouched (change 1). Compute programs don't gain because their hot loops use `ldx`/`stx`, which ADR 0625 already folds. `fasta` and `mandelbrot` still fail to build natively (`.join()` on `List[char]`), so they have no row.

Tested: `./ail selftest --test native` passes on x86-64 and on arm64 under qemu (same outputs as `ail run`), and the compiler cards are regenerated. The full `./ail selftest` was not run on this branch.

## Day 3: changes 1 and 3 (2026-10-21, lane 235)

Branch `lane/235-day3` (PR against claude/project-thread-ipafhh), ADRs 0766 and 0767. Callgrind, wordfreq at 100,000 words, on batch head 079c6d8 (without the displacement pass):

| Step | Total instructions | `map_hash` /word | `map_find_h` /word |
|---|---|---|---|
| head | 27,838,487 | 64 | 69 |
| + 8-byte hash (0766) | 25,654,385 | 44 | 67 |
| + first probe (0767) | 25,479,264 | 44 | 43 (+22 in `map_probe` on collisions) |

-8.5% in all, against the 39% the three changes were estimated to give together (the displacement pass, measured separately, adds another 9%). The estimates were high because the generated code for the new loops is as wasteful as the old (a three-move swap of the two argument registers at each function entry, `movabs` constants rebuilt in every loop iteration, `imul $8` for `8 * x`), so each saved source-level operation still costs two or three instructions. Wall-clock on the shared box did not separate (best of 21 0.039 s before, 0.041 s after; median 0.053 s, 0.047 s). The next gains in map-heavy code are in code generation, not in `map.ail`: the argument-register swap alone is three instructions in every function.
