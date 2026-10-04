# 0094. Numeric literals: underscores, hex/binary/octal prefixes, and exponents

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 2 — Core syntax
- **Spec rules:** [lex.num.1–3](../spec/02-lexical.md)

## Deciding criterion

Token usage **ties** for common literals. **Human readability** decides:
digit separators and base prefixes are the established readable forms.

## Decision

1. Integer literals: decimal (`1_000_000`), hex (`0xff`), binary (`0b1010`) and
   octal (`0o755`). `_` may separate digits.
2. Floating-point literals have a `.` with digits on both sides (`1.5`, `0.25`)
   and/or an exponent (`1e9`, `2.5e-3`).
3. Literal types follow `type.num.3` (`int`/`float` by default, otherwise
   inferred). A literal that doesn't fit its type is a compile-time error.
