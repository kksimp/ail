# 0647. Diagnostics for round-two mistakes: help texts, anchors and three rule changes

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 20 — AI semantic API
- **Spec rules:** gram.foreign.1, gram.syntax.1, expr.call.1, expr.call.3, expr.match.1, expr.match.4, expr.match.12, type.bool.1, trait.op.3, type.bytes.13, type.struct.6, std.use.2, mod.use.2, mod.use.4 (ADR 0262, ADR 0146)
- **Builds on:** [ADR 0646](0646-mistakes-corpus-round-two.md), [ADR 0465](0465-bit-operations-as-int-methods.md), [ADR 0377](0377-checker-in-ail-ported-pass-for-pass.md) (the AIL checker and parser must agree with the bootstrap)

## Context

Round two's programs (ADR 0646) exposed diagnostics that were silent on
help, pointed at the wrong token, or named the wrong rule. An agent reads
`help` and the anchor, not the message, so each one needs an action it can
take.

## Decision

Help texts and fix anchors, rule unchanged:

1. **Unknown named argument** (`expr.call.3`): the help lists the parameters
   when no name is close (`its parameters are dx, dy`).
2. **`name=value` in a call** (`expr.call.1`): the help says named arguments
   are `name: value`, and the fix writes `:`.
3. **`a @ b`** (`gram.foreign.1`): the help names the method `a.matmul(b)`
   (ADR 0382), and the fix rewrites it.
4. **`x or d` on an optional** (`type.bool.1`): the help is `x else d`, and the
   fix writes `else`.
5. **`n / 4.0` with an `int` variable** (`trait.op.3`): the fix converts the
   integer with `.to_float()`. Literals stay exempt (type.num.10).
6. **`bytes.fromhex`** (`type.bytes.13`): the fix writes `from_hex`, the same
   name with the underscore.
7. **Another library's field** (`type.struct.6`): `status_code` and `text` on
   `net.Response` name `status` and `body`.
8. **`regex.matches`, `regex.findall`, `random.randint`, `random.random`**
   (`std.use.2`): the help and fix name the AIL function (ADR 0648).
9. **`use re`** (`mod.use.4`): a standard module with a near name is named in
   the help; the fallback names the standard modules, since they need no `use`
   (std.use.4).
10. **`mod.use.2` with an unknown item**: the help lists the module's items
    when no name is close. This had no help before.
11. **`n.is_some()` and `n.is_none()` on an optional** (`expr.name.1`): the fix
    writes `!= none` and `== none`.

Rule changes, where the token or the rule was wrong:

12. **`<<` and `>>`** were `gram.syntax.1` ("expected an expression, found
    `<`"). They are `gram.foreign.1` with the fix `a.shl(k)` and `a.shr(k)`,
    the same shape as `^` and `&` (expr.bit.1).
13. **`(a, b)`** (tuples) was `gram.syntax.1`. It is `gram.foreign.1`
    ("AIL has no tuples"; a list `[a, b]` or a struct instead).
14. **`case 1 => ...`** was `expr.match.1` at the `1`. It is at the `case`
    token, and the fix deletes `case`.
15. **Open ranges `5..` and three-dot `0...9`** were `expr.match.4` ("expected a
    pattern"). They are `expr.match.12` (ADR 0262 ranges), and `0...9` is
    fixed to `0..=9`.

The AIL lexer and parser (`compiler/syntax/`) carry items 12, 13, 14 and 15
and the argument-list cases of items 2 and 3, so they agree with the
bootstrap. The AIL checker (`compiler/check/`) agrees on rule and position for
every change here, and the self-hosting gate compares only those two, so the
checker's help texts are left as they are (items 1, 4 to 11).

## Rejected alternatives

- **New rule IDs for the new messages.** Each new ID needs a spec rule, an
  `ail explain` entry, and a row in every corpus. The existing IDs describe
  the same mistakes.
- **A fix for `(a, b)` that changes brackets.** It needs the extent of the
  closing parenthesis, which the tree does not carry at that point. The help
  says what to write, and no fix is attached.
- **Recording the silent `t[0][1]` on a one-dimensional tensor as a passing
  program.** The spec (std.tensor.15) gives `t[i]` a shape with the first axis
  dropped, so `[1]` on it is undefined in the spec. That question is in
  [open-questions.md](../open-questions.md) (OQ-122). The corpus omits the
  program until it is decided.
- **`t / 2` on a float tensor as a diagnostic.** Spec std.tensor.10 and
  type.num.10 say an `int` literal beside a `Tensor[float]` is a `float`. The
  behaviour is right; the program is not a mistake.

## Consequences

The round-two help texts and anchors are visible in `ail check --json`. The
self-hosted parser agrees on the rule and position of every changed case
(`cargo test --test self_hosting`, 4 of 4). Token cost of the help text
changed by this ADR is reported in
[diagnostics-audit.md](../design/diagnostics-audit.md#round-two-adrs-0646-0648-lane-197).
