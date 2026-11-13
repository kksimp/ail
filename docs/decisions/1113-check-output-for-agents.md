# 1113. `ail check` output for agents

- **Status:** Accepted
- **Date:** 2026-11-13
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked on 2026-11-13 that `ail check` be "really really good" for 0.1 (package S29)
- **Topic:** 22 — Tools
- **Spec rules:** [check.out.1-3, check.exit.1](../spec/22-tools.md)
- **Builds on:** [ADR 0145](0145-diagnostics-schema-v1.md), [ADR 1047](1047-ail-check-fix.md)

## Context

A clean `ail check` printed nothing, so an agent could not tell "clean" from
"did not run" (a wrong path, a crashed wrapper). Errors named a fix only when
one existed and never said what to run next. `--json` had no written schema and
no end marker. Exit statuses were unwritten.

## Decision

1. A check with no error prints `ok PATH: N functions, M warnings` on stdout.
   Only the launcher's `ail check` asks for it (`--summary`); the check
   `ail run`, `ail build` and `ail test` make first stays silent, so a
   program's output is unchanged.
2. Every error ends `next: ail fix FILE` (a fix settled) or
   `next: ail explain RULE`; in JSON, a `next` member. Only `ail check` adds it,
   so the other commands' diagnostics and their goldens are unchanged.
3. `--json` has a schema, `docs/design/check-json.schema.json`, and a summary
   record `{"api":1,"summary":true,"errors":N,"warnings":M,"fixable":K}` on
   stdout, so the diagnostic lines on stderr keep their count. The gate checks
   every record of the mistakes corpus against the schema's required members.
4. Exit statuses: `check` 0 or 2; `run` of a program that doesn't compile 2,
   as `check`; `test` non-zero. The gate tests them on all three hosts.
   (Amended 2026-11-13 by the integrator, delegated per ADR 0076: the draft
   said 1, as a trap. `ail run` already exited 2 for a check error on every
   host, the `--deny`, `--deterministic`, `--seed` and lock-file gate tests
   all rely on it, and keeping 1 for traps alone lets an agent tell "didn't
   compile" from "crashed" by the status.)
5. `expr.match.3` stays a warning (ADR 0056, 0379). Its help now writes the
   variants to put in place of `_`. Quieting it needs the corpus rate; see
   Follow-up.

## Rejected alternatives

- **Summary on stderr with the records.** Breaks every consumer that counts
  or parses stderr lines.
- **The `ok` line from every `check-program` call.** Pollutes `ail run`
  output; opt-in cannot fail that way.
- **Demote `expr.match.3` now.** Contradicts ADR 0056 and 0379 and needs the
  measured rate first.
- **`next:` only for fixable errors.** The unfixable ones need it most.
- **`run` of a program that doesn't compile exits 1.** It would share the
  status of a trap, so the status alone couldn't tell a compile error from a
  crash, and it breaks the gate's tests that rely on 2.

## Follow-up

Measure warnings per 1,000 lines over `corpus/` and `examples/`
(`ai-review/s29-warn-job.sh`). A warning on more than 5% of programs is
reviewed; a demotion of `expr.match.3` goes to the designer.
