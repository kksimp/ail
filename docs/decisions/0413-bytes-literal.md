# 0413. The `bytes` literal is `b"..."`, ASCII with `\xHH`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 17 — Binary data
- **Spec rules:** [lex.bytes.1](../spec/02-lexical.md#string-and-character-literals)


## Context

Binary data needs a literal for magic numbers and protocol text. ADR 0029
says to back syntax with measurements.

## Measurement

The same four-function program (a PNG signature check, an HTTP request
header, a two-line `main`) written with each candidate; `tools/tokbench/run.sh
--json`, tokens per tokenizer (lower is better):

| Form | cl100k | o200k | claude-legacy | tekken | spm-v1 | mean |
|---|---:|---:|---:|---:|---:|---:|
| `b"\x89PNG\r\n..."` | 77 | 76 | 101 | 80 | 111 | **89.0** |
| `"...".to_bytes()` (text only; can't say `\x89`) | 91 | 90 | 114 | 92 | 124 | 102.2 |
| `bytes.from_hex("89504e47...")` | 114 | 113 | 129 | 156 | 175 | 137.4 |
| `bytes.from_list([137, 80, ...])` | 183 | 182 | 155 | 240 | 263 | 204.6 |

## Options considered

### Option A — `b"..."` with ASCII text and `\xHH` (chosen)
- 13% fewer tokens than the nearest form that can't even express every
  byte, and 35% fewer than hex. Escapes match `lex.str.1`, so nothing new
  to learn beyond `\xHH`.

### Option B — `"...".to_bytes()` only
- **Cons:** cannot write `0x89` or any non-UTF-8 byte; allocates at run time.

### Option C — `bytes.from_hex("...")`
- **Cons:** 54% more tokens, fallible (`bytes?`), unreadable for text.

### Option D — `b"..."` accepting any Unicode, encoded as UTF-8
- **Cons:** two ways to spell text bytes, and a byte count that depends on
  the editor's normalisation. Non-ASCII is an error whose help says to use
  `\xHH` or `"...".to_bytes()`.

## Decision

Option A. `b"..."` is one line, ASCII plus the escapes `\n \t \r \0 \\ \"
\xHH`; `{` is literal (no interpolation). The lexer keeps the bytes in the
token; the syntax API shows them as lowercase hex.

## Consequences

- `lex.bytes.1` is the rule for every lexical error in a literal (bad escape,
  non-ASCII, missing quote). No triple-quoted or raw form: a multi-line
  payload is rare, and `"...".to_bytes()` covers text.
- The AIL lexer, parser and syntax JSON gained `Tok.Bytes(hex)` and
  `Expr.Bytes(hex, pos)`.
