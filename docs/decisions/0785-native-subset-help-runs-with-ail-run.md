# 0785. Every `native.subset.1` help line says `ail run` runs the program

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076), on the coordinator's direction
- **Topic:** Native backend diagnostics
- **Spec rules:** native.subset.1 (message text only; no rule change)

## Context

[ADR 0722](0722-native-subset-help-lines.md) added a `help:` line to a
`native.subset.1` problem when `compiler/mir/subset_help.ail` has an entry for the
message. Auditing the 149 message forms in `compiler/mir/lower*.ail` found that
123 had no entry and so no help, and 16 of the 26 that had one did not say that
`ail run` still runs the program. Every message names the construct or form and
the position, which `compiler/main.ail` prints as `FILE:LINE:COL`.

## Decision

`subset_help` always returns a line for a `native.subset.1` problem (except an
internal error, which is not a subset problem):

- an entry that already names `ail run` is unchanged;
- an entry that doesn't gets `; or `ail run`` appended;
- a message with no entry gets `` `ail run` runs this program``.

The messages in `lower*.ail` are unchanged (one line each). The mapping stays in
`subset_help.ail`, so no `lower.ail` slot is needed.

## Measurement (cl100k, the 149 forms)

| | forms with a help line | forms saying `ail run` | help tokens, all forms |
|---|---:|---:|---:|
| Before | 26 | 10 | 639 |
| After | 147 | 147 | 1,760 |

The suffix costs about 6 tokens on the 16 changed entries; the generic line costs
9 tokens including `help:` on 114 forms. A real run costs the generic line once per
unlisted problem.

## Rejected alternatives

- **One trailing note per build.** The line would sit away from the problem it
  explains and agents read one problem at a time.
- **A longer generic line naming the owner PR.** Without an owner map for the 114
  forms it would be a guess.
- **Rewriting the vague messages (`this call`, `this expression`, `this print`).**
  They live in `lower.ail` and its siblings, outside this change; the position
  already points at the construct. Listed in
  [native-subset-messages.md](../design/native-subset-messages.md).
