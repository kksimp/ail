# 0452. Match guards: `pattern if cond => result`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 3 — Expressions and control flow
- **Spec rules:** [expr.match.11](../spec/05-expressions.md)
- **Builds on:** [ADR 0138](0138-literal-and-or-patterns.md), which listed guards as not designed

## Context

An arm sometimes needs a test the pattern can't express: `Rect(w, h)` only
when `w > h`. Today the author nests an `if` in the arm and repeats the
fall-through result for the other arms. The deciding criteria are **AI
understanding** (one arm, one condition, one result) and **tokens**.

## Options considered

### Option A — `pattern if cond => result` (chosen)
- **Pros:** the condition sits next to the pattern it refines; `if` is already
  the condition keyword; no new token. Tokens on a four-arm `describe`
  function (cl100k / claude-legacy / mistral-spm-v1): 77 / 78 / 91, against
  78 / 73 / 95 for the nested `if` form, a tie by ADR 0076's 2% rule, so the
  structural criterion decides.
- **Cons:** arms can now fall through, so exhaustiveness must ignore them.

### Option B — nested `if` in the arm body (status quo)
- **Cons:** the fall-through arms must be repeated or restructured, and the
  reader can't see at the arm head which cases are partial.

### Option C — `when` or `where` as the guard keyword
- **Cons:** a new keyword for what `if` already says.

## Decision

Option A. The guard is a `bool` expression checked after the pattern matches,
with its bindings in scope (`expr.match.11`). A false guard continues with the
next arm. A guarded arm covers nothing for exhaustiveness (`expr.match.2`) or
redundancy (`expr.match.5`), and `_ if c` doesn't draw the `expr.match.3`
warning, since it hides no variant. A non-`bool` guard reports `expr.if.2`.
The syntax tree gains `Arm.guard`; both parsers and the interpreter and
bytecode tiers implement it.

## Consequences

- There is no binding pattern for a whole scrutinee (`n if n > 0`); the guard
  names the scrutinee's own variable, as in `_ if n > 0`. A binding pattern
  is a later ADR if the agent corpus shows the need.
- `ail fix` suggestions for a missing arm are unchanged.

## Rejected alternatives

- A nested `if` in the arm body, the status quo (option B): rejected because fall-through arms must be repeated or restructured, and the reader cannot see at the arm head which cases are partial.
- `when` or `where` as the guard keyword (option C): rejected because it is a new keyword for what `if` already says.
