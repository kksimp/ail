# 1170. `to_float() else` returns a marker NaN for a miss; one-byte split reads word marks

- **Status:** Accepted
- **Date:** 2026-11-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked on 2026-11-17 for lanes on the open slowdowns against C, runtime speed only, no syntax change
- **Topic:** Native runtime: text
- **Spec rules:** type.num.8, type.str.4, err.else.1-2 (no observable change)
- **Builds on:** [ADR 0766](0766-native-string-hash-by-words.md), [ADR 0823](0823-lower-ail-feature-files.md)

## Context

`s.to_float() else x` called `rt.str_float_opt`, which allocated a zeroed
box for every number and read the per-thread `parse_ok` flag twice. On the
Mac the parse loop took 1.2 times its C twin. `s.split(" ")` on one thread
took 1.19 times its C twin: the scan found a word holding a separator and
then walked its eight bytes one at a time, so short fields were read
byte by byte.

## Options considered

### Option A — a marker NaN for the miss, and exact marks for the split (chosen)

`rt.str_float_try` returns the number, or the NaN with bits
`0x7FF8000000000001` when the text is not a number. The lowering
(`compiler/mir/lower_tofloat.ail`) compares the result's bits with that
constant and takes the `else` on a match. The parser builds every result
from bits it computes: the text `nan` gives `0x7FF8000000000000`, and no
other text gives a NaN, so no text parses to the marker. The test is on
bits, never `x != x`, so `"nan".to_float() else ...` is present.

`str_split1` computes a word with 128 in each byte equal to the separator
and nowhere else, masks off bytes past the end of the text, and takes each
separator's byte number from the lowest mark with one multiplication.

- **Pros:** no allocation and no flag per parse; no byte loop in the split;
  the plain `s.to_float()` (a `float?` value) is unchanged.
- **Cons:** one constant shared by the runtime and the lowering.

### Option B — keep the flag, as `to_int() else` does

- **Cons:** two lookups of the per-thread flag per parse (a call each once
  threads exist); only the allocation goes.

### Option C — return the optional unboxed in two registers

- **Cons:** a second result register for runtime calls is a new calling
  convention on four backends for one method.

## Decision

Option A. `flt_scan` takes a `note` argument: `str_to_float` passes true
and sets `parse_ok` as before; `str_float_try` passes false. `flt_scan`
also loads each digit once and finishes numbers under 2^53 with an exponent
within 22 itself. `str_to_int` finds its flag's address once.

## Rationale

ADR 0076 order: programs and diagnostics do not change, so speed decides.
A removes both costs; B leaves the flag; C costs far more for the same gain.

## Consequences

- Tests `str_to_float_else` (including `nan`, `inf`, and misses) and
  `str_split_marks` (a separator at every position of texts of 1 to 20
  bytes).
- Against main f84136bf (2026-11-20, `textgap-mac-1f077488.txt`,
  `textgap-linux-1f077488.txt`). Mac arm64: `floatparse` 0.884 s to 0.718 s
  (C twin 0.705 s), `fields` 0.586 s to 0.436 s (C twin 0.480 s),
  `splitline` 0.068 s to 0.060 s. ail-linux x86-64: `floatparse` 1.621 s to
  1.232 s, `fields` 1.139 s to 1.037 s (C twin 1.115 s), `splitline` on one
  thread 0.350 s (C twin 0.341 s).
- The marker must stay a NaN the parser cannot produce. A change that lets
  text choose a NaN payload must move the miss to another channel.

## Rejected alternatives

- **Option B:** leaves two flag lookups per parse.
- **Option C:** a new calling convention for one method.
