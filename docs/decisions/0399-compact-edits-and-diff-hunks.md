# 0399. Compact `ail edit` hunks by declaration name, `--test`, and changed lines in `ail diff`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20). Criterion: token usage.
- **Topic:** 22 — Tools; AI semantic API
- **Spec rules:** [edit.in.3, edit.id.3, edit.op.7, edit.out.2, edit.out.4](../spec/22-tools.md#structured-edits), [diff.out.7](../spec/22-tools.md#semantic-diff)
- **Amends:** [ADR 0397](0397-ail-edit-structured-edits-by-id.md), [ADR 0396](0396-ail-diff-semantic-diff.md)

## Context

A whole-task measurement (five agent tasks, each with one realistic error,
`/mnt/project-files/whole-task/`) found that the AI-first tools made the
total *higher*: 1,771 tokens before, 1,832 with them. Each fix went through
`ail api --text` (to learn the ids) and a JSON `replace_body`, which resends
the whole function body with every quote and newline escaped. Every one of
the five fixes changes one line or adds two. A review task (t7) cost 5% more
than Python's because `ail diff` said `fn:tax (body changed)` without the
lines, so the reviewer also fetched the textual diff.

## Options considered

### Option A — a compact text form: hunks under a declaration name (chosen)
An id line, then `-`, `+` and context lines, as in a unified diff without
line numbers. The declaration bounds the search, so one changed line is
usually enough; one `-` and one `+` line that match no whole line replace a
unique part of a line (`-words()` / `+split(" ")`).
- **Pros:** no quoting; only the changed text is written; the form agents
  already write for patches; still formatted and checked in the same call.
- **Cons:** a second input syntax next to JSON (JSON stays for the ops that
  carry whole declarations: insert, replace).

### Option B — line-number edits (`ail edit FILE:LINE`)
- **Cons:** numbers go stale after the first edit (rejected in ADR 0397).

### Option C — keep JSON, add a `hunk` op only
- **Cons:** the escaping of quotes and newlines is a large part of the cost.
  `hunk` is still accepted in JSON, for tools.

Details decided with Option A:

- **Bare names** (`edit.id.3`): `average` stands for `fn:average`, so no
  `ail api --text` read is needed to learn ids. A name two kinds share is an
  error asking for the kind.
- **Indentation** (`edit.op.7`): `+` lines move by the difference between
  the found line's indentation and the hunk's own, so a hunk may be written
  at column 0. Matching ignores leading and trailing spaces.
- **Ambiguity is an error**, never a guess: a part that occurs twice asks
  for more of its line.
- **Answer:** `edited 1 file; 0 errors` (`edit.out.2`), with warnings only
  when there are some. The edit count is dropped: the agent wrote it.
- **`--test`** (`edit.out.4`) runs the tests after a clean edit, saving a
  command and the edit summary's round trip.
- **`ail diff` hunks** (`diff.out.7`): each changed body's removed and added
  lines, without the body's indentation, at most 12 lines. Rejected:
  unlimited hunks (a rewritten function would dominate the report), and a
  context line around each change (the `~ ID` line already says where).
- Found on the way: `rename` on a single file without `main` failed on
  `mod.main.1`, which the rest of `ail edit` already ignores; now it does too.

## Measurements

tokbench mean of five tokenizers. Each row is the edit step of one task,
from the first `ail` command after the error to the passing test report.
Before is the measured flow: `ail api --text`, a JSON `replace_body`, the old
summary, then `ail test`. After is one `ail edit FILE --test` with a compact
hunk. The test report is in both.

| Task (fix) | Before | After | Saved |
|---|---|---|---|
| t1 `{}` → `Map[str, float]()` | 119 | 38 | 68% |
| t1 `.to_float() else 0.0` | 172 | 61 | 65% |
| t2 `.to_int()` | 176 | 63 | 64% |
| t3 guard for an empty list | 163 | 64 | 61% |
| t4 `ages.get(n) else 0` | 162 | 60 | 63% |
| t5 `split(" ")` | 139 | 47 | 66% |
| **edit steps, five tasks** | **931** | **333** | **64%** |
| t7 review: `ail diff` + `diff -u` → `ail diff` with hunks | 207 | 111 | 46% |

On the five tasks this takes the AIL total from 1,832 to about 1,234
tokens, against Python's 1,952 (about 37% fewer than Python, from 6%).
All five compact edits produce the reference fixes byte for byte.

## Decision

Option A with the details above.

## Consequences

- The agent flow is: `ail test`, then `ail edit FILE --test` with a hunk.
  JSON remains for whole-declaration ops and for tools.
- `ail diff` reviews usually need no textual diff.
- Windows: the compact form reads standard input as JSON did (ADR 0004).

## Rejected alternatives

- Line-number edits (`ail edit FILE:LINE`): rejected because numbers go stale after the first edit, as ADR 0397 already found.
- Keeping JSON and adding only a `hunk` op: rejected because the escaping of quotes and newlines is a large part of the cost. `hunk` still works in JSON, for tools.
- Unlimited hunks in `ail diff`: rejected because a rewritten function would dominate the report. Hunks are capped at 12 lines.
- A context line around each diff change: rejected because the `~ ID` line already says where the change is.
