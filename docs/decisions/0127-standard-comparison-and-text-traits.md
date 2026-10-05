# 0127. The standard traits `Eq`, `Ord` and `ToStr`; an empty `impl` derives them

- **Status:** Accepted
- **Date:** 2026-10-05
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 10 — Traits; 21 — Standard library
- **Spec rules:** [trait.std.1–4](../spec/10-traits.md), [trait.derive.1](../spec/10-traits.md), [trait.op.1](../spec/10-traits.md), [type.str.3](../spec/04-types.md)
- **Extends:** [ADR 0077](0077-operator-traits.md) (names the comparison traits), [ADR 0081](0081-default-methods.md)
- **Amends:** [ADR 0116](0116-core-library-basics.md) (`str` gains ordering)
- **Evidence:** `tools/tokbench/corpus/r7/o*.ail`

## Deciding criterion

**Token usage** decides deriving: `impl Ord for Version` alone costs 16.0
tokens, against 57.2 for a hand-written `cmp`. The rest **ties** on tokens
(names are one or two tokens, used rarely), so **AI understanding** decides:
- `Eq`, `Ord` and `Ordering` (`Less`, `Equal`, `Greater`) are the names Rust
  and Haskell use. `Order` would collide with a common user type name.
- A three-way `cmp` is the convention of Rust, Haskell, Java, C#, Kotlin and
  Go's `cmp.Compare`. A `less` method is 7% smaller on the one rare
  construct that writes it (53.4 vs 57.2), which is a tie.
- `to_str` pairs with `to_int` (ADR 0124) and Rust's `to_string`.

## Decision

```ail
enum Ordering
    Less
    Equal
    Greater

trait Eq
    fn eq(self, other Self) bool

trait Ord                     # a type that is Ord is also Eq
    fn cmp(self, other Self) Ordering

trait ToStr
    fn to_str(self) str
```

1. `a == b` calls `a.eq(b)`, and `a != b` is its negation. `<`, `<=`, `>`
   and `>=` compare `a.cmp(b)` with `Less`, `Equal` and `Greater`.
2. A bound `Ord` also provides `==`.
3. An `impl` of `Eq`, `Ord` or `ToStr` with no methods derives the
   implementation: field by field in declaration order for structs, by
   variant declaration order and then fields for enums. Every field's type
   must implement the trait.
4. `int` and `str` implement `Ord`. `str` compares UTF-8 bytes
   lexicographically, which is Unicode code point order. Lists of an `Ord`
   type are ordered lexicographically.

```ail
struct Version
    major int
    minor int

impl Ord for Version          # compares major, then minor
```

## Consequences

- `"a" < "b"` works, so names can be sorted.
- `bool` and optionals aren't ordered.
- Declaring supertraits in user code (`Ord` extends `Eq`) needs syntax,
  which isn't designed yet (OQ-039).
- The arithmetic and indexing operator traits are still to be named, with
  numeric types.

## Rejected alternatives

- **`less(self, other Self) bool`:** a tie on tokens, and less familiar.
- **`Order`:** collides with a common user type name.
- **`Show`/`show`, `Display`/`fmt`:** `show` reads like printing, and `fmt`
  needs a formatter type that doesn't exist.
