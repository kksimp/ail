# 0440. `std.regex` API: module functions over a `Regex` value; no `re"..."` literal yet

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 17 — Standard library
- **Spec rules:** [std.regex.1–2](../spec/std-regex.md), [std.regex.5](../spec/std-regex.md), [std.regex.9](../spec/std-regex.md), [std.regex.23](../spec/std-regex.md)
- **Criterion that decided:** token usage measured with `tools/tokbench`; the literal and the methods are ties by the 2% rule on representative code and are deferred.

## Context

The brief asks for `regex.compile(pat) Regex ! RegexError`, `is_match`,
`find`, `find_all`, `captures`, `replace`, `split`, and says to decide
whether a `re"..."` literal earns its tokens.

## Options considered

### Option A — module functions taking the `Regex` (chosen for this slice)
`regex.is_match(re, text)`. No change to the checker's method dispatch, so
the slice touches only small, local places in shared files.

### Option B — methods, `re.is_match(text)`
Fewer tokens (see below), but needs a method hook for a standard type like
`tensor.Tensor`. Planned as slice 2 behind the same behaviour.

### Option C — a `re"..."` literal that is compiled at check time
Catches a bad pattern at `ail check` and saves the `regex.compile(r"...") else
return` ceremony. Needs the lexer, parser, checker, formatter and the
AIL front end in `compiler/syntax` to change in the same branch.

## Measurement (tools/tokbench, mean of five tokenizers, one program with
four patterns used in a loop)

| Form | Tokens | vs A |
|---|---|---|
| A: `regex.compile(r"...") else return`, `regex.f(re, x)` | 177.4 | — |
| C: `re"..."` and `regex.f(re, x)` | 155.6 | −12.3% |
| B+C: `re"..."` and `re.f(x)` | 146.8 | −17.3% |

The saving is large in a regex-heavy file (5.5 tokens per pattern, 2.2 per
call) but regexes appear in few lines of the agent corpus programs, so on
representative code the difference is under 2%: a tie by ADR 0076. Both
are deferred to a follow-up slice, not rejected; a bad pattern in a raw
string already fails at `regex.compile` with a rule and a position.

## Decision

Option A. `Regex` is a value with the field `pat str`; `compile` validates
it and the interpreter caches compiled programs by pattern text (so copies
and repeated `compile` calls are cheap). `Match` carries character offsets
and the text, `Captures` the groups and the named groups, and
`RegexError` carries `pattern`, `at`, `kind`, `rule` and `message`, so a
failure can be repaired from the value alone. `escape` is included because
agents otherwise build patterns from user text wrongly.

## Rejected

A `regex.Regex` that hides its pattern: printing and `==` are more useful
with it visible.
