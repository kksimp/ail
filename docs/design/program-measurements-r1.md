# Program measurements, round 1: tokens and AI usability

> **Non-normative design evidence,** 2026-10-05. First measurements on whole
> programs that run in the bootstrap (milestone 2), rather than syntax
> fragments.

## Tokens

Two programs were written idiomatically in seven languages, without
comments, and formatted by each language's usual style:
- `gradebook`: structs, an enum, methods, `match`, a loop, and interpolation;
- `inventory`: the same, plus mutating list elements through a function.

All versions except Swift (no compiler available) were run, and their output
was byte-identical to AIL's. The sources are in
`tools/tokbench/corpus/programs/`. Counts are the mean of five tokenizers
(cl100k, o200k, claude-legacy, mistral-tekken, mistral-spm-v1), from
`tools/tokbench/run.sh`.

| Language | gradebook | inventory | Total | AIL vs. this language |
|---|---|---|---|---|
| **AIL** | **214.4** | **284.8** | **499.2** | — |
| Swift | 243.0 | 306.4 | 549.4 | −9% |
| Python (type-hinted) | 270.0 | 325.8 | 595.8 | −16% |
| TypeScript | 277.6 | 334.0 | 611.6 | −18% |
| Go | 285.4 | 354.4 | 639.8 | −22% |
| Rust | 298.2 | 375.6 | 673.8 | −26% |
| C | 379.8 | 429.8 | 809.6 | −38% |

Where the savings come from: no braces or semicolons, signatures without `:`
or `->`, bare variants where the type is known (`category: Food`), `match`
without `case`/`return`, `"{x}"` interpolation, and no conversions such as
Rust's `students.len() as i64` (ADR 0116).

Where AIL loses ground: `inventory` mutates list elements through indexes
(`for i in 0..items.len()` and `items[i].qty` three times), while Python and
Rust iterate over the elements directly. This is now
[OQ-038](../open-questions.md).

The Python, TypeScript and Swift versions use top-level script code, with no
`main`, so the comparison slightly favours them.

## AI usability

**Method.** Fresh AI agents of two sizes (a small and a mid-size model) were
given only [quick-reference.md](../quick-reference.md) and one of four tasks:
- a temperature report;
- a bank account with `mut self` methods;
- a stable bubble sort with a `mut` list parameter;
- FizzBuzz with an enum and `match`.

They saved every attempt before running it, with at most five attempts, and
each attempt was re-run independently afterwards.

| Task | Small model | Mid-size model |
|---|---|---|
| Temperature report | 2 attempts | 1 |
| Bank account | 1 | 1 |
| Stable bubble sort | 1 | 1 |
| FizzBuzz | 1 | 1 |

**Results:**
- 7 of 8 first attempts were accepted and printed exactly the right output,
  and 8 of 8 were correct by the second attempt.
- The single failure was `warmest := readings[i]` inside a loop, which
  redeclares `warmest` instead of assigning it. In Go the same line compiles
  and silently prints the wrong city. AIL rejected it (`expr.bind.4`), and
  the agent fixed it from the diagnostic.

**What the agents found unclear in the quick reference** (none of it caused a
failure):
- that `match` and `if` also work as plain statements, and as a function's
  last expression;
- which contexts allow a bare variant;
- whether `for x in xs` gives a copy of each element;
- whether fields and list elements can be assigned (`xs[i].score = 5`);
- whether `push` and `swap` work on a `mut` parameter.

**Caveats:**
- The tasks are small and use only implemented features.
- The agents are from one model family.
- The quick reference was written by the same AI that designed the
  implementation.
- A broader round (larger programs, more model families, tasks needing error
  handling) should follow each milestone.

## Follow-ups

- Quick reference: the gaps above are now covered.
- Diagnostics: when `:=` redeclares a visible `mut` name, the help will
  suggest assigning with `=` (with milestone 3).
- [OQ-038](../open-questions.md): in-place iteration over list elements.
