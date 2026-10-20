# 0203. `ail fix` applies diagnostics' fixes until none are left

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** AI semantic API
- **Spec rules:** `fix.apply.1`–`fix.apply.4` (non-normative, [semantic-api.md](../design/semantic-api.md))

## Context

Diagnostics carry fixes but nothing applies them, so every agent re-implements edit application (positions in characters, exclusive ends, last to first). Criteria: tokens (#1), AI understanding (#2).

## Decision

`ail fix FILE [--dry-run] [--json]`:

1. **fix.apply.1** — Check the file; take the first fix of every diagnostic that has one; skip a fix whose edits touch an earlier-chosen fix's range; apply the rest from last to first.
2. **fix.apply.2** — Re-check and repeat, up to 5 rounds, until no fixable diagnostic remains.
3. **fix.apply.3** — If the file was already in canonical layout ([ADR 0169](0169-canonical-formatting-style.md)) before, format the result.
4. **fix.apply.4** — Write the file, or with `--dry-run` print a unified diff and leave it. Print each applied fix (`--json`: one `{"api":1,"applied":{...}}` object each, then a `{"api":1,"summary":true,...}` object). Remaining diagnostics go to stderr in the usual form. Exit 0 if the file has no errors afterwards (warnings, such as holes, don't count), 1 if it has, 2 for a usage or read error.

## Rationale

Re-checking between rounds makes overlapping and dependent fixes safe: a fix is never applied on top of a stale position. Five rounds is enough for chains (`else if` hides later errors; arms add holes) and bounds a loop.

## Consequences

- One call replaces a read-edit-check loop. `ail fix` never changes a file with no fixable diagnostic.
- A fix that makes things worse is possible in principle; the rounds cap and the final check report it.

## Rejected alternatives

- **Apply all fixes of a diagnostic.** Fixes are alternatives, "best first" ([ADR 0145](0145-diagnostics-schema-v1.md)).
- **Keep going until a fixed point.** An unbounded loop on a misbehaving fix; 5 rounds is a fixed budget.
- **Always format.** Would rewrite a file the author kept in another layout; `ail fmt` is the explicit command.
