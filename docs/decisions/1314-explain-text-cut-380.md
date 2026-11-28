# 1314. `ail explain` cuts a rule's text at 380 characters

- **Status:** Accepted
- **Date:** 2026-11-28
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 20 AI semantic API (`ail explain`)
- **Spec rules:** [explain.cover.4](../spec/22-tools.md)
- **Amends:** [ADR 0336](0336-explain-listing-and-length.md)

## Context

`explain.cover.4` keeps the mean entry under 400 characters (about 120 tokens). The std wave added several hundred
rules whose text runs past the cut, each entry about 600 characters with its example, and the mean reached 400.9
with the sqlite writer. Each chain since has had to shorten examples to stay under.

## Decision

The text form cuts a rule's text after its last whole sentence within 380 characters (was 400), with the same
pointer to the line above. The mean is 393. `ail explain --json` and the spec are unchanged.

## Rejected alternatives

- **Raise the mean to 420.** The budget is what keeps an entry one read; the long tail is std rules, which the cut
  already shortens.
- **Cut examples to three lines.** The example is what turns a rule into a repair (ADR 0336); the rule text is
  one `--json` or `Read` away.
- **Shorten the spec text.** Rejected in ADR 0336: the spec is read whole by people.
