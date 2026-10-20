# 0582. Number text through a byte builder; format specifications and `to_float` in the runtime

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by: AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)**
- **Topic:** Native runtime and ABI (lane 177)
- **Spec rules:** [lex.str.7](../spec/02-lexical.md), [type.str.11](../spec/04-types.md), [type.num.7](../spec/04-types.md), [type.num.8](../spec/04-types.md)
- **Design note:** [native-abi.md](../design/native-abi.md) §6 and §8

## Context

ADR 0409 prints floats by exact digit generation, writing each byte to the
output buffer. Lane 172's `to_str` and interpolation need the same text as a
string object, and interpolation takes `{x:>8}`, `{x:08.3}` (written
`{x:0>8.3}`), `{x:x}`; `s.to_float()` must read back exactly what was
printed (type.num.8).

## Options considered

### Option A — a growable byte builder in the runtime; `put_*` write to it (chosen)
- **Pros:** one implementation for printing and for strings; printing a
  number allocates nothing (the builder is reused); padding is a copy.
- **Cons:** a few hundred bytes of global state (three slots).

### Option B — separate string-returning and printing versions
- **Pros:** no shared state.
- **Cons:** two copies of ~300 lines of digit generation.

### Option C — print floats to strings only, allocate per call
- **Pros:** smallest code.
- **Cons:** garbage per printed number, with the bump allocator never freeing.

## Decision

A. `compiler/rt/native/fmt.ail` (moved from `core.ail`): `put_int`, `put_hex`,
`put_float`, `put_float_fixed` write into the builder; `fmt_int`, `fmt_hex`,
`fmt_bool`, `fmt_float`, `fmt_float_fixed` return string objects;
`fmt_pad(s, width, fill, align)` pads by characters (`<` left, `>` right,
`^` centre with the odd one right; numbers default right, everything else
left, as the interpreter does); `out_float` and `out_float_fixed` print from
the builder. `lower_os.ail` provides `Lower.fmt_text(value, spec)` for lane
172's interpolation and uses it in `print`; a float with only `.N` still
writes straight to the output buffer. `str_to_float` reads the grammar of
type.num.8 and rounds the exact value to nearest even: the digits become a
big integer N, the value N·10^e is scaled by a power of two until the
quotient has 54 bits, found by 54 compare-and-subtract steps on the same
base 10^9 big numbers as the digit generator, then rounded in 64-bit
integers; overflow gives infinity, underflow a subnormal or zero. No code or
tables come from another project.

## Rationale

Criterion: AI understanding (exact big-integer arithmetic that needs only
`int` and `float`, checked against the interpreter), then token usage of the
runtime source. Speed is secondary: number formatting isn't the hot path of
the benchmarks (ADR 0409).

## Consequences

- The `os_fmt.ail` test prints specs, subnormals, ties and overflow and
  matches `ail run` on both architectures.
- `to_float` costs a few big multiplications per call; a table-driven
  version can replace it with the same tests (ADR 0409's plan).

## Rejected alternatives

B: duplicates the hardest code. C: leaks.
