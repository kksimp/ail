# 1179. Checker rough edges: optional and plain branches agree, `_` repeats, certain fixes for five habits

- **Status:** Accepted
- **Date:** 2026-11-25
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); no syntax change. The one rule that accepts more programs (type.opt.11) is flagged to the designer in the lane's report
- **Topic:** 2 — Type system
- **Spec rules:** [type.opt.11](../spec/04-types.md), [expr.if.4](../spec/05-expressions.md), [expr.bind.4](../spec/05-expressions.md), [type.coll.13](../spec/04-types.md)
- **Amends:** [ADR 1169](1169-final-run-first-draft-misses.md) (its expr.if.4 help), [ADR 1168](1168-open-collection-later-use-fix.md) (first uses that fix a `[]`)

## Context

104 probe programs of code an agent would write as a first draft were run through `ail check` and `ail run` on main
(9ca7f957, `rough-edges-probes-main.txt` in the project library). The ones below failed with no fix applied, or with
help that pointed the wrong way:

- `x := if c then xs[0] else xs.first()` was expr.if.4, and its help (ADR 1169) said to write `x T? := if ...`. AIL has
  no typed local, so the help could not be followed, and there is no expression that turns a `T` into a `T?`. A bare
  `none` branch already gives `T?` (type.opt.10).
- `for _ in 0..n` inside `for _ in 0..m`, and `[[0 for _ in 0..w] for _ in 0..h]`, were expr.bind.4: `_` was declared.
- `for c in s` on a `str`, `Node(val: 1)` with the optional fields left out, `c - 'a'`, `total + b[0]` with a `u8`,
  and `xs.extend(ys)` as the first use of `mut xs := []` each had one meaning and no certain fix. `a + 1` on an
  `int?` and `c + 1` on a `char` printed the help for mixing `int` and `float`.

## Decision

- **type.opt.11:** an `if` or `match` with no expected type whose branches are `T` and `T?` has the type `T?`; the
  plain branches convert as type.opt.5 says, in either order. Every other difference stays expr.if.4. Native: the arms'
  type is settled by the first arm, so when a plain arm came first its edges into the join go through one block that
  makes the value present (`compiler/mir/lower_optarm.ail`).
- **expr.bind.4:** `_` declares nothing, so it may repeat in nested loops and comprehensions.
- **Certain fixes** (fix.auto.1), each a rewrite with one meaning:
  - expr.loop.4, `for c in s` with `s` a `str` local: `s.chars()`. The body is checked with a `char` either way.
  - type.struct.4, every missing field optional: each gets `none`. With other fields missing the fix stays a guess
    (`todo`), and the optional ones get `none` in it.
  - trait.op.1, `a - b` on two `char`s: `a.code() - b.code()`.
  - trait.op.3, an `int` beside an `i8`, `i16`, `i32`, `u8`, `u16` or `u32`: `.to_int()` on the sized operand
    (no value is lost; `u64` is left out because it can be).
  - type.coll.13, first use `xs.extend(ys)` or `xs.insert(i, e)` on a `[]`: the local is declared `List[T]()`.
- **Help only:** arithmetic with an optional operand says to take the value first; `char` with `int` names
  `c.code()` and `char.from_code`.

## Options considered

- **Keep expr.if.4 and correct the help text:** rejected. There is no form to recommend short of a helper function,
  and the mixed branches have one sensible type, the one a `none` branch already produces.
- **Typed locals (`x T? := e`):** rejected here; new syntax is the designer's call.
- **`char - char` as an operator:** rejected; characters are not numbers (type.char.1), and the fix teaches `code()`.
- **Implicit widening of sized integers to `int`:** rejected; same-type operands is the rule (type.num.13), and the
  fix costs the author nothing.
- **Type a `[]` by `extend` in the checker and the lowering:** rejected for now; the fix gives the same result with
  no second place to keep in step.
- **Optional fields default to `none`:** rejected; fields have explicit defaults (type.struct.8), and a silent one
  hides a forgotten field.

## Consequences

- `compiler/check/{body,call,expr,lateext}.ail`, `compiler/mir/lower_optarm.ail`, one line in `lower.ail`.
- Cases `optional_branch_widen`, `underscore_nested`; mistakes pairs `for_over_str`, `optional_field_omitted`,
  `char_minus_char`, `int_plus_u8`, `list_extend_first_use`; rule-only `char_plus_int`, `optional_plus_int`.
  The mistakes case `if_branch_plain_then_opt` is removed: it compiles now. The fix and fmt goldens need regenerating.
- Not built in the cloud (ADR 0842); gated on the Mac.
