# TBD. std.regex scans for a literal prefix before the DFA and the Pike VM

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** std.regex speed
- **Spec rules:** [std.regex.27](../spec/std-regex.md)

## Context

`std.regex` already has named groups (`(?P<n>` and `(?<n>`), lazy repetition, a start-set skip over the first ASCII
character and an ASCII DFA for the match span. Patterns that start with a literal of several characters, such as
`error: (\d+)`, still step the DFA from every position that begins with the first character.

## Decision

`compile` records the literal prefix: the `I_CHAR` instructions the program takes before its first branch, jump, class
or assertion (at least 2 characters, case-sensitive, not anchored). `exec` jumps the start to the first occurrence of the
prefix, and none means no match. Results are the same as without it, since every match starts with the prefix.

## Rejected alternatives

- **Possessive quantifiers and atomic groups:** cannot be matched in time proportional to the text with the Pike VM
  (`std.regex.4`); they stay `Unsupported` (`std.regex.21`).
- **More Unicode classes now:** needs generated tables; the `std.unicode` lane owns them. A later change can share them.
- **Boyer-Moore-Horspool for the prefix:** gains little on the short prefixes real patterns have; the plain scan keeps the code small.
