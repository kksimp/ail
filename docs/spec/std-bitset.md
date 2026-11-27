# Standard library: `std.bitset`

Chapter key: `std`

> **Status:** Drafted (ADR 1246). Written in AIL (`std/bitset/bitset.ail`).

A fixed-size set of the integers 0 to n - 1, packed 64 to an `int` word. Every function is pure
(`eff.infer.1`) and needs no capability.

## The module

**std.bitset.1** — `bitset` is in scope without `use` (`std.use.4`); `use std.bitset` stays valid.
`bitset.new(n int) BitSet` is an empty set that holds 0 to `n - 1`; a negative `n` traps. `bitset.from_list(n, xs List[int]) BitSet` is the set of `n` bits holding `xs`; a member outside 0
to `n - 1` traps. `len() int` is `n`. A `BitSet` is a value: `t := s` is an independent copy.
*(ADR 1246)*

**std.bitset.2** — `test(i) bool` is whether `i` is a member; it is `false` for an `i` outside 0 to `len() - 1`,
never a trap. `set(i)`, `clear(i)` and `flip(i)` add, remove and toggle `i` on a `mut` set; an `i` outside 0 to
`len() - 1` traps. *(ADR 1246)*

**std.bitset.3** — `fill()` adds every number 0 to `len() - 1`; `reset()` removes every member. Both work on a `mut`
set. *(ADR 1246)*

**std.bitset.4** — `count() int` is the number of members, one population count per word; `is_empty() bool` is
whether there is none. *(ADR 1246)*

**std.bitset.5** — `members() List[int]` lists the members in ascending order. `next_set(from int) int?` is the
smallest member that is `from` or above; a negative `from` starts at 0, and a `from` at or above `len()` gives
`none`. Both step from one set bit to the next, so the cost follows the number of members and words, not `len()`.
*(ADR 1246)*

**std.bitset.6** — `complement() BitSet` has every number 0 to `len() - 1` that is not a member, and the same `len`.
*(ADR 1246)*

**std.bitset.7** — `union(o)`, `xor(o)`, `intersect(o)` and `difference(o)` return a new set and leave both
unchanged. `union` and `xor` have the larger `len` of the two, a bit past the end of the shorter counting as absent;
`intersect` and `difference` have `self`'s `len`. Each is one word operation (`|`, `^`, `&`, `& ~`) per word.
*(ADR 1246)*

**std.bitset.8** — `is_subset(o) bool` is whether every member of `self` is a member of `o`; `intersects(o) bool` is
whether the two share a member. A set of a different `len` compares as if the shorter were padded with absent bits.
*(ADR 1246)*

**std.bitset.9** — Bits at or above `len()` are always absent, so `==` on two sets compares their `len` and members
and nothing else. *(ADR 1246)*

```ail rules=std.bitset.1,std.bitset.2,std.bitset.3,std.bitset.4,std.bitset.5
fn main()
    mut s := bitset.new(100)
    s.set(7)
    s.set(64)
    s.flip(99)
    print(s.test(7))                    # true
    print(s.test(100))                  # false: outside the set, no trap
    print(s.count())                    # 3
    print(s.members())                  # [7, 64, 99]
    print(s.next_set(8) == 64)          # true
    s.reset()
    print(s.is_empty())                 # true
    s.fill()
    print(s.count())                    # 100
    print(bitset.from_list(10, [1, 3]).len())   # 10
```

```ail rules=std.bitset.6,std.bitset.7,std.bitset.8,std.bitset.9
fn main()
    a := bitset.from_list(10, [1, 2, 3])
    b := bitset.from_list(70, [3, 4, 69])
    print(a.union(b).members())         # [1, 2, 3, 4, 69]
    print(a.intersect(b).members())     # [3]
    print(a.difference(b).members())    # [1, 2]
    print(a.xor(b).members())           # [1, 2, 4, 69]
    print(a.is_subset(b))               # false
    print(a.intersects(b))              # true
    print(bitset.from_list(4, [0]).complement().members())   # [1, 2, 3]
    print(bitset.from_list(10, [1, 2]) == bitset.from_list(10, [2, 1]))   # true
```
