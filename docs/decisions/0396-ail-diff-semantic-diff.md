# 0396. `ail diff`: a semantic diff of declarations, diagnostics and test outcomes

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 22 — Tools; AI semantic API
- **Amended by:** [ADR 0399](0399-compact-edits-and-diff-hunks.md)
- **Spec rules:** [diff.out.1–6](../spec/22-tools.md#semantic-diff)
- **Builds on:** [ADR 0150](0150-ail-api-outline.md) and the ids of `api.id.1`

## Context

An agent reviewing a change (its own, another agent's, a pull request)
reads the textual diff: every changed line with context, and it still has
to work out which signatures changed, whether a function now does I/O or
can fail, and whether any test broke. The plan (finish-plan.md §2.4) asks
for a semantic diff with a short text form and JSON.

## Options considered

### Option A — diff the `ail api` JSON of both versions with a generic JSON diff
- **Pros:** no new code in the compiler.
- **Cons:** reports moved lines as changes, misses bodies, diagnostics and
  tests, and its output is longer than the textual diff.

### Option B — compare by declaration id, plus diagnostics and test runs (chosen)
- **Pros:** reuses the outline (`api.rs`), effects and error inference;
  answers the reviewer's questions directly (what's new, what changed in
  meaning, what broke); deterministic; JSON lines rank under `--budget`
  (a test that now fails carries `"status":"fail"`).
- **Cons:** a renamed declaration shows as removed plus added; a body
  change is reported as "changed", not shown (the textual diff is still
  there when the agent needs the lines).

### Option C — a structural tree diff of the syntax trees
- **Pros:** exact edit scripts.
- **Cons:** much more work, and longer output; reviewers want meaning, not
  tree edits.

Details decided with Option B:

- **Body changes** are detected by hashing the lines after a function's own
  line up to the next declaration (FNV-1a, stable across platforms), so a
  pure signature change isn't also reported as a body change unless the
  contract lines changed.
- **Diagnostics** are compared without line numbers, which shift with every
  edit above them.
- **Tests** run in a child `ail test --json` with a 10-second limit; packages
  are not tested because `ail test` takes a file. `--no-tests` skips them.
- **`--rev REV PATH`** compares against git's copy (git archive into a
  temporary directory, removed afterwards): the common "what did I change?"
  question needs no second checkout. Rejected: `REV:PATH` syntax (clashes
  with Windows drive letters).
- **Exit status** follows diff(1): 0 same, 1 different, 2 error.

## Measurements

Real commits from this repository; the textual diff is `git diff` of the
commit for that path, the semantic one is `ail diff` of the two versions
(estimated tokens, ADR 0311):

| Commit | Path | `git diff` | `ail diff` | Saved |
|---|---|---|---|---|
| 180cd61 (editor: find references) | `tools/edit/edit.ail` | 2,236 | 196 | 91% |
| 34a5a8a (FFI slice 1) | `compiler` | 1,453 | 270 | 81% |
| 8756af0 (OQ-114 to OQ-117) | `compiler` | 2,893 | 308 | 89% |
| 17adc5c (agent data moved) | `examples/poller.ail` | 1,020 | 37 | 96% |

The `ail diff` figures for files include running both versions' tests.

## Decision

Option B with the details above.

## Consequences

- Review loops can start from `ail diff --rev HEAD PATH` and open the
  textual diff only for the declarations it lists.
- Renames are not detected; a later version may pair a removed and an added
  declaration with the same body hash.
- Windows: `--rev` needs `git` and `tar` on the path (both ship with
  current Windows and Git for Windows); the rest is portable (ADR 0004).
