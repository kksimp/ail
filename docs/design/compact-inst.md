# Compact `Inst`: cutting the self-build working set

Status: proposal (RR-1 lane). Not normative. No ADR yet; the integrator allocates numbers if a step needs one.

## Problem

The native self-build holds about 335 MB live and peaks near 1.6 GB (3x live budget plus the heap). Pacing the collector instead trades time for memory almost linearly (GCBENCH 2026-10-25: 2x budget above 256 MB cost +6.6% wall for -19.7% RSS, so pacing is shelved). The remedy is a smaller live set.

## Where the memory is

The last heap census of the self-build (`AIL_GC_STATS=1`, kinds from `heap.ail`), at 335 MB live and about 2.0M `Inst`:

| kind | objects | bytes | what |
|---|---|---|---|
| 2 (list objects) | 1.97M | 60 MB | 32 B each (`+0 len, +8 cap, +16 data`) |
| 3 (list data) | 1.97M | 114 MB | 58 B average |
| 4 (structs) | 1.73M | 80 MB | `Inst` is 7 words, 64 B block |
| 1 (strings) | 0.57M | 26 MB | |
| 5 | 0.43M | 15 MB | |

Almost every `Inst` owns an `args List[int]`, so one instruction costs a 64 B struct, a 32 B list object and a data block of at least 48 B (`list_alloc` rounds the capacity up to 4 elements, so 32 B of payload plus the header): about 144 B for an instruction whose payload is a few words.

## Steps, smallest first

Each step is its own PR with ail-linux self-build wall time and max RSS from the runner (before and after, alone), and each keeps the output byte-identical (the self-host fixed point).

1. **Exact capacity for small lists.** `list_alloc` takes the requested capacity as is below 4 (a one-element list is a 16 B block instead of 48 B, two or three elements 32 B). Growth is unchanged (`list_grow` goes to 16 from a capacity under 8), so a list that is pushed to after a literal pays one regrow, as an empty `List[T]()` does today. Estimated saving: 16 to 32 B on each small args list, about 30 to 60 MB. Touches `core.ail` only; every native program benefits. Risk: code that assumes room for 4 after `list_new`; check `list_reserve`, `ldx`/`stx` folds and `lower_list`.
2. **No data block for an empty list.** `inst(dst, op, [])` and `List[T]()` point at one shared zero-capacity block; the first push regrows. Saves 48 B on each argument-less instruction (`const`, `str`, `fnaddr`). Needs the collector to skip the shared block and `list_push`/`list_grow` to treat capacity 0 like the current path. Measure after step 1; skip if step 1 and 3 already reach the target.
3. **Inline the first operands in `Inst`.** Replace `args` by `a`, `b`, `c` plus `rest List[int]` (usually the shared empty list) for up to 3 operands, with an accessor `Inst.operands()` for the cold users. This drops the 32 B list object and the data block (80 B of 144 B, about 55% of instruction memory, an estimated 130 MB). It is a wide change: `.args` is read in about 30 files under `compiler/mir`, `x64`, `a64`; it should land as a mechanical PR (accessors first, the layout change last) through the hot-file queue, with the optimizer lane since `inline.ail`, `fold.ail`, `forward.ail` and `reuse.ail` rewrite `args`.
4. **Fewer instructions.** Count `Inst` per pass (`AIL_OPT_OFF` knobs exist for each) before and after inlining and generic instantiation; if monomorphization or inlining multiplies instructions that later passes delete, free them earlier (a function's blocks are dead once emitted).

## Rejected

- **Collector pacing** (budget 2x or 1x above a live threshold): +6.6% wall for -19.7% RSS; the cost scales with the saving.
- **A 16 B `Inst` in a typed arena**: needs arena support in the native runtime and a second representation for the interpreter-free tooling; step 3 gets most of the gain without it.
- **Packing `op` as an integer**: `op str` costs nothing per instruction (strings are shared literals); the 26 MB of strings is elsewhere.

## Order and measurement

Step 1 first (smallest, benefits every program), then re-run the census (`AIL_GC_STATS=1`) to see the new split before choosing between 2 and 3. Target: self-build max RSS under 1.1 GiB with wall time no worse than +2%.
