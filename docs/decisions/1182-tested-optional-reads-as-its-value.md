# 1182. A tested optional reads as its value

- **Status:** Accepted
- **Date:** 2026-11-26
- **Decided by:** Designer (chose it on the 2026-11-26 decision card, "All three"; proposal by the checker rough-edges lane). The shape below, which keeps every valid program's meaning, is the AI's (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 2 — Type system; AI understanding
- **Spec rules:** [type.opt.12](../spec/04-types.md)
- **Amends:** [ADR 0048](0048-optional-shorthand.md) (how an optional's value is reached)

## Context

Optional typing is the one cause of failed first drafts that kept its size across both paid runs. The probes of this
shape (`rough-edges-probes-main.txt`) all failed with no fix: `if a != none and a > 0` is expr.cmp.1, and `a + 1`
after `if a == none` then `return` is trait.op.3. Every typed language an agent has written narrows here, so the
agent writes the test and uses the name. `if v := x` stays the binding form; this rule is for the code that tests.

## Decision

- **type.opt.12:** a local bound once and not `mut`, of type `T?`, is known to be present in the block of
  `if x != none`, in the later branches and `else` of `if x == none`, to the right of `x != none and` and
  `x == none or`, in a comprehension after `if x != none`, and after an `if` statement with no `else` whose
  blocks all end in `return`, `break` or `continue`.
- **Where it reads as `T`:** as an operand of arithmetic, an ordering or unary `-`, a range bound, an index, the
  receiver of a field, method or index (an optional is never valid in these), and as a call argument, a `return`
  value or the value of `=` when `T` is the type expected there (a `T?` there is a type error today).
- **Everywhere else it is still `T?`:** `x else d`, `if v := x`, `x?`, `x == none`, `y := x`, a list element.
  So every program that was valid means what it meant, the compiler's own source included, which tests
  `if c != none` and then writes `c else 0`.
- **Decided on the syntax, once:** `compiler/ast/narrow.ail` walks a function and marks the uses. The checker
  (`compiler/check/narrow.ail`) and the native lowering (`compiler/mir/lower_narrow.ail`) both read that walk, so
  neither passes types to the other and they can't disagree about where the rule applies. The lowering unwraps at
  the read of the name.
- `mut` locals and fields are left out: a call or an assignment between the test and the use can change them.
  The one exception is the right side of `and` / `or` in the same condition (`best == none or v > best`), when
  that side is a plain expression: nothing can assign the local in between.
- The receiver of `x.unwrap()`, `x.is_some()` and the other methods agents bring from elsewhere stays optional, so
  the certain fixes for those habits still apply.

## Options considered

- **Full narrowing (`x` has type `T` in the whole block):** rejected. `y := x` would change type and
  `x else d` would stop compiling, which breaks valid programs, among them the compiler.
- **Narrow `mut` locals until the next assignment:** rejected for now; it needs flow analysis in two places and
  the binding form covers it.
- **A certain fix that rewrites the test to `if v := x`:** rejected; it has to rename every use and can't express
  `a != none and a > 0` followed by more code.
- **Types passed from the checker to the lowering:** rejected; the lowering has its own typing by design, and the
  one channel between them (`--parallel` loop keys) is optional.

## Consequences

- `compiler/ast/narrow.ail`, `compiler/check/{narrow,body,checker,expr}.ail`, `compiler/mir/lower_narrow.ail`;
  in `lower.ail` one state field with its initial value and one dispatch line in `lower_name` (ADR 0823).
- Cases `optional_narrow`, `optional_narrow_errors`. trait.op.3's help for an optional operand names the rule.
- A test in a `while` condition and a `for` over a tested optional list are not narrowed yet.
- More programs are accepted; none changes meaning.
- Not built in the cloud (ADR 0842); gated on the Mac.
