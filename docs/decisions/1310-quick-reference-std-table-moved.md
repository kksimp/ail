# 1310. The quick reference's std table moves to `docs/std-modules.md`

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** Agent documentation
- **Amends:** [ADR 0817](0817-token-budgets-for-agent-pages.md)

## Context

`docs/quick-reference.md` has a 12,518-token budget. Main was about 14,800 and the std wave took it to 21,965, almost
all in the table of std modules.

## Decision

- The table moves whole to `docs/std-modules.md` (non-normative), one row per module group: what the signatures do
  not say. The quick reference keeps four lines that name every module by area and link the table.
- `check-doc-dupes` checks the new file's rows. The quick reference is 12,513 tokens.

## Rejected alternatives

- **Trim each row.** The rows hold what `ail card std.NAME` cannot show (effects, traps, `use`); cutting them loses
  that, and the table still grows with each module.
- **Raise the budget.** The page is meant to be read whole.
