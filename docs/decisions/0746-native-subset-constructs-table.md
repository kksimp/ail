# 0746. One table of native-subset constructs, with owners

- Status: Accepted
- Date: 2026-10-21
- Decided by: AI (delegated, ADR 0076)

## Context

The native backend refuses constructs it does not take yet (`native.subset.1`).
Each refusal names the construct, but not who is building it, and the owner
lists lived in lane briefs, in `docs/design/self-compile-status.md` and in a
regex list inside `tools/native-coverage/coverage.py`. Three copies drift.

## Decision

`docs/design/native-subset-constructs.md` is the one table: Construct | Wording |
Owner. Its rows are data for two readers:

- `ail explain native.subset.1` prints each construct with its owner
  (`construct: owner`). The wording is not printed, to stay under the
  400-token budget for the whole explain entry (now 392 cl100k tokens).
- `tools/native-coverage/coverage.py` names a blocker's owner by the first row
  whose wording occurs in the diagnostic. Unmatched blockers are `unowned`.

`bootstrap/tests/native_subset_table.rs` checks that every owner is `-`, `#N`
or `lane N` (comma-separated), and that every wording occurs in `compiler/`
or in `docs/design/native-coverage.md` with backticks removed. A row whose
construct is gone from the backend fails the test, so the table cannot list
phantom gaps.

## Rejected alternatives

- Keep the regex list in `coverage.py`. Two lists would drift, and the explain
  output could not show owners.
- Put the table in a `.toml` file. The explain output is text, and Markdown
  already renders for the designer. The table is loaded with `include_str!`,
  so the binary needs no runtime file.
- Print the wording in explain. It doubles the entry's tokens for a reader who
  needs only the owner; the wording stays for the coverage tool and the test.
- Owner column as free text. Owners must be checkable, so only `#N`, `lane N`
  or `-` are accepted.

## Consequences

Adding a construct means adding one row. A row whose owner lands is updated in
the same change, and the test keeps the wording honest.
