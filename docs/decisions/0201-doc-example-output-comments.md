# 0201. Assert example output with `# => text` comments

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Testing, [ADR 0167](0167-test-functions-and-ail-test.md)
- **Spec rules:** `test.doc.2`

## Context

A doc example is most useful when it shows the result. Asserting it with `assert_eq` hides the result in code; a comment next to the `print` shows it where a reader looks.

## Options considered

### Option A — `# => text` after a `print` (chosen)
- **Pros:** familiar from other languages' doc tests; costs nothing at run time; the comment is a plain `#` comment, so `ail fmt` and `ail run` ignore it.
- **Cons:** the whole output is compared, not each `print` separately.

### Option B — only `assert_eq`
- **Pros:** no new convention.
- **Cons:** the expected value reads as code, not as output.

## Decision

If an example has any `# => text` line, the lines after `# =>` (one leading space dropped, trailing spaces ignored), in order, must equal the lines the example prints. An example with no such line only has to pass.

## Rationale

ADR 0076 criterion 3 (human readability), with no token cost for examples that don't use it.

## Consequences

- An example that prints more than it annotates fails; annotate every `print`.

## Rejected alternatives

- Option B: kept as a valid alternative; both can be used in one example.
