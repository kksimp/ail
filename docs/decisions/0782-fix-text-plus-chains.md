# 0782. `ail fix` joins text with interpolation across a `+` chain and around calls with strings

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 8 — Errors; 22 — Tools
- **Spec rules:** trait.op.1, trait.op.3, err.fix.1, fix.edit.2
- **Builds on:** [ADR 0335](0335-explain-and-fix-coverage.md)

## Context

`trait.op.1` (`+` doesn't join strings) and `trait.op.3` (`int + str`) already carried an
"interpolate" fix. It was dropped in two plain cases of the mistakes corpus
(`string_plus_chain`, `hash_crc_plus_text`): the operand finder stopped at the next `+` of a chain,
and it refused any operand with a string inside its brackets (`hash.crc32("abc")`).

## Decision

- `trait.op.1` rewrites the whole `+` chain that starts at the reported operator:
  `a + b + "z"` becomes `"{a}{b}z"`. Only the operators to the right are taken in; the reported `+` is the
  first that fails, so everything to its left is a single operand. `trait.op.3` still rewrites one `+`.
- Operand scanning, both ways, accepts plain strings (no escapes) inside call brackets, and a piece
  may contain quotes if it has no braces or backslash: `hash.crc32("abc") + "x"` becomes
  `"{hash.crc32("abc")}x"`.
- Anything else stays unfixed: a chain with an operand that has braces, an escape or a character
  literal, or one that does not start and end an expression of its own (`1 + 2 + "x"`).

## Rejected alternatives

- **Extend the chain leftwards.** `1 + 2 + "x"` would become `"12x"`, not `"3x"`. The first failing `+` has
  only one operand to its left, so there is nothing to gain.
- **Rewrite with `to_str()` calls.** Longer, and interpolation is what the help already teaches.

## Consequences

Corpus: `string_plus_chain` and `hash_crc_plus_text` are fixed (153 of 250). Test: `fix/plus_chain`.
