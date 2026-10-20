# 0227. A trait may name a derive generator: a compile-time function from `TypeInfo` to AIL source

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 17 — Metaprogramming
- **Spec rules:** [ct.derive.1–4](../spec/15-compile-time.md), [trait.derive.1](../spec/10-traits.md) (unchanged use site)
- **Design note:** [metaprogramming.md](../design/metaprogramming.md)

## Context

The common need for metaprogramming is one implementation per type
(serialisation, test generators, conversions). AIL already derives `Eq`,
`Ord` and `ToStr` from an empty `impl` (`trait.derive.1`), but a library
trait can't be derived. The plan asks for the smallest mechanism with no
hidden code and no textual macros. Measured with `tools/tokbench` (mean of
five tokenizers):

| Use site | 1 derive | 3 derives |
|---|---|---|
| empty `impl T for X` (existing) | 16.0 | 31.8 |
| `@derive(...)` on the type | 15.6 | 25.4 |
| `derives` clause | 12.8 | 22.6 |

Derives appear about 10 times in the 2,099 lines of `examples/`, so the
largest saving is under 0.2% of a program: a **tie on tokens**.
**AI understanding decides**: keep the one existing way to implement a
trait. For the generator, a `quote`/splice form measured 69.6 against 77.6
for returning text, but generators are rare library code (a tie), and text
needs no new syntax.

## Options considered

### Option A — `@derive(gen)` on the trait; generator returns AIL text (chosen)
- **Pros:** no new use-site syntax; the generator is an ordinary,
  testable, compile-time-safe function; its output is AIL the agent already
  reads, checked in place.
- **Cons:** a generator's mistakes surface when a type derives it, not when
  it is written; mitigated by testing generators with `@test`.

### Option B — typed syntax trees with `quote`/`$()` (Rust proc macros, Swift)
- **Pros:** malformed output is caught when the generator is checked.
- **Cons:** two new syntaxes, a `Code` type and hygiene rules.

### Option C — attribute on the type, `@derive(Ord, ToJson) struct X`
- **Pros:** one line for many derives.
- **Cons:** a second way to implement a trait; a tie on tokens.

## Decision

1. `@derive(f)` before a `trait` names its generator `f`, a function
   `fn f(t TypeInfo) str ! DeriveError` (`ct.derive.1`).
2. An `impl Trait for T` with no method definitions derives it; for a trait
   with a generator, the compiler calls the generator at compile time with
   `T`'s `TypeInfo` (ADR 0229) (`ct.derive.2`).
3. The generator must be compile-time-safe and runs under the limits of
   ADR 0226 (`ct.derive.3`).
4. A `DeriveError`, a trap, or output that doesn't check is a compile error
   at the `impl` line (`ct.derive.4`).
5. No call-site macros, no textual macros, no attribute macros that rewrite
   their item.

## Consequences

- **Designer, look first at:** generators returning *text* (Option A)
  rather than syntax trees (Option B).
- `Arbitrary` (generated property-test inputs, ADR 0198) and `ToJson` /
  `FromJson` for `std.json` become library derives once the bootstrap
  implements this.
- The bootstrap doesn't implement generators yet: `@derive` on a trait is
  an unknown attribute (`test.attr.1`) until it does. Spec examples are
  tagged `skip=bootstrap`.

## Rejected alternatives

- **Quoted syntax trees (B):** more syntax for a rare construct; can be a
  library that returns text.
- **Type-site attribute or clause (C):** a second way; tokens tie.
- **Call-site macros (Rust `macro_rules!`, C preprocessor):** syntax that
  exists only after expansion.
- **Unrestricted compile-time code generation (Zig `@Type`, D mixins
  anywhere):** the use site would no longer say what happens.
