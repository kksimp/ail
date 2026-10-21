# 0724. Trim the quick reference by at least 15% without losing a construct

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; lane 230)
- **Topic:** Docs (lane 230)
- **Spec rules:** none (non-normative page)

## Context

`docs/quick-reference.md` is the page an agent loads to write AIL. It grew with
every lane. At baseline it was 15,176 tokens (tokbench mean; cl100k 14,392).

## Decision

Cut the page's tokens by at least 15%, keeping every construct and every
runnable example. Three kinds of change:

1. Remove duplicates: an older `fix` line, an older `std.fs` row (the later row
   has `read_bytes` and `write_bytes`), and two older `std.tensor` rows (the last
   row has `/`, indexing and every method the others list).
2. Shorten prose to `construct: example` form, and shorten table cells without
   dropping a function name. The CLI block is an `sh` fence, so its comments are
   shortened and every command and flag kept.
3. Fix a block that did not run: the `unique` example had top-level statements,
   so it now sits in `fn main()`.

Code blocks are otherwise unchanged (33 `ail` blocks identical to the previous
version, apart from that one).

## Measurements

| | tokbench mean | cl100k |
|---|---:|---:|
| Before | 15,176 | 14,392 |
| After | 12,891 | 12,202 |
| Change | -15.1% | -15.2% |

Removed text, for the PR body: the old `fix` line; the old `std.fs` row; two old
`std.tensor` rows; the note that `ail test --record` keeps failed tests; the
`JsonMember` type name (its fields are still described); the note that fused
tensor chains run in one pass. Every other flag, function and construct is kept.

## Checks

- Every `ail` block that is not `skip=context` or `norun`: `ail check` passes.
  Declaration-only blocks report only `mod.main.1`, as libraries do. The doc-snippets
  harness the brief names (#230) is not in this repository, so this is a manual
  equivalent, not that harness.
- Three whole-task programs written from the page (JSON derive, word counts with
  a sorted key, error handling with `?` and `else err`) pass `ail check` and print
  the expected output.

## Rejected alternatives

- **Moving reference rows to a separate page.** Agents would need a second load
  for every std call, which costs more than the rows.
- **Dropping the `ail api` CLI lines.** They are how an agent finds the rest of
  the tooling; only their comments were shortened.
