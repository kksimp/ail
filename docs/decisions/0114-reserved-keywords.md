# 0114. Reserved keywords

- **Status:** Accepted; amended by [ADR 0120](0120-error-keyword.md) (adds `error`), [ADR 0210](0210-contract-syntax.md) (adds `requires`, `ensures`) and [ADR 1085](1085-keywords-as-member-names.md) (a keyword names a field, a member after `.`, a label and a method)
- **Date:** 2026-10-04
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 2 — Core syntax
- **Spec rules:** [lex.kw.1–2](../spec/02-lexical.md)

## Context

The spec used keywords without listing them. The bootstrap needs the list to
reject `if := 3` and to tell a keyword from a name.

## Deciding criterion

Token usage **ties**: every candidate is one token. **AI understanding**
decides "reserved everywhere, no contextual keywords". A word then always
means the same thing, so a reader never has to work out from context whether
`match` is a keyword or a variable.

## Decision

1. These words are reserved and can't be used as names:

   ```
   and any as break continue defer elif else enum extern false fn for if
   impl in match move mut none not or pub ref return self spawn struct
   trait true unique unsafe use while
   ```

2. `c` isn't reserved. It has its FFI meaning only directly after `extern`
   and `use` (`ffi.decl.1–2`), so `c` stays usable as an ordinary name.

## Consequences

- Adding a keyword later breaks programs that use it as a name. The list will
  be revisited when compile-time execution, type aliases and tests get
  syntax, before the language is published.

## Rejected alternatives

- **Contextual keywords** (C#, Swift): fewer reserved words, but the meaning
  of a word depends on its position, which costs AI understanding.
