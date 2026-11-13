# 0465. Bit operations are `int` methods: `bit_and`, `bit_or`, `bit_xor`, `bit_not`, `shl`, `shr`

- **Status:** Superseded by [1070](1070-bit-operators.md) (the methods stay; the operators are added)
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 11 — Operators; integer arithmetic
- **Spec rules:** new `expr.bit.1` and `expr.bit.2`; the operator set of
  [ADR 0077](0077-operator-traits.md) is unchanged
- **Resolves:** none (lane 140 of the batch 2 plan)

## Context

The operator set is fixed by [ADR 0077](0077-operator-traits.md). Bit
operations were missing: flags, hash mixing and bitsets had no spelling. The
batch brief asks for `& | ^ ~ << >>` or named methods, decided by measurement.

## Measurement

`tools/tokbench/run.sh --json` on six small programs: three in the method
spelling and their three operator twins (flag set, a hash mix over bytes, a
bitset). Counts are tokens per file; the total is over the three programs of
each spelling.

| Program | Spelling | cl100k | o200k | claude-legacy | mistral-tekken | mistral-spm-v1 |
|---|---|---|---|---|---|---|
| flags | methods | 91 | 92 | 108 | 103 | 119 |
| flags | operators | 80 | 80 | 69 | 84 | 88 |
| hash mix | methods | 78 | 80 | 91 | 90 | 110 |
| hash mix | operators | 75 | 75 | 74 | 83 | 94 |
| bitset | methods | 120 | 120 | 143 | 124 | 159 |
| bitset | operators | 111 | 111 | 120 | 114 | 138 |
| **total** | methods | 289 | 292 | 342 | 317 | 388 |
| **total** | operators | 266 | 266 | 263 | 281 | 320 |

Operators save 8% (cl100k) to 23% (claude-legacy) on these bit-only files.
Those files contain nothing but bit code, so the saving is the bit code's own.

## Options considered

### Option A — named methods (chosen)
- **Pros:** no change to the fixed operator set of ADR 0077; no lexer change
  (`<`, `>`, `|`, `&`, `^`, `~` are all tokens that already have meanings);
  no precedence table entry; methods already exist on `int`, so the checker,
  both tiers and the formatter need only new table rows.
- **Cons:** 8–23% more tokens on bit-only code; `shl` and `shr` read less
  familiarly than `<<` and `>>`.

### Option B — operators `& | ^ ~ << >>`
- **Pros:** fewer tokens on the bit code (table above); familiar to C, Rust
  and Python readers.
- **Cons:** amends the fixed operator set of ADR 0077; `<<` and `>>` need
  a precedence decision against comparisons and `.`; bit code is confined to
  a rare slice of programs, so the whole-program token gain is far under the
  2% of ADR 0076, which counts as a tie.

### Option C — function-style `bit_and(a, b)`
- **Cons:** the receiver-first method form is already how `int` works
  (`n.abs()`), so a free function would be the only one of its kind.

## Decision

Option A. On ADR 0076's order, token usage would favour operators on
bit-only code, but bit operations are a rare construct and the whole-program
difference is a tie. Human readability and the fixed operator set then decide
for methods. The table is recorded here so the designer can revisit it; a
designer decision for operators would be an amendment of ADR 0077.

## Semantics

- `bit_and`, `bit_or`, `bit_xor` take an `int`; `bit_not` takes none. They work
  on the 64-bit two's complement value (`expr.bit.1`).
- `shl(n)` discards the bits that leave the top. `shr(n)` is arithmetic: the
  sign bit fills the vacated positions, so `(-8).shr(1)` is `-4`.
- A shift count outside `0..=63` traps with `expr.bit.2`. The trap is
  the choice here: a silent result would hide a bug in a bit mask, and the
  alternative (masking the count, as C's hardware does) makes `1.shl(64)`
  equal `1`, a surprise for readers.
- Sized integers are not covered yet. Their methods are computed in
  `i128` and need a per-width mask for `shl`; that is a follow-up.

## Consequences

- `int` has six more methods in the checker (`check/builtins.rs`) and the
  interpreter (`interp/builtins.rs`), which the bytecode tier shares.
- The bootstrap's self-hosted checker (`compiler/check`) has no builtin method
  table, so it needs no port for this change; `self_hosted_check` passes.
- Tests: `bootstrap/tests/cases/bit_ops.ail` (positive, every method and
  both signs), `bit_shift_range.ail` (trap `expr.bit.2`) and
  `bit_bool_operand.ail` (negative: `bool` has no `bit_and`).
- Rejected alternatives are recorded above.

## Rejected alternatives

- Fixed operators `& | ^ ~ << >>` (option B): rejected for now. Operators save 8% to 23% on bit-only code, but bit code is a rare construct, so the whole-program gain is a tie under ADR 0076. The operator form would also amend the fixed operator set of ADR 0077, and `<<` and `>>` need a precedence decision. The table is kept so the designer can revisit it.
- Function-style `bit_and(a, b)` (option C): rejected because the receiver-first method form is already how `int` works (`n.abs()`).
