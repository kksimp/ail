# 0116. Sizes and indexes are `int`; list basics; text forms of `int`, `bool` and `str`

- **Status:** Accepted; amended by [ADR 0127](0127-standard-comparison-and-text-traits.md) (`str` gains ordering)
- **Date:** 2026-10-04
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 4 — Type system; 21 — Standard library
- **Spec rules:** [type.coll.5–6](../spec/04-types.md), [type.str.3](../spec/04-types.md), [type.num.5](../spec/04-types.md)
- **Resolves:** the size and index type left open by `plat.ptr.2` ([ADR 0007](0007-64-bit-little-endian-only.md))

## Context

Running real programs needs a few library basics before the standard library
is designed: how long a list is, how to add to one, and how `print` and
string interpolation show numbers and booleans.

## Deciding criterion

**Token usage** decides the index type. With an unsigned size type (Rust's
`usize`), code is full of conversions such as `i as usize` and
`xs.len() as i64`. With `int` there are none. **AI understanding** agrees:
mixing signed and unsigned integers is a classic bug source, and Go, Java,
C# and Swift all use a signed index type. **Speed** is unaffected, because a
bounds check on a signed index is still one unsigned comparison
(`i as u64 < len`).

The method names tie on tokens. `len` (Go, Python) and `push` (Rust,
JavaScript) are the most common spellings in the code AIs read.

## Decision

1. The size and index type is `int` (`i64`). Indexes, lengths and
   counts are `int`.
2. `xs.len()` returns the number of elements of a `List[T]` as `int`.
3. `xs.push(v)` appends `v` to a `List[T]`. `push` takes `mut self`, so
   `xs` must be a mutable place (`trait.method.3`).
4. Text forms (used by `print` and interpolation): an `int` is written in
   decimal, with a leading `-` when negative. A `bool` is `true` or
   `false`. A `str` is itself. Other types get text forms from the
   formatting trait (ADR 0088), which isn't designed yet.
5. Integer types support `+ - * / %`, unary `-` and all six comparisons.
   `bool` and `str` support `==` and `!=` (`str` compares its UTF-8
   bytes).

## Consequences

- Whether user-defined types get `==` automatically, and how `str` values
  are ordered, are decided with the standard-library traits.

## Rejected alternatives

- **An unsigned size type** (`usize`): conversions on almost every index.
- **`length`, `size`, `count`; `append`, `add`**: no token benefit and
  less common.
