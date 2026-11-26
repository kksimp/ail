# 1183. A `Json` value's text is JSON; where to collect results from many tasks

- **Status:** Accepted
- **Date:** 2026-11-26
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer's rule of no syntax change
- **Topic:** 17 — Standard library; 3 — Diagnostics
- **Spec rules:** [json.print.3](../spec/std-json.md), [conc.task.5](../spec/11-concurrency.md) (help text only)

## Context

A hands-on trial of v0.2.0 by a newcomer agent (2026-11-26, nine programs) lost the most to two std points.

- **A silent wrong result.** `v.to_json().to_str()` compiled and wrote `Json.Obj(members: [JsonMember(key: ...` to a
  file: the default text of an enum (trait.auto.2). The JSON text is `to_text()`, which was not on the card and which
  `ail ask std` did not find. The next load failed and the program's fallback hid it. Nothing reported an error.
- **Results from N tasks.** The agent pushed `spawn` handles into a list (conc.task.5), tried `push(move t)`, and
  rebuilt the program with a channel and a `TaskGroup`: 3 build rounds. `[f(x) par for x in xs]` does this in one line,
  and neither the error nor `ail ask how` said so.

## Decision

- **`Json` implements `ToStr` with the text of `to_text`** (json.print.3). `j.to_str()`, `print(j)` and `"{j}"` write
  JSON, as does a `Json` inside a list or another value. `BigInt` and `Date` already give their written form this way.
  `to_text()` stays and is the name the card teaches.
- **No new task type.** The conc.task.5 help for a handle that is pushed, copied or passed writes the `par`
  comprehension; the card's Tasks section and a how-to ("collect results from many tasks") say the same.

This changes what a 0.2.0 program prints for a `Json` value: the variant dump becomes JSON text. No program can have
wanted the dump as data, and `ail fix --migrate` has nothing to rewrite.

## Rejected alternatives

- **A warning on `Json.to_str()`.** The agent in the trial had learned to skip warnings, and `"{j}"` and `print(j)`
  would stay wrong.
- **A certain fix from `to_str()` to `to_text()` on a `Json`.** It repairs one spelling of three and keeps a text form
  nobody wants.
- **A `TaskGroup` that returns results, or task handles in a list.** Handles are unique and waited once on every path
  (ADRs 0095, 0096); a list of them needs a typed group or a change to that rule. That is a design question for the
  designer, and the `par` comprehension covers the measured case.

## Consequences

- Goldens that printed a `Json` change: `tests/cases/json_fields.out`, `tests/cases/else_catches_try.out`,
  `tests/native/golden/recursive_text.out`.
- `compiler/mir/prelude_embed.ail` is regenerated from `tools/native/prelude.ail`.
