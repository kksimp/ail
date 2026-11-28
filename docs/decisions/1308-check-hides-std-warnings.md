# 1308. `ail check` hides warnings located in standard-module files

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)), from the 2026-11-27 state review
- **Topic:** 22 — Tools
- **Spec rules:** [check.out.5](../spec/22-tools.md), [fix.auto.1](../spec/22-tools.md)
- **Amends:** [ADR 1113](1113-check-output-for-agents.md)

## Context

A four-line program that calls `db.memory()` printed 32 `expr.match.3` warnings from `<std>/db/db.ail`. The author
can't act on them, and they bury the program's own diagnostics. The review also found a certain fix that was wrong:
`.to_owned()` on a map value became `.to_str()`, which changes the value's type.

## Decision

- `check.out.5`: `ail check` prints and counts only warnings in the program's own files. A warning located in a
  standard module's file is neither printed nor counted in the `ok` line or the summary. Errors there are always
  shown, since they stop the build.
- `.to_owned()` on a map value has the fix that drops the call (values are already independent, `own.move` rules);
  the mistakes pair `map_to_owned` checks it.

## Rejected alternatives

- **Fix every std warning instead.** Worth doing, and done module by module, but a new std warning would leak to users
  again; the filter makes the rule hold whatever std holds.
- **A flag to show them.** No use was found for it; it can be added if a std author needs it.
