# 0334. Use the fast hasher in the front end and lex keywords without allocating

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Tooling speed (Stage 2), extends [ADR 0175](0175-interpreter-fast-paths.md)
- **Spec rules:** none (no behaviour changes; output is byte-identical)

## Context

ADR 0175 moved the interpreter's tables to `fasthash::FastMap`. The checker's
symbol tables (`types`, `fns`, `methods`, `impl_of`, ...), the module
resolver's and the parser's sets still used SipHash: 8% of `ail check
tools/edit/edit.ail`. The lexer built a `String` for every word and every
symbol, then compared it against all 41 keywords and 37 symbols: about 6%.

## Options considered

### Option A — swap the hasher by import; match keywords on characters (chosen)
`check.rs`, `check/moves.rs`, `modules.rs`, `parser.rs` and `implicit_self.rs`
import `FastMap`/`FastSet` under the old names; `lexer.rs` matches a keyword or
a symbol on the line's characters and allocates only for a name.
- **Pros:** 163M → 139M instructions on the editor check; a handful of
  changed lines per file, so concurrent lanes merge cleanly.
- **Cons:** FNV is weak against chosen keys; the keys are names from the
  program being checked, as in ADR 0175.

### Option B — intern names into integer ids
- **Pros:** cheaper still.
- **Cons:** changes `Tok::Ident`, every AST name and every pass.

## Decision

Option A.

## Rationale

Criterion 4, at no cost to the higher criteria: no behaviour or output change.

## Consequences

- Iteration order of these maps is now fixed per build rather than random per
  run; no output depends on it (the selftest compares output byte for byte).

## Rejected alternatives

- **Interning:** a cross-cutting rewrite of shared files for a few percent more.
