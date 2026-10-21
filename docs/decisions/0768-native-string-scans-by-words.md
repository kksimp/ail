# 0768. Native string scans work a word at a time, and interpolation joins in one allocation

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 23 — Compiler architecture; performance
- **Spec rules:** type.str.3, type.str.6, type.str.7
- **Builds on:** [ADR 0766](0766-native-string-hash-by-words.md), [ADR 0565](0565-native-string-runtime-calls.md)

## Context

`tools/perfbench/agent/lines_split.ail` (200,000 generated log lines, split and
counted) ran 0.56 s natively, slower than CPython, and callgrind put 33% of its
2.46 billion instructions (after the allocator work in #364 and #367) in
`rt.str_sub`, `rt.str_cat`, `rt.str_split` and `rt.str_lines`:

- `u8_count` counted a piece's characters one byte at a time (about 10
  instructions per byte), in every `str_sub`.
- `bytes_find` compared byte by byte from every offset, for a one-byte
  separator too.
- An interpolation of k pieces made k-1 `str_cat` calls, k-1 allocations, and
  copied the growing prefix each time.

## Decision

1. **`u8_count` counts eight bytes at a time.** A word's continuation bytes
   (`10xxxxxx`) are `w & ~(w << 1) & 0x80..80`, one bit per byte, summed by a
   multiply; an all-ASCII word (no high bit) adds eight without that. The last
   partial word is masked to its bytes. Reading the whole last word is safe for
   the reason given in ADR 0766.
2. **`bytes_find` looks for the first byte of the separator eight bytes at a
   time** (a word xor the byte repeated has a zero byte at each hit, found by
   the subtract-and-mask test, whose lowest flagged byte is exact), then checks
   the rest of the separator.
3. **Interpolations join up to six pieces in one allocation**: `rt.str_cat3`
   to `rt.str_cat6` (the existing `rt.str_cat` for two). Longer ones join five
   more pieces onto what is built so far. The lowering evaluates the pieces in
   the same order as before; only the joining changes.

## Result (Linux x86-64, `lines_split`, callgrind)

| Step | Instructions | Best of 21 |
|---|---|---|
| head with #364 and #367 | 2,457,895,220 | 0.417 s |
| + word-at-a-time `u8_count` | 2,310,984,647 | |
| + word-at-a-time `bytes_find` | 2,174,210,367 | |
| + interpolation in one allocation | 1,794,127,327 | 0.280 s |

-27% instructions; wall-clock 0.56 s on the unchanged head, 0.417 s with the
allocator work, 0.280 s with this. CPython 0.477 s, C 0.070 s (best of 21 on a
shared box). `rt.str_cat` (324 M) is gone; the largest remaining costs are the
allocator (`rt.alloc_z`, 309 M, lane 171's), `rt.str_put` (166 M, the per-piece
copy call) and `rt.str_sub` (151 M, one allocation per field).

## Rejected alternatives

- **A builder object for interpolation** (`rt.sb_*` exists for formatting):
  needs growth checks per piece; the pieces are known when lowering, so one
  sized allocation is cheaper.
- **`memchr`-style search with a hardware instruction:** the baseline has no
  such instruction guaranteed on both architectures; the word test needs only
  integer operations.
- **One variadic `str_cat`:** the call ABI has no varargs; fixed arities up to
  six fit the register arguments on both architectures.

## Consequences

- `bootstrap/tests/native/run/str_count_chars.ail`, `str_find_words.ail` and
  `str_interp_wide.ail` compare native and `ail run` on mixed-width text,
  separators of 1-4 bytes and interpolations of 1-13 pieces.
- `compiler/mir/lower_str.ail` changes only the joining; `lower.ail` is not
  touched, so this is not in the lower.ail queue.
