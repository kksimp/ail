# 0667. `regex.group_all`: the text of one group in every match

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 17 — Standard library
- **Spec rules:** [std.regex.1](../spec/std-regex.md), [std.regex.24](../spec/std-regex.md)
- **Criterion that decided:** Token usage: `regex_codes` drops from 127.0 to 106.2 tokens (-16%), and the gap to Python falls from +35% to +13%. No other criterion differs.
- **Builds on:** [ADR 0440](0440-regex-api-is-module-functions-and-values.md), [ADR 0441](0441-regex-iteration-replace-and-split-rules.md)

## Context

Agent round 4 ([measurements](../design/program-measurements-agent-r4.md),
PR #234) found the largest gap in `regex_codes`: finding `code=NNN` and
reading the digits took `find_all`, `m.text`, `strip_prefix` and an `else`
default, because `Match` holds only the whole match. Python's `re.findall`
returns the group directly.

## Decision

Add `regex.group_all(re, text, group int) List[str]`. It is `find_all`
(std.regex.6) with each match reduced to the text of group `group`, counted
as in `captures` (std.regex.9). A group that took no part gives `""`, as
Python's `findall` does, so the caller needs no `?`. A group the pattern
does not have traps (std.regex.24), as `$name` does in `replace`
(std.regex.7).

## Rejected alternatives

- **`captures_all(re, text) List[Captures]`.** Every element carries
  `start`, `end` and a map that most loops never read. Measured cost is
  higher than the group-only form, and it still needs an index.
- **`Match.group(i) str?`.** `Match` would need its groups as a field. That
  changes the value every `find_all` builds and every `regex.Match(...)`
  literal. The `str?` result also brings back the `else` that the change is
  meant to remove.
- **A `str.byte_len` change or new `bytes.len_of`.** `str.byte_len()` already
  exists (type.str, ADR 0414). `gz_check` now uses it; no new API is needed.
- **Keep `strip_prefix`.** Status quo; it is the +35% gap.

## Measurements (tokbench, `--json`, mean of five tokenizers)

| Program | Before | After | Change |
|---|---|---|---|
| `regex_codes.ail` | 127.0 | 106.2 | -20.8 (-16%) |
| `gz_check.ail` (`big.byte_len()` for `big.to_bytes().len()`) | 110.6 | 108.6 | -2.0 |
| all other eight programs, `.ail` and `.py` | unchanged | unchanged | 0 |

`regex_codes` against its Python twin (94.2): +35% before, +13% after.
`gz_check` against its twin (130.0): -15% before, -16% after.

## Consequences

- `docs/language-card.md`, `docs/quick-reference.md` and `docs/status.md`
  list `group_all`. The card grows by about 12 tokens.
- Still open for the batch: `task_sum` (one handle per spawn, -3%). It is a
  syntax question (`par` over a range), not a library addition, so it is for
  lane 208 and the integrator.
- Bytes-to-str equality (`bytes == str`) would shorten `gz_check` further. It
  changes semantics, so it goes to [open questions](../open-questions.md) as OQ-123, not
  into this ADR.
