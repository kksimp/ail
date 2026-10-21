# 0137. Integer arithmetic methods, `min` and `max` as `Ord` methods, and inherent methods beat prelude defaults

- **Status:** Accepted
- **Date:** 2026-10-05
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 4 — Type system; 10 — Traits; 21 — Standard library
- **Spec rules:** [type.num.12](../spec/04-types.md), [trait.res.2](../spec/10-traits.md) (new), [trait.std.2](../spec/10-traits.md) (amended: `Ord` default methods), [expr.arith.2](../spec/05-expressions.md) (syntax now decided)
- **Amends:** [ADR 0082](0082-method-placement-coherence.md) (adds `trait.res.2` to its call-resolution rules), [ADR 0127](0127-standard-comparison-and-text-traits.md) (`Ord` gains default methods `min` and `max`)
- **Extends:** [ADR 0035](0035-integer-overflow-traps.md) (the explicit forms it requires), [ADR 0081](0081-default-methods.md)
- **Evidence:** `examples/interp.ail`, `examples/json.ail`, `examples/sheet.ail`; `tools/tokbench` (mean of five tokenizers)

## Context

ADR 0035 requires explicit wrapping, saturating and checked arithmetic and
leaves the syntax open (`expr.arith.2`). The interpreter program needs checked
`+ - * /` and negation to report overflow as a language error instead of
trapping. The programs also take the least or greatest of two values: JSON's
nesting depth, and the spreadsheet's range corners and column widths (7 call
sites).

Two designs for `min` and `max` collide on lists. The programs want
`xs.min() -> T?` for a list (ADR 0136), and also `a.min(b)` for two values.
If `a.min(b)` is a default method of `Ord`, a `List[int]` has it too
(`trait.std.4`), so `nums.min()` could mean either method, and `trait.res.1`
(ADR 0082) makes that a compile-time error whatever the argument count. Any
user type with `impl Ord` and its own `min` method would break the same way.

## Options considered

Seven call sites (json 2, sheet 5), mean of five tokenizers:

| Option | Example | Tokens |
|---|---|---|
| Free prelude functions | `max(w, n)` | 87.0 |
| **Methods (chosen)** | **`w.max(n)`** | **82.8** (−4.8%) |

On the spreadsheet's three lines alone it is 58.2 against 55.2 (−5.2%). That is
0.07% of the whole file.

Where the methods live:

| Option | Generic code | Clash with `xs.min()` |
|---|---|---|
| **`Ord` default methods plus `trait.res.2` (chosen)** | `fn clamp[T Ord]` works | none: the list's own method wins |
| Inherent methods of `int`, `float`, `char`, `str` only | not available | none |
| `Ord` defaults, list aggregates renamed (`least`, `greatest`) | works | none |
| `Ord` defaults, resolution by argument count | works | none, but `xs.min()` and `xs.min(ys)` coexist |

## Deciding criterion

Token usage **ties** between free functions and methods (under 0.1% of any
file). **AI understanding** decides, by ADR 0084: methods keep `min` and `max`
legal as local names (`expr.bind.5`), and the prelude stays small. The three
drafts split (the sheet wrote the free form 5 times, the JSON program the
method form twice), so the free spelling gets a fixed help (ADR 0140).

Between the method homes, AI understanding decides: one definition on `Ord`
covers every orderable type and generic code, where scalar-only methods would
repeat the rule four times and leave `T Ord` code without it. `trait.res.2`
keeps "what can `x.f()` mean?" answerable from the receiver's type alone, the
aim of ADR 0082.

The integer method names tie on tokens (used at a handful of sites). The
Rust-style names (`checked_add`, `wrapping_add`, `saturating_add`) decide on AI
understanding: their meaning is the same everywhere AIs have seen them.

## Decision

1. **type.num.12.** `int` has these methods (the bootstrap implements `int`
   only; the narrower types of `type.num.1` aren't covered here):
   - `checked_add`, `checked_sub`, `checked_mul`, `checked_div`,
     `checked_rem` and `checked_neg`, returning `int?`: none on overflow or a
     zero divisor;
   - `wrapping_add`, `wrapping_sub`, `wrapping_mul`, `wrapping_neg` and
     `saturating_add`, `saturating_sub`, `saturating_mul`, returning `int`;
   - `abs()` (traps on the minimum value, like unary `-`), `to_float()`
     (`type.num.9`) and `to_hex()` (`type.str.8`).

   This satisfies `expr.arith.2` for add, subtract, multiply and negate.
   Division and remainder have the checked forms; wrapping and saturating
   division are deferred.
2. **`Ord` default methods (`trait.decl.2`).** `a.min(b)` is `b` when
   `b.cmp(a)` is `Less`, otherwise `a`. `a.max(b)` is `b` when `b.cmp(a)` is
   `Greater`, otherwise `a`. A tie gives `a`. Every `Ord` type has them,
   also through a bound.
3. **trait.res.2 (new; amends `trait.res.1`).** A built-in or inherent method
   takes precedence over a default method of a standard (prelude) trait with
   the same name. A list's own `xs.min()` therefore hides `Ord`'s
   `min(other)`, and a user type's own `min` keeps working when it implements
   `Ord`. A qualified call (`Ord.min(a, b)`) still selects the default. Methods
   of user traits stay ambiguous.
4. **No free `min` or `max`.** The unknown-name helps are `min(a, b)` →
   `a.min(b)` and `min(xs)` → `xs.min()` (same for `max`).

```ail excerpt
fn clamp[T Ord](x T, lo T, hi T) T
    x.max(lo).min(hi)

deepest := depth.max(inner)              # an int
total := a.checked_add(b) else return    # leaves the function on overflow
lo := nums.min() else 0                  # the list's own min, a T?
```

## Consequences

- Float `min` and `max` follow `cmp` (type.num.11), not the IEEE operators:
  NaN is greatest, so `a.min(nan)` is `a` and `a.max(nan)` is NaN; `-0.0` and
  `0.0` tie, giving the receiver.
- Adding a default method to a prelude trait can't break a user type's
  existing method of that name, so extending `Eq`, `Ord` and `ToStr` stays
  cheap.
- The sheet's three free-function lines become method calls.

## Rejected alternatives

- **Free `min(a, b)` and `max(a, b)`** (Python, Go, Swift): a tie on tokens
  (87.0 against 82.8), but a prelude name blocks locals called `min` or `max`
  (ADR 0084). Their only gain was avoiding the `trait.res.1` clash, which
  `trait.res.2` removes.
- **Methods on scalars only:** the same call-site tokens, but generic code
  (`fn clamp[T Ord]`) and user `Ord` types can't use them, and `int`,
  `float`, `char` and `str` would each declare the same rule.
- **Renaming the list aggregates** (`least`, `greatest`): unfamiliar names for
  the most common spelling of the operation (`min(xs)`, `xs.min()`).
- **Resolving by argument count:** `xs.min()` and `xs.min(ys)` would then
  both exist on a list, and one call would depend on arity, against
  ADR 0082's single-place rule.
- **Operators for the explicit forms** (such as `+%`): new lexemes and
  grammar, and `trait.op.2` closes operator definition; methods need neither.
