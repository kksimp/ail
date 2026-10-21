# 0749. Methods on `regex.Regex`

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20); the integrator approved the change on PR 294
- **Topic:** 17 — Standard library
- **Spec rules:** std.regex.25 ([std-regex](../spec/std-regex.md))

## Context

Lane 192's twin t20 (count the lines that match a pattern) cost 62.6 tokens
against Python's 42.2: every use wrote `regex.` and passed `re` first
(`regex.is_match(re, l)`) while Python writes `pat.search(l)`.

## Decision

`re.is_match(s)`, `re.find(s)`, `re.find_all(s)`, `re.captures(s)`,
`re.replace(s, with)`, `re.replace_first(s, with)` and `re.split(s)` on a
`regex.Regex` mean `regex.NAME(re, ...)`. The module functions stay (and
`group_all`, `escape`, `compile` have no method form: `group_all` is rare and
`escape` and `compile` take no regex). The checker types a method call as the
function with the regex first; the interpreter calls the same code.

## Token measurement

Twin t20 (one test per line): functions 62.6 tokens, methods (`agent-r4/t20b_grep_methods.ail`) **60.6 (-3%)**; Python 42.2. The saving is about 2 tokens per use, so it grows with the number of matches in a program; the rest of the gap is `regex.compile(...)?`. Language card 6,845.8 → 6,853.0 (+7).

## Rejected alternatives

- **Remove the module functions.** They are in every example and card; two
  spellings of seven functions cost a line of documentation, not a rule.
- **A trait `Matcher`.** Only `Regex` would implement it.
- **Methods that take the text as the receiver (`s.matches(re)`).** Text
  would gain seven methods for one type.
