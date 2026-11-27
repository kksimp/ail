# Standard library: `std.bloom`

Chapter key: `std`

> **Status:** Drafted (ADR TBD). Written in AIL (`std/bloom/bloom.ail`) over `std.bitset` and `std.hash`.

A Bloom filter: a set that answers "never added" or "probably added" in a fixed amount of memory. It never gives a
false negative. Every function is pure (`eff.infer.1`) and needs no capability.

## The module

**std.bloom.1** — `bloom` is in scope without `use` (`std.use.4`); `use std.bloom` stays valid.
`bloom.new(expected int, rate float) Bloom` is a filter for about `expected` keys with a false-positive share near
`rate`: `bits = ceil(-expected * ln(rate) / ln(2)^2)` and `hashes = round(bits / expected * ln(2))`, at least 1. An
`expected` below 1, or a `rate` not above 0 and below 1, traps. `bloom.new(1000, 0.01)`
has 9586 bits and 7 hashes. *(ADR TBD)*

**std.bloom.2** — `bloom.with_size(bits int, hashes int) Bloom` is a filter of exactly that many bits and hashes per
key; either below 1 traps. *(ADR TBD)*

**std.bloom.3** — `add(key str)` and `add_int(key int)` add a key to a `mut` filter. A `str` key is hashed with
`hash.hash64` (`std.hash.4`), an `int` key with a 64-bit mixer. Each key sets `hashes()` bits found by double hashing
(Kirsch and Mitzenmacher): bit `i` is `(h1 + i * h2) mod bits`, with `h2` from a second mix of `h1`, kept from 1 to
`bits - 1`. The bits are the same on every platform and version. The hashes are not keyed and the filter is not for
security. *(ADR TBD)*

**std.bloom.4** — `has(key str) bool` and `has_int(key int) bool` are `false` when the key was never added with the
matching `add` or `add_int`, and `true` when it was; for a key never added they are `true` in about `rate` of cases
(a false positive), and `false` otherwise. A `str` key and an `int` key are different keys even when the text of the
number is the same. *(ADR TBD)*

**std.bloom.5** — `bits() int` and `hashes() int` give the size; `count() int` is the number of `add` and `add_int`
calls so far, a repeated key included. *(ADR TBD)*

**std.bloom.6** — `fill_ratio() float` is the share of bits set, 0.0 to 1.0. `false_positive_rate() float` is
`fill_ratio() ^ hashes()`, the share of never-added keys `has` would report as added right now; it passes the `rate`
given to `new` once more than `expected` keys are in. *(ADR TBD)*

**std.bloom.7** — `reset()` removes every key and sets `count()` to 0. *(ADR TBD)*

**std.bloom.8** — `union(o) Bloom?` is a filter that has every key of both, with the bits OR-ed and the counts added.
It is `none` when `bits()` or `hashes()` differ, since the bits then mean different keys. *(ADR TBD)*

```ail rules=std.bloom.1,std.bloom.2,std.bloom.3,std.bloom.4,std.bloom.5
fn main()
    mut seen := bloom.new(1000, 0.01)
    print(seen.bits())                  # 9586
    print(seen.hashes())                # 7
    seen.add("alice")
    seen.add_int(42)
    print(seen.has("alice"))            # true: an added key is always found
    print(seen.has_int(42))             # true
    print(seen.count())                 # 2
    print(bloom.with_size(256, 3).bits())   # 256
```

```ail rules=std.bloom.6,std.bloom.7,std.bloom.8
fn main()
    mut a := bloom.with_size(256, 3)
    mut b := bloom.with_size(256, 3)
    a.add("x")
    b.add("y")
    both := a.union(b) else
        return
    print(both.has("x") and both.has("y"))      # true
    print(a.union(bloom.with_size(128, 3)) == none)   # true: the shapes differ
    print(a.fill_ratio() < 0.02)        # true: three of 256 bits
    a.reset()
    print(a.count())                    # 0
```
