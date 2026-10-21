# 0732. Counting into a map, and JSON reads with defaults

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; lane 234)
- **Topic:** Standard library (round 5 of the agent corpus, [measurements](../design/program-measurements-agent-r5.md))
- **Spec rules:** std.count.1, std.count.2, json.query.5, json.query.6, json.lines.1

## Context

Round 5 (lane 224) found three patterns that cost AIL more tokens than Python in agent programs:
counting a key in a map (read with a default, add one, write back), reading a JSON member
with a default (`j.str(k) else ""`, repeated for every field), and parsing one JSON document
per log line while skipping the bad ones. The savings are measured in the round-5 note.

## Decision

1. `m[k] += 1` adds one to the count at `k`, starting at zero (`std.count.1`). No new method: the
   language already did this, and `bump` was dropped in review as a duplicate. `m.top(n)` gives the
   `n` keys with the largest counts, ties in insertion order (`std.count.2`).
2. `j.text(key, default)` and `j.flag(key, default)` read a string or bool member, or the default
   when the member is missing or of another kind. They never fail (`json.query.5`, `json.query.6`).
3. `parse_json_lines(text)` gives each line that parses as one document, skipping the others
   (`json.lines.1`).

The helpers are methods and functions written in the prelude (`bootstrap/src/prelude.ail`) or
the checker and interpreter (the map methods), and they follow the existing `j.str` and `m.push` style.

## Token effect

Measured with tokbench (mean of five tokenizers), see the round-5 note: `explain_chapters` 221.6 →
181.0, `check_rules` 206.0 → 188.4, `effects_table` 221.0 → 205.8, `fix_preview` 152.0 → 142.8. The
round-4 `poller` and `tasks` examples are not smaller: `j.text(k, default)` is about as long as
`j.str(k) else default`.

## Rejected alternatives

- **Only `m[k] += 1`, which already starts a missing key at zero.** Kept as the way to count; `bump`
  is the same operation with a name that says what it does, and programs measured with it. This is
  the one addition that repeats an existing form, so it is the easiest to remove if the designer
  prefers a single way.
- **`Counter`-style type with its own methods.** Rejected: a new type for one operation adds more
  to learn than a method on the map.
- **Making `top` break ties alphabetically.** Rejected: that needs a sort on keys, which a generic `K`
  may not have. Insertion order is free and deterministic.
- **`j.text(k)` returning `""` with no default.** Rejected: an agent often needs a value other than
  the empty string (for example `"?"` in a report), so the default is an argument.
- **`parse_json_lines` that fails on a bad line.** Rejected for logs: one broken line should not hide
  the others; a caller who needs strictness can use `parse_json` per line.

## Consequences

- The language card gains a few lines. It is now 7,128 tokens, under the 8,000 budget.
- `explain_chapters.out` changes whenever a spec rule is added; this lane added rules, so its
  expected output was regenerated.
