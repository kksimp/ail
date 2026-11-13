# 1111. checkbench: `ail check` measured against injected mistakes, with floors in the gate

- **Status:** Accepted
- **Date:** 2026-11-13
- **Decided by:** AI (delegated, ADR 0076), on the designer's ask that `ail check` be "really really good for 0.1"
- **Topic:** AI-first tooling, diagnostics
- **Spec rules:** gram.rec.1, gram.rec.3, gram.foreign.1, lex.str.2, lex.stmt.2, lex.kw.1, expr.name.1, err.prop.1, fix.apply.1

## Context

`ail check` is the tool an agent runs after every edit, and its quality was
known only from hand-picked probes (`tests/mistakes`, the review lane's
`probes.sh`). Those showed cascades (`fn f<'a>` gave three errors), generic help
("declare it first", "docs/language-card.md lists the methods") and fixes that
left the error. A hand-picked corpus measures the mistakes someone thought of;
it can't say how often the checker points at the right line on the mistakes
programs actually get.

## Options considered

### Option A — a mutation harness over the public programs (chosen)
Take every clean program under `corpus/` and `examples/` that has an expected
output, make copies with one typical mistake each (a misspelled local, field,
method, function or type; a dropped `?` or `else`; a deleted match arm or field;
a bad indent or an unclosed bracket; each foreign habit of the review's table),
then `ail check --json`, `ail fix`, check again, build and compare the output.
- **Pros:** measures the properties of the bar (one diagnostic per cause, the
  right line, no generic help, fixes offered and right, nothing silent) on
  hundreds of realistic sites; deterministic (seeded per program and class);
  new classes are a few lines each.
- **Cons:** a mutator is a heuristic over text, so a few sites are not the
  mistake they claim (reported in the table, never a gate failure on their own).

### Option B — grow `tests/mistakes`
- **Pros:** exact expected output per case.
- **Cons:** one program per mistake; says nothing about the rate on real code.

### Option C — measure with an agent writing programs
- **Pros:** the real distribution of mistakes.
- **Cons:** needs a model at gate time (paid, nondeterministic); ruled out for a gate.

## Decision

1. `tools/checkbench/` holds the harness: `mutate.py` (the classes) and
   `checkbench.py` (the driver, the per-class table, `--report` lines for each
   mutant that misses a target, `--json`, `--out` with the worst 20 as files).
   It is Python, like `tools/tokbench` (ADR 0070): a measurement tool that
   drives `./ail` and is never shipped.
2. Per class it reports: errors per mistake (median, share with at most 2),
   first error on the mistaken line or a note naming it, errors with generic or
   missing help, mistakes with a fix, applied fixes that are right, silent
   mutants (check clean, output differs), crashes and hangs, and the check time.
3. A fix is right when the fixed file checks clean and prints the expected
   output. For classes that delete what no fix can know (a match arm, a field's
   value, an argument, a fallback), clean and building is right. A program
   whose unchanged copy does not print its output from a temp directory is
   counted unverified, not wrong. Classes the checker cannot catch in general
   (a missing dedent, `2.0` written `2`) are listed as decided silent classes.
4. The gate suite `checkbench` runs the fast form (3 mutants per class, about
   200, under 2 minutes) and fails when a total falls below its floor in
   `tools/checkbench/floors.txt`. The floors are the measured numbers, so a
   change can only keep or raise them; raise a floor in the change that earns it.
5. The fixes made from its findings (this change): a swap of two neighbouring
   letters is one edit for did-you-mean and a name of 5 or more characters
   allows two; an unknown method, associated function or name lists what does
   exist instead of a generic pointer; one diagnostic, with a fix, for a Rust
   lifetime, for `\{}` placeholders and `format!` (the fix moves the values into
   the braces), for JSON or C code in a string, and for `.iter()`/`.collect()`
   (the chain stays typed); `#[derive]` is reported, not silently a comment; a
   lone unclosed `[` is closed after the index; lex.kw.1 offers no fix that
   would rename the declaration but leave later uses broken; `?` on a value that
   can't fail has a fix; of two syntax reports at one place the fixed one stays.

## Consequences

- `ail check` quality has numbers, per class, that the review lane and the
  soak (criterion 10) read; a regression fails the gate.
- New mistake classes are cheap to add; the coordinator's probe findings become
  classes rather than one-off cases.
- Rejected: an AIL implementation of the harness (no runtime gain, slower to
  change while the classes are being tuned), and output-only judging of fixes
  for deleting classes (would call every placeholder fix wrong).
