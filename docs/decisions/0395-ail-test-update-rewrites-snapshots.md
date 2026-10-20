# 0395. `ail test --update` rewrites failing literal expectations, `# =>` lines and `.out` files

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 22 — Tools; 19 — Testing
- **Spec rules:** [snap.update.1–6](../spec/22-tools.md#snapshot-updating)

## Context

After a deliberate change (a new message format, a renamed field), many
tests fail only because their expected values are stale. An agent then
reads each failure, finds the literal, and writes one edit per literal.
`ail test` shows only the first failing assertion of each test, so a test
with two stale literals takes two rounds. The plan (finish-plan.md §2.4)
asks for snapshot updating that prints what changed.

## Options considered

### Option A — a new `snapshot("name", value)` builtin with files under `snapshots/`
- **Pros:** values of any type; the conventional design in other ecosystems.
- **Cons:** a new language feature and a new directory; expected values move
  out of the test, so a reader (human or agent) opens two files to know what
  a test checks. That costs AI understanding (local reasoning, wishlist 1).

### Option B — rewrite the literals already in the tests (chosen)
- **Pros:** no new syntax; the expectation stays next to the call
  (wishlist 4.3); works on existing tests; the diff is the review.
- **Cons:** only literal expectations can be rewritten (strings, numbers,
  characters, booleans, lists of these). A struct expectation fails as
  before.

### Option C — print the new values and let the agent edit
- **Cons:** keeps the cost this feature exists to remove.

Details decided with Option B:

- **Mechanism:** under `--update`, a failing `assert_eq` with a literal
  expectation records the literal's position and the new text and lets the
  test continue, so every stale literal in a test is found in one run. The
  literal's extent is found by scanning the source from that position.
- **Conflicts:** a literal reached with two different values (a loop, a
  property) is ambiguous; it is left alone and the assertion fails.
  Rejected: last value wins (silently wrong).
- **Strings:** always written as one-line literals with escapes. Rejected:
  keeping `"""` blocks (needs the dedent rules of `lex.str.3` in reverse for
  a rare case).
- **`.out` files:** `NAME.out` next to the file is the repository's own
  snapshot convention (examples and test cases), so `--update` refreshes it
  from `ail run`. `.err` and `.trap` expectations are not touched: they
  record diagnostics, which deserve a reading, not a refresh.
- **`--dry-run`:** as `ail fix --dry-run`, for review before writing.

## Measurements

A representative task (estimated tokens, ADR 0311): a report module with
four tests and eight string expectations; the money and header formats
change.

| Way | Steps | Tokens |
|---|---|---|
| Old: read failures, one edit per literal, re-run | 4 failures shown (of 8 stale literals), 8 edits of ~77 tokens, a second round for the 4 hidden ones | 1,007 for the first round, about 1,400 in all |
| New: `ail test --update report.ail` | one command; reads the diff and the passing report | 560 |

The new way saves about 45–60%, and the output tokens an agent writes
(the edits, the expensive kind) fall from about 600 to about 10.

## Decision

Option B with the details above.

## Consequences

- `ail test --update` writes source files; agents should review the diff it
  prints (or run `--dry-run` first).
- Doc examples' `# =>` lines and `.out` files are refreshed by the same flag.
- Windows: file writes and the `ail run` child use the standard library,
  as on Linux and macOS.
