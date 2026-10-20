# 0464. A JSON number outside the float range is a parse error, and a non-finite number prints as null

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** Standard library — std.json
- **Spec rules:** [json.parse.2](../spec/std-json.md), [json.print.2](../spec/std-json.md)

## Context

The module fuzzer (seeds 6 and 15, `fuzz.mod.3`) found that `parse_json("1e999")`
succeeds with an infinite number and `to_text` then prints `inf`, which is not JSON
and fails to parse again. `json.parse.1` promises one RFC 8259 document; the text
the module prints must be one too.

## Options considered

### Option A — reject out-of-range numbers when parsing, print non-finite as `null` (chosen)
- **Pros:** a number read from text is always finite; `to_text` output always parses;
  two lines in the prelude; matches what most JSON writers do for a value that
  JSON can't represent.
- **Cons:** `1e999` that parsed before now fails (nobody relied on it; no case did).

### Option B — keep infinity, print it as `1e999`
- **Pros:** lossless.
- **Cons:** a value that is not a float comes back as one; `Num(NaN)` still needs an answer.

### Option C — parse failure only
- **Cons:** a program can still build `Json.Num(inf)`, and its text would not parse.

## Decision

Option A. Test: `bootstrap/tests/cases/json_number_range.ail`. Criterion **AI
understanding**: a round trip that always holds needs no caveat in the docs.
