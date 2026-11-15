# 1136. A broken statement or a failed import hides only what depends on it

- **Status:** Accepted
- **Date:** 2026-11-15
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer's 0.2 goal of fewer agent iterations than C (diagnostics only, no language change)
- **Topic:** 3 — Diagnostics
- **Spec rules:** [gram.rec.4, gram.rec.6, gram.rec.7](../spec/03-grammar.md)
- **Builds on:** [ADR 1112](1112-check-one-pass-and-fuzz.md), [ADR 1077](1077-decision-order-ai-understanding-first.md); works with ADR 1130 (certain fixes applied by `ail check`, in review)

## Context

ADR 1112 runs the checker after a syntax error and keeps its errors from the
declarations that parsed cleanly. Two holes stayed. In the E13 agent benchmark
27 of the 87 failed compiles that followed a failed compile (12% of all 227)
were errors already in the program that the first run had not shown:

- A syntax error anywhere in a function dropped every checker error in that
  function, although the parser had already skipped only the one statement
  (`n = args[1].to_int() else` on line 65 of `main` hid `(0..n).to_list()` on
  line 115 of `main`).
- Any loader error dropped more than its own: an error on a `use` line dropped
  every checker error of the program, and an item error (a private `Store`)
  dropped the checker's errors in its function, the private fields among them.
  `use src.event` from the package root (mod.unit.2) left `event` unloaded, so
  nothing that used it was checked until the agent fixed the import.

## Options considered

### Option A — narrow what each error hides (chosen)
- A syntax error in a function body whose header parsed hides the checker's
  errors on its statement's lines (the lines it continues over and its indented
  block), plus those in the same function that name a name the statement binds
  or judge the whole body (`expr.block.2`, `conc.task.5`, `own.move.4`,
  `own.defer.3`, `type.never.3`). Errors in the block structure, a string or a
  bracket left open across lines still break the whole declaration.
- The loader's errors are told from the parser's (`Loaded.parsed`). A private
  item, a late or a second import leave the program whole: only the checker's
  errors on the same line go. An import that names no module makes its names
  unknown: the declarations that use them are left out, and, transitively, the
  ones that use a declaration whose signature names them.
- `use src.store` from the root's own directory loads `store` and binds the
  import to it, beside the `mod.use.4` error and its fix.
- **Pros:** the E13 cases are reported in one run; no syntax change; the
  change stays in `compiler/load/` (onepass, load, resolve, expand).
- **Cons:** the filters are textual (lines, words); an error that depends on a
  broken statement in a way the message doesn't name can still show. The
  checker fuzz and `checkbench` floors measure it.

### Option B — poison types in the checker (an "error" type that absorbs uses)
- **Pros:** precise. **Cons:** touches every checker rule; a large change for
  0.2; rejected for now, open if the textual filters leak in `checkbench`.

### Option C — leave the parser's recovery as the only one
- **Cons:** keeps the 12% of failed compiles; rejected.

### Option D — new error-recovery in the parser
- The parser already recovers per statement (`skip_statement`) and per
  declaration (gram.rec.5); the loss was in what the loader showed. Rejected as
  unnecessary.

## Decision

Option A. With ADR 1130 the two compose: the first run reports every error and
applies the certain fixes, so the next run starts from fewer.

## Consequences

- `ail check`, `ail build`, `ail run` and `ail test` report more errors per run;
  goldens of packages with import errors list the checker's errors too.
- Derived impls are still expanded only for a program with no errors, so a
  derive-related checker error beside a loader error can show (as before).
