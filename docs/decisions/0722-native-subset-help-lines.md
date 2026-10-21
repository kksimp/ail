# 0722. `native.subset.1` problems print a `help:` line

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; lane 229)
- **Topic:** Native backend diagnostics (lane 229)
- **Spec rules:** native.subset.1 (message text only; no rule change)

## Context

`ail build` refuses a program outside the native subset with `native.subset.1`
messages. An agent reading one needs the construct, the rule, and what to write
instead. On `examples/*.ail` the baseline is 457 messages, 35 forms, and no help.

## Decision

1. `compiler/main.ail` prints a `help: TEXT` line after each `native.subset.1`
   error when `compiler/mir/subset_help.ail` has text for the message. The help is
   keyed on the message's wording, so `lower.ail` and its siblings are not edited.
2. The help says the nearest in-subset form when one exists (`join` for
   interpolation, `for i in 0..xs.len()` for `for mut`), and otherwise says the
   construct is outside the native subset and that `ail run` runs it. Where a
   queued PR owns the construct, the help names it (`#180`, `#184`, `#194`,
   `#219`, `#222`).
3. The proposed changes to the messages themselves go to
   [docs/design/native-subset-messages.md](../design/native-subset-messages.md), to
   be applied in `lower.ail` when its queue clears.

## Measurement

| | `native.subset.1` lines on `examples/*.ail` | with a `help:` line | cl100k tokens per diagnostic |
|---|---:|---:|---:|
| Before | 457 | 0 | 32.2 (message only) |
| After | 457 | 416 (91%) | 37.9 (message and help) |

The help adds about 6 tokens per diagnostic (+18%). The number of diagnostics does
not change: the help fixes no construct, it says what to write instead.

## Rejected alternatives

- **Editing `lower.ail` messages now.** Its queue is long; a PR at the end would
  not land tonight.
- **Adding the help in Rust's `ail explain` data.** The help must print where the
  problem prints, and that is `compiler/`. The brief says port nothing to Rust.
- **Generating the `ail explain native.subset.1` list from the blockers table in
  this change.** Not done; the blockers table has no owner column yet, so the
  owners in the help text are set by hand from the night-run merge order. See the
  open item in the PR.
