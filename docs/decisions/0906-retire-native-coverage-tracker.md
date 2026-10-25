# 0906. The native coverage tracker is retired

- **Status:** Accepted
- **Date:** 2026-10-25
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 23 — Compiler architecture; 28 — Testing strategy
- **Amends:** [ADR 0583](0583-native-coverage-tracker.md), [ADR 0584](0584-native-coverage-classification.md), [ADR 0746](0746-native-subset-constructs-table.md)

## Context

`tools/native-coverage` ran `ail build` on every program and compared the
result with `ail run`'s output to class each one as built-and-matching,
differing, or outside the native subset. The Rust bootstrap and its interpreter
are deleted ([ADR 0904](0904-rust-bootstrap-deleted.md)), so there is nothing to
compare with, and a refresh of `results.json` is not possible. Every program now
runs natively, and the gate suites check the outputs against frozen goldens.

## Decision

`tools/native-coverage/` is deleted. `docs/design/native-coverage.md` stays as a
frozen snapshot of 2026-10-21. `docs/design/native-subset-constructs.md` stays
the one table of constructs and owners; the `native_misc` gate suite checks it,
and `ail explain native.subset.1` prints it. Only the tracker's owner column is
gone.

## Rejected alternatives

- **Rewrite the tracker against goldens.** The gate suites already fail when a
  program's native output differs, so a second report would be double work
  (ADR 0865).
- **Keep the stale `results.json`.** It lists 757 programs by paths that no
  longer exist.
