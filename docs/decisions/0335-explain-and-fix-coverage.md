# 0335. Every emitted rule has an explain entry, and ten more mistakes have fixes

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20). Criterion: AI understanding (an agent repairs a mistake from `ail explain` and `ail fix` alone), then tokens.
- **Topic:** 20 AI semantic API (`ail explain`, `ail fix`)
- **Spec rules:** `explain.cover.1`–`explain.cover.3`, `explain.list.1`, `fix.edit.1`, `fix.edit.6`, `api.pkg.2`, `api.id.1`, `api.doc.1` ([22-tools](../spec/22-tools.md))

## Context

`ail explain RULE` prints a rule from the specification embedded in the binary
([ADR 0146](0146-ail-explain.md)), so an entry exists exactly when a chapter of
`docs/spec` defines the rule. About 200 rules were added in rounds 1–7. A
diagnostic could cite a rule no chapter defined, and a chapter could cite one
that didn't exist. Of the 109 programs in the mistakes corpus (ADR 0262), 34
had no `.fixed` fixture; with this change 86 of 112 do (77%).

## Decision

1. **Coverage tests** (`bootstrap/tests/explain_coverage.rs`, run by
   `./ail selftest`): every rule ID that is a string literal in `bootstrap/src`
   is defined in `docs/spec` (`explain.cover.1`); every rule ID cited in
   `docs/spec` is defined there (`explain.cover.2`).
2. The sweep found four gaps (`api.pkg.2` emitted but undefined; `fix.edit.1`,
   `api.id.1`, `api.doc.1` cited but defined only in design notes or ADR
   tables). The new spec chapter [22-tools](../spec/22-tools.md) defines them.
3. **Ten new fixes**, each with a `.fixed` fixture, from the most common
   Python/Rust/C habits in the corpus: `"a" + b` and `"x" + 1` (`trait.op.1`,
   `trait.op.3`) to `"{a}{b}"`; `"ab" * 3` to `"ab".repeat(3)`;
   `"n is %d" % n` to `"n is {n}"`; `1 in xs` to `xs.contains(1)`; `s[i]` to
   `s.chars()[i]`; `pass` to `todo`; `for (i := a; i < b; i += 1)` to
   `for i in a..b`; `xs: List[int] = []` to `xs := List[int]()`;
   `printf("%d, %.2f", n, x)` to `print("{n}, {x:.2}")`; and `sqrt(x)`,
   `floor`, `ceil`, `round` to the method.
4. Three new anchors (`Operands`, `CFor`, `EmptyDecl`, in `src/diag/rewrite.rs`)
   read the source line, as the call anchors do ([ADR 0264](0264-anchored-call-rewrite-fixes.md)).
   An operand is a name, literal, call, index or `.` chain with nothing binding
   tighter beside it; `a + b + "z"` and `f(x) * 2 + 1` get no fix, so a wrong
   rewrite is never offered (`fix.edit.6`).
5. **`ail explain`** with no rule lists every rule with its chapter key, file
   and whether `ail fix` repairs it; `--json` is one object (`explain.list.1`).
   The list of fixable rules is `src/fixable.txt`; `explain.cover.3` fails when
   it differs from the fixes the test programs produce. Today 39 of the rules
   in the specification have a fix.

## Rationale

AI understanding decides: a rule that can be reported but not explained sends
the agent to read the specification, which is what `explain` exists to avoid.
A test that scans the sources keeps the gap closed without a human audit.

## Rejected alternatives

- **A hand-written explain table** beside the specification. It would drift
  from the rules; the specification is already the one source (ADR 0146).
- **Deriving "has a fix" by reading the sources** for `.fix(` calls. A
  diagnostic's fix often depends on the line, so the static answer is wrong in
  both directions; observing the corpora is exact for what is tested.
- **Fixing `missing_main`, `unclosed_string` and `unreachable_dedent`.** Each
  has more than one plausible repair, so a fix would guess.
