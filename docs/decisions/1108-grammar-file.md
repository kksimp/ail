# 1108. A machine-checked grammar file

- **Status:** Accepted
- **Date:** 2026-11-13
- **Decided by:** AI (delegated, ADR 0076), as review package S17 of the AI-first review that the designer approved for 0.1.0 on 2026-11-12
- **Topic:** Grammar ([03-grammar.md](../spec/03-grammar.md))
- **Spec rules:** `gram.ebnf.1`; completes ADR 0002's machine-checkable grammar

## Context

The grammar chapter described only function declarations, generics and attributes. An
agent that wants to know whether a construct exists had to read the parser (3,000 lines)
or the language card, and a formal grammar was promised by ADR 0002 and never written.

## Options considered

### Option A: one EBNF file, checked against the whole tree by a Python tool (chosen)
- **Pros:** one place lists every construct (AI understanding); the tool lexes each `.ail`
  file that compiles and recognises it, so a parser change that the grammar misses fails
  the docs gate; the recogniser handles ambiguity by tracking every parse, so the grammar
  can be written for readers and stay close to the parser's own shape.
- **Cons:** the tool has its own small lexer, which can drift from `compiler/syntax/lex.ail`;
  it reads the keyword list from that file, and a lexer change shows as a rejected file.

### Option B: a grammar generated from the parser
- **Cons:** the parser is hand-written with error recovery for foreign syntax; a generated
  grammar would be the parser's code in another notation, and no shorter.

### Option C: grammar prose in the chapter only
- **Cons:** can't be checked, so it would drift; that is the state this replaces.

## Decision

`docs/spec/grammar.ebnf` covers the whole surface, with `NEWLINE`, `INDENT`, `DEDENT` and
`AFTER_BLOCK` (a statement that ends with a block needs no `NEWLINE`). It leaves to the
checker the restrictions that need more context than the grammar has: `expr.bit.4`,
`expr.prec.2`, `expr.call.4`, `type.struct.8`. `tools/check-grammar.py` accepts every `.ail`
file that compiles, and skips files meant to be rejected (a `.err` file, a `.rules` file with
no `.out`, `tests/mistakes`, `tests/fix`, the fuzz inputs and the tokbench syntax variants).
The docs gate runs it.

## Consequences

- A new syntax decision edits `grammar.ebnf` in the same change, or the docs gate fails.
- `ail explain` goldens move with the added rule `gram.ebnf.1`; the integrator refreezes them.
