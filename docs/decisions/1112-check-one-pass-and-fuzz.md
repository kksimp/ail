# 1112. `ail check` reports everything in one pass, and a checker fuzz guards it

- **Status:** Accepted
- **Date:** 2026-11-13
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked for `ail check` to be really good for 0.1 (package S28)
- **Topic:** 3 — Diagnostics
- **Spec rules:** [gram.rec.4](../spec/03-grammar.md); builds on gram.rec.1-3
- **Builds on:** [ADR 0263](0263-one-diagnostic-per-mistake-recovery.md), [ADR 1077](1077-decision-order-ai-understanding-first.md)

## Context

`ail check` ran the checker only when the file had no syntax error. A file with
one syntax error in one function and four type errors in four others showed one
diagnostic; the agent fixed it, ran `check` again, and met the next four. Each
round costs a model call. The parser already recovers at the statement and at
the next declaration, so the rest of the file is available.

Separately nothing proved that `check` survives hostile input: a trap in the
checker on a half-written file is the worst answer an agent can get.

## Options considered

### Option A — run the checker on the clean declarations (chosen)
- After a syntax error, run the checker over what parsed and keep an error only
  when it lies outside every declaration that holds a syntax error, and does not
  mention a name those declarations declare (the name is missing from the program,
  so its uses would only repeat the error). Warnings are dropped.
- A declaration is the text from one column-1 line to the next.
- **Pros:** five mistakes in five functions are five diagnostics in one run; no
  cascades from a broken declaration; a small, local change (`compiler/load/onepass.ail`).
- **Cons:** an error that depends on a broken declaration in a way the message
  doesn't name could still show; the fuzz measures it.

### Option B — poison types and keep every checker error
- **Pros:** more reports. **Cons:** cascades, which property 2 of the bar forbids.

### Option C — leave it, document "fix syntax first"
- **Cons:** costs a round per syntax error; rejected.

## Decision

Option A (gram.rec.4). A checker-only fuzz, `tools/fuzz/check_fuzz.ail`, runs
mutants of the corpus, examples and cases (truncation, deleted, duplicated and
swapped lines or tokens, inserted keywords and bytes, open brackets and quotes,
bad indents), the extremes (a 5 MB line, 10,000 nested brackets, 100,000 lines)
and generated files of five functions with one mistake each. No hang (10 s), no
trap, exit 0, 1 or 2 with an error behind 2, all five reported and none in `main`.
The fast form is the gate suite `checkfuzz`; every failure it finds becomes a
file in `tests/cases/`. The mutation harness for diagnostic quality is
`tools/checkbench` (package S27), a separate tool.

Amended 2026-11-13 by the integrator (delegated, ADR 0076), from the batch 12
gate: one mistake must stay one diagnostic. The extra checker pass adds nothing
after an import error (`use`, a foreign `import`), drops `mod.main.1` like a
warning, and counts short declared names (`fn f`, `const N`) too; without that,
`fn f() int !` added "unknown name `f`" at every call.

## Amendment 2026-11-13: what the fuzz found

- An unclosed `(` or `[`, or a line ending in an operator, swallowed every
  declaration after it (lex.stmt.2 ran to the end of the file), so one-pass
  could not reach them. A column-1 declaration line now ends the unfinished one
  (gram.rec.5).
- 10,000 nested `(`, `[` or `not` overflowed the parser's native stack. The
  parser stops at 200 levels with `gram.syntax.1`. Rejected: a larger limit
  (no program nests 200 levels) and a rewrite of the descent as a loop (a large
  change for no real program).
- `bootstrap.unsupported` is a documented rule for constructs AIL doesn't have
  (`ail explain bootstrap.unsupported`); the fuzz no longer counts it as an internal error.
- 100,000 lines and 20,000 functions are checked under 60 s, not 10 s: they test
  scale, and the measured time goes in this ADR once the gate has run it.
