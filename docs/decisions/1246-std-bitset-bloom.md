# 1246. `std.bitset` and `std.bloom`: a dense bit set and a Bloom filter in AIL

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked for a fuller std on 2026-11-26
- **Topic:** 17 — Standard library
- **Spec rules:** [std.bitset.1 to std.bitset.9](../spec/std-bitset.md), [std.bloom.1 to std.bloom.8](../spec/std-bloom.md)

## Context

Agents write a bit set (sieves, visited flags, small-integer sets) and a Bloom filter (seen-before checks) by hand, and
the corpus holds both (`bit_set_ops`, `bloom_filter_int`). The hand-written forms use one `int` per bit or 32 bits per
word, count bits one at a time, and pick ad hoc hash functions. The std has no bit set, no population count and no
Bloom filter. Speed matters: a sieve or a large filter is bound by memory and by word operations.

## Decision

- Two pure AIL modules, `std/bitset/bitset.ail` and `std/bloom/bloom.ail`, in scope without `use`. No runtime or
  checker code: registration lines only.
- `BitSet` holds `List[int]` words of 64 bits. Test, set, clear and flip are one shift and mask; `count` is a
  branch-free SWAR population count per word; the set algebra is one `&`, `|` or `^` per word; `members` and
  `next_set` find the lowest set bit with `w & -w` and clear it with `w & (w - 1)`, so zeros cost nothing. `int` has
  no population count or trailing-zero method, and `>>` is arithmetic, so the count masks after each shift and uses
  `wrapping_sub` and `wrapping_mul`. A native `popcount` method would be a later builtin; this module's `popcount`
  is the one place to swap it in.
- Bits at or above `len` stay 0, so `==` and `complement` are correct without a mask on read.
- `test` is total (false outside the range, like `xs.get(i)`); `set`, `clear` and `flip` use a `requires`
  contract and trap outside the range, like `xs[i]`.
- Set algebra returns new sets, and a shorter operand counts as padded with absent bits. `union` and `xor` take the
  larger `len`, `intersect` and `difference` keep `self`'s.
- `Bloom` is a `BitSet` plus a hash count, sized by `bloom.new(expected, rate)` with the textbook formulas. Keys are
  `str` (hashed with `hash.hash64`) or `int` (SplitMix64 finalizer). Bit `i` of a key is `(h1 + i * h2) mod bits`
  (Kirsch and Mitzenmacher), so one hash and one mix give every index. `union` of two filters of one shape is `none`
  otherwise.
- Module names `bitset` and `bloom` are not common locals; the types are `BitSet` and `Bloom`.

## Rejected alternatives

- **`std.bits` or `bitvec`:** `bits` is a common local name (std.use.4 would hide the module); `bitset` says what it is.
- **32-bit words:** the corpus form; halves the work per word for no gain on 64-bit targets (ADR 0007).
- **One `bool` or one `int` per bit in a `List`:** 8 to 64 times the memory and one operation per bit.
- **`and`, `or`, `not` as method names:** they are operators in AIL. `union`, `intersect` and `difference` match the
  collection vocabulary of `Set` (ADR 1099).
- **In-place algebra (`union_with`):** doubles the surface; a new set per call is the same cost as the pass over words
  and keeps values independent (mutable value semantics, ADR 0060).
- **A `T ! BitSetError` for an index out of range:** an error type for a programming mistake; list indexing traps
  and so do these.
- **A generic `Bloom[T]`:** needs a hashing trait for user types (ADR 0955 is open); `str` and `int` keys cover
  nearly every use, and a user type hashes through its text or an `int` field.
- **A native runtime `hash` for ints:** the mixer is five word operations in AIL; the runtime call would add a
  lowering file for no measured gain.
- **Counting or deletable filters, scalable filters:** a different structure with a different API; later.
- **Rounding `bits` up to a multiple of 64:** hides the size the formula gave; `bits()` reports exactly what was asked.

## Unverified at writing

The lane runs in a cloud container with no `./ail` build (AGENTS.md), so the code, tests and goldens are written
without a compile. The integrator's Mac gate is the first build. The `.out` goldens hold only values that follow from
the spec (counts, members, sizes from the formula, and true or false for the bounded false-positive share).
