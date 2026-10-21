# 0760. Lists of `int`, `float` and `bool` are stored packed

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; approved by the integrator after the measured prototype)
- **Topic:** 20 — Interpreter
- **Builds on:** [ADR 0759](0759-bytecode-tier-stalls-not-frames.md), [ADR 0621](0621-bytecode-bounds-checks-stay.md)

## Context

[ADR 0759](0759-bytecode-tier-stalls-not-frames.md) left sieve at 1.2x CPython: a `List[bool]` was
`Rc<Vec<Value>>` and a `Value` is 32 bytes, so 3,000,000 flags took 96 MB of memory traffic where CPython moves
8 bytes per element. Its instruction count had fallen 10% and its wall time had not moved. The same cost sits under
every large `List[int]` and `List[float]` (matmul rows, nbody's state).

A prototype that packed only named locals (`AIL_FLAT=1`, not merged) ran sieve in 0.23 s instead of 0.60 s and
matmul 21% faster, which is what this ADR commits to doing for every list.

## Decision

`Value::List` holds an `Rc<List>` (`bootstrap/src/interp/list.rs`):

```
enum List { Boxed(Vec<Value>), Bool(Vec<u8>), Int(Vec<i64>), Float(Vec<f64>) }
```

- **Packed means homogeneous.** A packed list holds only the one variant it was built from. A list literal or
  comprehension whose elements are all `int`, all `float` or all `bool` is built packed; so is an empty list at its
  first `push` or `insert` of one of those. Everything else stays boxed (`str`, structs, lists, optionals, sized numbers).
- **The kind is never taken from a static type.** A mismatch (a `none` pushed into a `List[int?]`) or an operation with
  no packed form turns the list into a boxed one in place (`List::boxed_mut`). That costs speed and never
  correctness; it needs no checker support and no second representation in the language.
- **Same behaviour, same bytes.** Copies are `Rc` clones with copy on write, as before (own.val.1, own.val.2).
  Equality, ordering, hashing as a map key, printing, JSON and `--trace` snapshots read a packed list through
  `List::get`, `iter` and `slice` and give the bytes a boxed list gives. A packed and a boxed list with the same
  elements are equal.
- **Where it is fast.** The tier's `Index`, `IndexOff` and `Index2` read an element straight from the vector into a
  register (`Got`, no `Value` is built), `SetIndex` writes a scalar into the vector (out of line, `store_packed`, to keep
  `exec` small), and `ListFast` pushes one. The walker lends an element to the place-assignment closures and writes it
  back (`with_in`), so `xs[i] = v` and `xs[i] += v` never unpack the list. `len`, `is_empty`, `first`, `last`,
  `contains`, `index_of`, `sum`, `min` and `max` read the vector in place; `sorted`, `reverse`, `swap`, `extend`,
  `insert`, `remove_at`, slicing and `take` keep the packed form. Every other list method works on `List::slice`, a copy
  for a packed list that the O(n) method pays for in passing.
- **Oracle.** `AIL_NO_PACK=1` builds every list boxed. The differential fuzzers compare a program with and without it
  (`fuzz.prop.6` in `tests/fuzz.rs`, `fuzz.bc.4` in `tests/fuzz_bytecode.rs`).
- **Native backend.** Not changed. A packed list is a leaf object for the collector (no pointers to trace), which the
  native runtime can use later; that is its own change.

## Result

Linux x86-64, `tools/perfbench/programs`, #445 head against this branch. Instructions are callgrind (millions, small
arguments); wall time is the minimum of 7-15 runs of `ail run PROGRAM` at the default size (CPython 3 in the last column).

| Program | Instr. before | after | Change | Wall before | after | After / CPython |
|---|---:|---:|---:|---:|---:|---:|
| sieve | 289.5 | 281.8 | -2.7% | 0.590 s | 0.208 s | 0.43x (was 1.24x) |
| matmul | 128.8 | 122.6 | -4.8% | 0.368 s | 0.275 s | 0.70x |
| binarytrees | 369.2 | 357.9 | -3.1% | 0.812 s | 0.722 s | 1.65x |
| nbody | 218.8 | 213.3 | -2.5% | 0.233 s | 0.250 s | 0.81x |
| fannkuch | 81.5 | 83.5 | +2.5% | 0.613 s | 0.603 s | 0.84x |
| collatz, spectralnorm, mandelbrot | 909.1, 121.9, 300.5 | 851.3, 116.5, 289.8 | -6.4%, -4.4%, -3.6% | within noise | within noise | 0.35x, 0.86x, 0.76x |

Geometric mean over CPython, eight programs: 0.871 before, 0.725 after.

Memory, sieve at `n` flags (peak resident set; the interpreter alone is 10 MB):

| n | Before | After | Per flag before | after |
|---:|---:|---:|---:|---:|
| 3,000,000 | 101 MB | 12 MB | 30 bytes | under 1 byte |
| 10,000,000 | 315 MB | 19 MB | 30.5 bytes | 0.9 bytes |

Binarytrees uses no lists: its instruction count moved with the layout of `exec` (ADR 0759) and its wall time is
still 1.5-1.7x CPython. Nbody is 5-7% slower by wall time with fewer instructions: its `s[i] += x` calls the out-of-line
store; inlining the store made mandelbrot, which uses no lists, 9% slower instead.

Checks on the final commit: `./ail selftest` exit 0; `fuzz_bytecode` 30,000 programs from seed 1 and from seed 777,
0 disagreements between the walker, the tier and the tier with `AIL_NO_PACK=1`; `fuzz` 10,000 programs with `fuzz.prop.6`,
0 packed-versus-boxed disagreements (its four `fuzz.mut.1` failures, seeds 2673, 8486, 8914 and 9087, reproduce on the
#445 head: `ail fix` output that no longer parses, not lists).

## Rejected alternatives

- **A `Value::Flat` variant beside `Value::List`** (the prototype). Every `Value::List` match that missed the new
  variant would fail at run time; with one `List` type behind `Value::List` the compiler found all 136 sites.
- **`Rc<RefCell<..>>` storage.** The prototype's borrow on every element cost 8% in instructions on sieve.
- **Taking the kind from the static element type.** Needs the checker's types at every construction site and a
  dynamic upgrade path anyway for `List[int?]`; the run-time kind is simpler and safe.
- **One `Vec<u64>` for all scalars, or bit-packed `List[bool]`.** No byte `bool`, transmutes for `float`; a bit store is
  a read-modify-write that slows the sieve's inner loop. Revisit the bit form after the byte form.
- **Packing every list at creation** (`json.load`, `split`, `range`). Only literals, comprehensions and the first
  `push` pack today; a boxed list is still correct, and the other builders can follow where a measurement asks.

## Not done

- A bytecode instruction for `grid[i][j] = v` (it hands off to the walker, which now costs about 160 instructions more
  per write than before because it lends the element); `IterNext` over a packed list builds a `Value` per element.
- Sized numbers (`Num`) and `char` are not packed.
