# 0173. Format specifications in interpolation: `{x:>8}`, `{r:.2}`, `{n:0>4x}`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 2 — Core syntax; 21 — Standard library architecture
- **Spec rules:** [lex.str.7](../spec/02-lexical.md), [type.str.11](../spec/04-types.md)
- **Resolves:** [OQ-041](../open-questions.md)
- **Evidence:** `tools/tokbench/corpus/r8/`

## Context

`"{x}"` gives only the default text form. Width and fill were written with
`pad_left`/`pad_right` ([ADR 0135](0135-string-literals-and-methods.md)), hex
with `to_hex()`, and fixed decimal places couldn't be written at all.

## Measurement

`corpus/r8/` holds one program that prints a table and money, written with
specs (`f1_spec.ail`) and with methods (`f2_methods.ail`: `pad_left`,
`pad_right`, `to_hex`, a hypothetical `to_fixed(2)`):

| Variant | cl100k | o200k | claude-legacy | mistral-tekken | mistral-spm-v1 | mean |
|---|---|---|---|---|---|---|
| specs | 185 | 188 | 183 | 205 | 229 | 198 |
| methods | 242 | 242 | 256 | 264 | 301 | 261 |

Specs cost about 25% fewer tokens on every tokenizer, far above the 2% tie
threshold of ADR 0076. **Tokens** decide.

## Options considered

### Option A — specs after `:` (chosen)
- **Pros:** about 25% fewer tokens; one local place to read a line's shape;
  familiar to Python and Rust authors.
- **Cons:** a small mini-language to learn. Kept minimal to limit that.

### Option B — methods only (`x.to_fixed(2)`, `pad_left`)
- **Pros:** no new syntax; one way to do each thing.
- **Cons:** about 33% more tokens; the interpolation grows long.

### Option C — both
- **Pros:** none beyond A.
- **Cons:** two spellings of one thing. The existing `pad_left`, `pad_right`
  and `to_hex` stay because they work on values outside a string; no
  `to_fixed` is added.

## Decision

1. `{expression:spec}` where spec is `[[fill]align][width][.decimals][x]` and
   nothing else (`lex.str.7`). The `:` is the first one outside brackets,
   strings and character literals, so `{m["a"]}` and `{f(x: 1)}` are unaffected.
2. `.N` needs a `float` and `x` an `int` (`type.str.11`, compile-time errors
   otherwise). Floats round the exact value half to even.
3. No `0` flag: zero padding is `0>4`, the same fill-and-align form as any
   other. No sign, `#`, `,`, `e` or `{x!r}`.
4. Default alignment: numbers right, other values left.

## Consequences

- Lexer, parser, checker and interpreter changed locally; `Part::Fmt` and
  `InterpPart::Fmt` carry the parsed specification.
- A later `Display`-style hook for user types is not decided here.

## Rejected alternatives

- **A `0` flag and `f`/`d` type letters (Python):** extra rules that the
  fill-and-align form and the float/int typing already cover.
- **Separate `format(...)` function:** longer, and the interpolation already
  carries every other expression.
