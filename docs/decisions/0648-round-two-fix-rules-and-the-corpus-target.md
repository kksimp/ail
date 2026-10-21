# 0648. Fix rules for round-two mistakes, and the 60% corpus target

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 20 — AI semantic API
- **Spec rules:** gram.foreign.1, expr.call.1, expr.call.3, type.bool.1, trait.op.3, type.bytes.13, type.struct.6, std.use.2, expr.name.1 (ADR 0262, ADR 0202, ADR 0264)
- **Builds on:** [ADR 0646](0646-mistakes-corpus-round-two.md), [ADR 0647](0647-round-two-diagnostics-help-and-anchors.md), [ADR 0454](0454-named-arguments.md) (named arguments)

## Context

The harness requires `ail fix` to resolve at least 60% of the corpus
(ADR 0262). With the round-two programs added the corpus had 214 programs,
of which `ail fix` resolved 103 (48%). Most round-two programs had a
diagnostic with no fix attached, so the target needed fixes for them, not
more programs.

## Decision

A fix is attached only when the answer is fixed by the rule of the
mistake. A rename qualifies when the arguments carry over unchanged. A fix
does not guess a value, a type, or an error-handling choice.

New fixes (rules in `src/fixable.txt`, now 48):

1. `f(dx=1)` writes `dx: 1` (`expr.call.1`).
2. `a @ b` writes `a.matmul(b)` (`gram.foreign.1`).
3. `x or d` on an optional writes `x else d` (`type.bool.1`).
4. `case 1 => ...` deletes `case` (`expr.match.1`).
5. `0...9` writes `0..=9` (`expr.match.12`).
6. `bytes.fromhex` writes `bytes.from_hex` (`type.bytes.13`).
7. `bytes(n)` writes `bytes.zeros(n)` (`expr.name.1`, the call table of
   `foreign.rs`).
8. `x.is_some()` writes `x != none` and `x.is_none()` writes `x == none`
   (`expr.name.1`).
9. `n / 4.0` with an `int` `n` writes `n.to_float() / 4.0` (`trait.op.3`).
10. `status_code` writes `status` and `text` writes `body` on `net.Response`
    (`type.struct.6`).
11. `regex.matches` writes `regex.is_match`, `regex.findall` writes
    `regex.find_all`, `random.randint` writes `random.int` and `random.random`
    writes `random.float` (`std.use.2`). Each keeps its argument order.
12. `s.encode()` writes `s.to_bytes()`, and `b.startswith(x)` and
    `b.endswith(x)` write `starts_with` and `ends_with` on `bytes`
    (`expr.name.1`, the method table in `check/builtins.rs`).

The fixed programs in round two have a `.fixed` beside each `.ail`. The
earlier fixes (ADRs 0202 and 0264) and the corpus harness are unchanged.

## Rejected alternatives

- **`else return` for an unhandled error (`err.use.1`).** It silently drops
  the error. A reviewer would choose `?`, `else` with a default, or a
  declared `! E`, so no one choice is the fix.
- **Brackets for a tuple `(2, 3)` to `[2, 3]` (`gram.foreign.1`).** It needs
  the closing parenthesis, which the anchor does not carry. The help says
  the list form instead.
- **`regex.sub` to `regex.replace`.** Python's `re.sub(pattern, repl, s)` and
  AIL's `regex.replace(re, s, rep)` take their arguments in different orders,
  so the rename would not fix a Python call. No fix; the help lists the
  functions.
- **`t.result()` to `t.wait()?`.** The `?` needs a fallible function, which
  the fix cannot add.
- **Default `float` for `int` in `fn f() int` returning a float.** That
  changes a declared type, which the designer owns.

## Measurement

| | Before round two | After round two (this ADR) |
|---|---|---|
| Corpus programs | 154 | 250 |
| Programs `ail fix` resolves | 96 (62.3%) | 150 (60.0%) |
| Rules in `fixable.txt` | 41 | 48 |

The target is met: `fixed * 100 >= files * 60` at 150 of 250 (the harness's
`the_corpus_meets_its_targets`). The margin is zero, so any new program that
has no fix will fail the harness until it gets one or the corpus gains a
fixed program.

## Consequences

`ail explain` reports the new fixable rules. `src/fixable.txt` is the list
the `rules_with_a_fix_match_the_corpora` test compares against the corpus,
so a fix added without its entry fails the test.
