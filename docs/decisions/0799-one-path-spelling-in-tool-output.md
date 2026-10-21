# 0799. One path spelling in every tool output

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 17 — Platforms (Windows)
- **Spec rules:** none new; it refines [std.path.4](../spec/17-std.md) for tool output only
- **Criterion that decided:** Understanding (criterion 2). An agent reading a diagnostic, a JSON report or an `ail api` answer should see one path form whatever the host, so its parsing and its test expectations do not depend on the OS.

## Context

Under Wine, the bootstrap's own test suite failed on 15 test targets. Most failures came from paths: diagnostics, JSON reports and the case runner printed `tests\cases\x.ail` where the expected text, written on Linux, uses `tests/cases/x.ail`. ADR 0504 keeps `std.path.join` on `/` and says Windows accepts both separators, so the question was only what tools print.

## Options considered

### Option A — print the native separator
- **Pros:** the output looks native to a Windows user.
- **Cons:** every expected output and every JSON consumer needs a per-OS form, and an agent sees two spellings of the same path.

### Option B — print `/` in every tool output; convert only at the OS boundary (chosen)
- **Pros:** one spelling for diagnostics, JSON and `ail api`; expected outputs stay as they are; Windows APIs still receive `\` where they need it.
- **Cons:** a Windows user reading a diagnostic sees `/`. Windows accepts it, so nothing breaks.

### Option C — normalise only in the test harness
- **Pros:** no product change.
- **Cons:** agents still see two spellings of a path, and the harness hides a real difference.

## Decision

Option B. Diagnostics, JSON reports and `ail api` output always use `/` as the separator. The OS form (`\`) appears only where the program calls the operating system. Test expectations stay written with `/`, and the case runner normalises actual output to `/` on Windows before comparing.

## Consequences

- Output of the interpreter, the checker and the `api` tools is identical on Linux and Windows for the same tree.
- Only the code that calls the OS converts paths. This is a small set of helpers in `bootstrap/src/`.
- The checklist in [windows-status](../design/windows-status.md) needs no new step for this rule.

## Rejected alternatives

- Option A and Option C, above.
