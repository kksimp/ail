# Standard library: `std.simd` vectors

Chapter key: `std`

> **Status:** Draft (ADR 1014), slice S1: `f64x4` only. The tests are
> `tests/native/run/simd_f64x4.ail` and its golden. More types (`f32x8`,
> integers, `u8x32`, 128-bit forms), masks and `select` follow in S2-S4.

**std.simd.1** — `use std.simd` provides the value type `f64x4`: four `float`
lanes, 256 bits. `simd.splat_f64x4(x)` has `x` in every lane.
`simd.load_f64x4(xs, i)` takes lanes `xs[i]` to `xs[i + 3]` of a `List[float]`
and traps (`type.coll.4`) unless both `i` and `i + 3` are indexes of `xs`.
`simd.store_f64x4(mut xs, i, v)` writes the lanes of `v` to those elements,
with the same check. A vector is a value: copying it, or changing the variable
that holds it, never changes another copy. *(ADR 1014)*

**std.simd.2** — `a + b`, `a - b`, `a * b` and `a / b` on two `f64x4` work lane
by lane: lane `k` of the result is lane `k` of `a` and lane `k` of `b` under
the `float` operation, as IEEE 754 binary64, with no fused multiply-add. A
program prints the same on every target. *(ADR 1014)*

**std.simd.3** — `v.sum()` is the sum of the four lanes added as a balanced
tree: `(l0 + l1) + (l2 + l3)`. The order is fixed, so the result is the same on
every target. *(ADR 1014)*

**std.simd.4** — A vector has no `==`, `<` or other comparison in S1: masks come
with S2. Using one is a compile error. *(ADR 1014)*

**std.simd.5** — A vector stays inside its function. It can't be a parameter, a
result, a struct field or a list element, or passed to a call: the compiler
reports it. (A function it is inlined into counts as the same function.)
*(ADR 1014)*

```ail rules=std.simd.1,std.simd.2,std.simd.3
use std.simd

fn main()
    xs := [1.0, 2.0, 3.0, 4.0, 5.0]
    a := simd.load_f64x4(xs, 1)
    b := simd.splat_f64x4(0.5)
    print((a * b + b).sum())    # 9.0: 1.5 + 2.0 + 2.5 + 3.0
    mut out := [0.0, 0.0, 0.0, 0.0, 0.0]
    simd.store_f64x4(mut out, 1, a / b)
    print(out)                  # [0.0, 4.0, 6.0, 8.0, 10.0]
```

## Rejected alternatives

- **Methods `v.store(mut xs, i)`.** Wanted by ADR 1014's design; S1 takes the
  function form because a built-in method with a `mut` argument other than
  `self` isn't in the checker yet.
- **Operators on structs of lanes.** `Add` and its kin can't be implemented
  (`trait.op.1`), and a struct costs a heap object.
