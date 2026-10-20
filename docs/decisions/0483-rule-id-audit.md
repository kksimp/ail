# 0483. A script audits spec rule IDs against the code, and the fix rules are normative

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** Spec and diagnostics (ADR 0002)
- **Spec rules:** `api.out.1`–`api.type.2` and the others in [chapter 22](../spec/22-tools.md#semantic-api) (moved here from [semantic-api.md](../design/semantic-api.md)); `fix.edit.2`–`fix.edit.4`, `fix.apply.1`–`fix.apply.4`

## Context

ADR 0002 requires every behaviour and diagnostic to cite a spec rule. Nothing checked that. A lane audit found 42 IDs cited in code that the spec did not define (the `api.*` rules were defined only in the design note, and the `fix.*` rules were marked non-normative in ADR 0203 and 0204), and several citations were ranges wider than the rules that exist (`expr.if.1-8` with no `expr.if.7`).

## Decision

1. `tools/rules-audit.sh` reports four things: cited IDs the spec doesn't define; spec rules no code cites; diagnostic IDs with no `ail explain` entry; `ail explain` entries with no example. It exits 1 on any finding. It is not in `./ail selftest` yet: two of the four checks are not clean, and the script joins the selftest when they are.
2. The `api.*` rules move from the design note into chapter 22, verbatim, so `ail explain` covers them. The design note keeps the output formats and points to the chapter.
3. The `fix.edit.2`–`4` and `fix.apply.1`–`4` rules are normative. They state the behaviour of accepted ADRs and the implemented tool, so the code can cite them.
4. Code citations that were wider than the spec are narrowed to the rules that exist (`api.loc.1-3`→`1-2`, `api.ast.1-6`→`1-3`, `err.msg.1-3`→`1-2`, `type.num.13-16`→`13-15`, `test.doc.1-4`→`1-3`, `expr.if.1-8`→`expr.if.1-6, expr.if.8`). No rule is renumbered.

## Rationale

Tokens and local reasoning (#1, #2): an agent that reads `ail explain` for a cited ID gets the rule text. Each cited ID now has text, and a wrong range fails the audit instead of misleading a reader.

## Consequences

- The audit's cited-but-undefined and diagnostics-without-explain checks are clean. Spec rules that no code cites (66) and rules without an explain example (274) are open; the examples are conformance tests (ADR 0002), so each needs a checked `ail` block, and they are filed for the owning lanes.
- The `fix.*` rules are now normative. A later change to `ail fix` must update them.

## Rejected alternatives

- **Leave the API rules in the design note.** `ail explain` could not explain them, and an agent would have to search `docs/design/`.
- **Delete the citations of undefined rules.** That hides behaviour the code implements and loses the link from code to its rule.
- **Write examples for all 274 rules in this change.** Each example is a conformance test; that is a separate, larger lane.
