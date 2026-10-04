# Syntax token measurements (round 1)

> **Non-normative measurement note,** 2026-10-04. This is the first use of the
> token benchmark method ([ADR 0029](../decisions/0029-token-efficiency-benchmarks.md),
> layers 1–2). The code below is **illustrative candidate syntax**, not decided
> AIL syntax.

## Method

- The same small program (an expression tokenizer: an enum, loops, branches, a
  `match`, and error propagation) was written in four candidate block styles
  and in Rust and Go for reference.
- Tokens were counted with five real tokenizers:

| Name | Source | Notes |
|---|---|---|
| `cl100k` | OpenAI `cl100k_base` (tiktoken file bundled in the `litellm` 1.104.0 wheel) | GPT-4-era |
| `o200k` | OpenAI `o200k_base` (same source) | GPT-4o-era |
| `anthropic-legacy` | `anthropic_tokenizer.json` (same source) | An older Claude tokenizer. **Current Claude tokenizers are not public**, so real counts may differ. |
| `mistral-tekken` | `tekken_240911.json` (`mistral-common` 1.12.0) | Recent Mistral |
| `mistral-spm-v1` | `tokenizer.model.v1` (`mistral-common` 1.12.0) | Older SentencePiece-style tokenizer, poor with whitespace |

- **Constraint:** the official tokenizer download hosts are blocked in the
  measurement environment, so the files came from PyPI packages that bundle
  them. Results are exact for these files.

## Result 1: block structure (whole program)

| Variant | cl100k | o200k | anthropic-legacy | mistral-tekken | mistral-spm-v1 | Mean | vs. A |
|---|---|---|---|---|---|---|---|
| **A. Indentation** | 181 | 182 | 201 | 198 | 243 | 201.0 | — |
| B. Braces, no semicolons | 197 | 198 | 222 | 214 | 269 | 220.0 | +9.5% |
| C. `end` keyword | 200 | 201 | 215 | 217 | 262 | 219.0 | +9.0% |
| D. Braces + semicolons | 198 | 199 | 232 | 214 | 278 | 224.2 | +11.5% |
| Rust (reference) | 237 | 239 | 272 | 257 | 330 | 267.0 | +32.8% |
| Go (reference) | 311 | 313 | 356 | 328 | 419 | 345.4 | +71.8% |

Findings:
- Indentation saves **~9–10%** over braces or `end` on this program.
- Braces and `end` cost about the same.
- Semicolons are almost free in OpenAI tokenizers (`;` merges with the
  newline) but cost ~5% in others.
- Every candidate AIL style is **~18–25% smaller than Rust** and **~35–42%
  smaller than Go** on this sample. Most of that comes from type and keyword
  design, not from block style.

## Result 2: individual constructs (tokens per snippet)

| Construct | cl100k | o200k | anthropic-legacy | tekken | spm-v1 |
|---|---|---|---|---|---|
| `fn` / `func` / `function` / `def` keyword | 9 / 9 / 9 / 9 | same | 10 each | 10 each | 12 each |
| Signature `a: i32, b: i32) -> i32` vs `a i32, b i32) i32` | 17 vs **14** | 17 vs **14** | 18 vs **15** | 20 vs **17** | 22 vs **19** |
| Brief's example, Rust-style vs AIL-style (whole function) | 23 vs **18** | 23 vs **18** | 26 vs **18** | 26 vs **21** | 30 vs **22** |
| Generics `Result[List[User], E]` vs `Result<List<User>, E>` | 14 vs 13 | 14 vs 13 | 16 vs 16 | 15 vs 13 | 18 vs 18 |
| `count := 0` vs `let count = 0` | **7** vs 8 | **7** vs 8 | **5** vs 6 | **7** vs 8 | **8** vs 9 |
| `and`/`not`/`or` vs `&&`/`!`/`\|\|` | 10 vs 10 | same | 9 vs 9 | 10 vs 10 | 11 vs 11 |
| `i64` vs `int64` | 10 vs 10 | same | 8 vs 8 | 11 vs 11 | 12 vs 12 |

Findings:
- **Keyword length doesn't matter** for common words: `fn`, `func` and
  `function` are each one token. Keywords can be chosen for clarity at no cost.
- **Dropping `:` and `->` saves ~3 tokens per signature** (~15–18%).
- **`[]` generics cost up to 1 token more** than `<>` in some tokenizers.
  That's a small price for removing the `a < b > c` ambiguity
  ([ai-author-wishlist.md 3.4](ai-author-wishlist.md)).
- `:=` saves one token per declaration compared with `let`.
- Word operators (`and`, `or`, `not`) cost the same as symbols.

## Addendum: indentation, signatures, declarations

Whole program (variant A) re-indented with each unit:

| Indent unit | cl100k | o200k | anthropic-legacy | mistral-tekken | mistral-spm-v1 | Mean |
|---|---|---|---|---|---|---|
| 4 spaces | 181 | 182 | 201 | 198 | 243 | 201.0 |
| 2 spaces | 181 | 182 | 201 | 198 | 242 | 200.8 |
| 3 spaces | 181 | 182 | 201 | 198 | 242 | 200.8 |
| 8 spaces | 181 | 182 | 201 | 198 | 259 | 204.2 |
| Tab | 184 | 185 | 227 | 201 | 268 | 213.0 (+6%) |

Signature `fn tokenize(src ..., limit ...) Result[List[Token], LexError]`:

| Form | cl100k | o200k | anthropic-legacy | tekken | spm-v1 |
|---|---|---|---|---|---|
| `src: str, limit: i64) -> R` | 20 | 20 | 23 | 23 | 25 |
| `src: str, limit: i64) R` | 19 | 19 | 22 | 22 | 24 |
| `src str, limit i64) R` | **17** | **17** | **20** | **20** | **22** |

Three lines (declare an int, declare a string, reassign the int):

| Form | cl100k | o200k | anthropic-legacy | tekken | spm-v1 |
|---|---|---|---|---|---|
| `count := 0` … | **20** | **20** | **17** | **20** | **21** |
| `mut count := 0` (mutable marked) … | 21 | 21 | 18 | 21 | 22 |
| `let count = 0` / `var count = 0` … | 22 | 22 | 19 | 22 | 23 |
| `let mut count = 0` … | 23 | 23 | 20 | 23 | 24 |

## Caveats

- One program is not a corpus. These results guide the first syntax
  questions. The full corpus (ADR 0029) must confirm them.
- Token count is only one input. Readability, ambiguity, and **edit
  robustness** (see [ai-author-wishlist.md 3.2](ai-author-wishlist.md)) weigh
  just as much.

## Appendix: variants A and B

Variant A (indentation):

```
type Token enum
    Num(i64)
    Op(char)
    LParen
    RParen

fn tokenize(src str) Result[List[Token], LexError]
    tokens := List[Token]()
    i := 0
    while i < src.len
        c := src[i]
        if c.is_space()
            i += 1
            continue
        if c.is_digit()
            start := i
            while i < src.len and src[i].is_digit()
                i += 1
            tokens.push(Token.Num(src[start..i].parse_int()?))
            continue
        match c
            '(' => tokens.push(Token.LParen)
            ')' => tokens.push(Token.RParen)
            '+', '-', '*', '/' => tokens.push(Token.Op(c))
            _ => return Err(LexError.Unexpected(c, i))
        i += 1
    Ok(tokens)
```

Variant B (braces):

```
type Token enum {
    Num(i64)
    Op(char)
    LParen
    RParen
}

fn tokenize(src str) Result[List[Token], LexError] {
    tokens := List[Token]()
    i := 0
    while i < src.len {
        c := src[i]
        if c.is_space() {
            i += 1
            continue
        }
        if c.is_digit() {
            start := i
            while i < src.len and src[i].is_digit() {
                i += 1
            }
            tokens.push(Token.Num(src[start..i].parse_int()?))
            continue
        }
        match c {
            '(' => tokens.push(Token.LParen)
            ')' => tokens.push(Token.RParen)
            '+', '-', '*', '/' => tokens.push(Token.Op(c))
            _ => return Err(LexError.Unexpected(c, i))
        }
        i += 1
    }
    Ok(tokens)
}
```

Variants C and D follow B with `end` lines, or with semicolons and commas,
respectively.
