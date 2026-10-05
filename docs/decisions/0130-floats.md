# 0130. `float` is IEEE 754 binary64, with a Python-style text form and a total `cmp`

- **Status:** Accepted
- **Date:** 2026-10-05
- **Decided by:** Designer (the core choice: IEEE 754 binary64, like Python and Go)
- **Details decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)): the text form, the `cmp` order, the `to_float` rules, integer literals as floats, and the method names
- **Topic:** 4 — Type system; 21 — Standard library
- **Spec rules:** [type.num.6–11](../spec/04-types.md)
- **Extends:** [ADR 0055](0055-numeric-type-names.md) (`float` is `f64`), [ADR 0094](0094-numeric-literals.md) (literal typing), [ADR 0126](0126-automatic-equality-and-text.md) (`float` joins the types with automatic `Eq` and `ToStr`)
- **Amends:** [ADR 0127](0127-standard-comparison-and-text-traits.md) (trait.std.1–2: for `float` the operators are IEEE wherever they are written, including through an `Eq` or `Ord` bound; trait.std.4: `float` implements `Ord`)
- **Evidence:** `examples/json.ail` (parses and prints floats, output identical to the Python version); the design review of the three example programs (tokbench, mean of five tokenizers)

## Context

`float` exists as an alias for `f64` (ADR 0055), but nothing says what it can
do. type.num.5 covers integer operators only, `type.str.3` gives text forms
for `int`, `bool` and `str`, and there's no conversion in either direction.
The JSON example must read `2.5E-3`, print `0.0025`, sum to `973.2525`, and
check `reparsed == doc`, byte for byte against its Python original. That needs
exact parsing, a fixed text form and defined operators.

The designer chose IEEE 754 binary64 (not decimal, not a trapping mode). This
ADR records that choice and the details delegated under it.

## Options considered

The details barely change program text (a literal is one or two tokens
longer or shorter, and float code is a rare construct in the three programs),
so token usage ties. The table names the alternatives behind each detail.

| Detail | Chosen | Main alternative |
|---|---|---|
| Text form | Python `repr` | Rust `{}` (no exponent; `1000.0` prints `1000`), Go `%v` (`1e+06`) |
| `cmp` with NaN | total order, NaN greatest | NaN Equal to everything; NaN least (Go) |
| `<` through `Ord` | IEEE, also in generic code | derived from `cmp` |
| `s.to_float()` overflow | `inf` (Python) | `none` |
| `x.to_int()` out of range | traps | clamps |
| `x * 2` | `2` is a float | error (Rust) |
| `x.round()` | half away from zero | half to even (Python) |

## Deciding criterion

Token usage **ties** on every detail. **AI understanding** decides:
- Python's `repr` is the form agents know, it always reads back as a float
  (`1000.0`), and the example programs must print what their Python originals
  print.
- One answer for `<` wherever it's written, and a `cmp` that is a real order,
  so `sort` doesn't depend on the algorithm.
- A literal's meaning doesn't depend on a distant declaration: `x * 2` and
  `total: 0` work, as in Go and Swift. Rust-strict literals fail on the first
  attempt.
- Methods, not free functions: a free `round` or `abs` would stop programs
  from naming a local `round` (ADR 0084). The names are Rust's, and
  `to_int`/`to_float` pair with `to_int` on `str` (ADR 0124).

## Decision

1. **Operators (type.num.6).** `float` supports `+ - * /`, unary `-`, `+= -=
   *= /=` and `== != < <= > >=`, all IEEE 754 (round to nearest, ties to
   even). There is no `%`. Nothing traps: overflow gives ±inf, `x / 0.0` gives
   ±inf or NaN. NaN is unequal to everything including itself, every
   `<`, `<=`, `>`, `>=` with NaN is false, and `-0.0 == 0.0`. These operators
   are IEEE wherever written, including generic code with an `Eq` or `Ord`
   bound. Int and float never mix implicitly (trait.op.3): `n.to_float()`,
   `x.to_int()`.
2. **Text form (type.num.7).** Python's `repr`. The digits are the shortest
   decimal that reads back as the same value (ties: the one closest to the
   exact value). Write it as d.ddd×10^e. If −4 ≤ e < 16, use fixed notation
   with at least one digit after the `.`; otherwise `d[.ddd]e±XX`: sign
   always, at least two exponent digits, no `.0` on the mantissa. `e` is the
   exponent of the shortest digits, so `9999999999999999.0` (it rounds to
   1e16) prints `1e+16` and `9999999999999998.0` prints as written.
   Specials: `inf`, `-inf`, `nan` (never `-nan`), `-0.0`. The same form is
   used inside other values.
3. **`s.to_float() -> float?` (type.num.8).** Accepts an optional `-`, digits,
   optional `.digits`, optional exponent (`e` or `E`, optional sign, digits),
   and exactly `inf`, `-inf`, `nan`, so every text form reads back. No `+`,
   no `_`, no spaces, no leading or trailing `.`. The result is correctly
   rounded (nearest, ties to even). Overflow gives ±inf (as Python);
   underflow gives a subnormal or ±0.0; `"-0"` gives `-0.0`. Float literals
   are correctly rounded too; one that overflows is a lex.num.3 error.
4. **Methods (type.num.9).** `x.abs()`, `x.floor()`, `x.ceil()`, `x.round()`
   (half away from zero), `x.sqrt()` (NaN for negatives) return `float` and
   keep the sign of zero. `x.to_int() -> int` truncates toward zero and traps
   on NaN or when the result is outside `int` (that is, unless
   −2^63 ≤ x < 2^63). `n.to_float()` rounds to nearest, ties to even, and
   never traps.
5. **Integer literals (type.num.10).** An integer literal (or `-literal`)
   whose expected type is `float` is a float: `x * 2`, `Stats(total: 0)`,
   `t += 1` after `mut t := 0.0`. An int-typed expression never converts.
6. **Order (type.num.11).** `cmp` is a total order: numbers in numeric order,
   `-0.0` Equal to `0.0`, NaN Equal to NaN and Greater than every number. It
   is used by `sort`, `sorted`, `min`, `max`, `List[float]` and derived `Ord`
   on structs with float fields, so they are deterministic.
7. `float` implements `Eq`, `Ord` and `ToStr`, not `Hash` (ADR 0133). Float
   literal patterns are an error (expr.match.7). `f32` stays unimplemented.

```ail
x := 0.1 + 0.2
print(x)                      # 0.30000000000000004
print(1e16)                   # 1e+16
mut t := 0.0
t += 1                        # 1 is a float here
n := 7
print(n.to_float() / 2.0)     # 3.5
y := 2.5
print(y.round())              # 3.0 (Python's round(2.5) is 2)
```

## Consequences

- `sort` and `sorted` put NaN last, `min` ignores NaN unless every element is
  NaN, and `max` returns NaN if any element is.
- `float` is the one standard type where `a.cmp(b) == Equal` doesn't imply
  `a == b` (NaN), and where `cmp` and the operators disagree:
  `[nan] <= [nan]` is true (list order uses `cmp`), `nan <= nan` is false.
- A float can't be a map key or set element (ADR 0131, ADR 0133).
- Ports from Python meet two differences: `round` returns a float and rounds
  half away from zero, and `floor` and `ceil` return floats. Write
  `x.round().to_int()` for an int.
- `mut sum := 0` followed by `sum += x` makes `sum` an int: only a literal
  with an expected float type converts. Write `0.0`.
- `x.to_int()` traps where `s.to_int()` returns `int?`: a bad string is
  input, an out-of-range float is a bug, as with overflow (ADR 0045).
  `(2^63 - 1).to_float().to_int()` traps, because the float is exactly 2^63.

## Rejected alternatives

- **NaN Equal to everything under `cmp`:** not transitive (`1.0` Equal NaN
  Equal `2.0`, but `1.0` Less `2.0`), so a sort's result depends on the
  algorithm; Python's `sorted([3.0, nan, 1.0, 2.0])` comes back unchanged.
- **NaN least (Go's `cmp.Compare`):** a tie with NaN last. Last chosen, so
  `min` ignores NaN.
- **IEEE `totalOrder` (`-0.0` before `0.0`, `-nan` first):** disagrees with
  `-0.0 == 0.0` and is less familiar.
- **A `cmp` that traps on NaN:** sorting a list that holds one would crash.
- **Operators derived from `cmp`:** `nan <= 1.0` would be true, contradicting
  IEEE; generic and concrete code would disagree.
- **`to_float` returning `none` on overflow:** merges "out of range" with
  "malformed"; Python gives `inf`.
- **Clamped `to_int`:** a silent wrong answer.
- **Rust-strict literals (`x * 2` is an error):** first-attempt failures; the
  JSON drafter wrote `0.0` carefully, other agents won't.
- **Python's `round` (half to even, returns int) and int-returning `floor`:**
  one method name would return two types across the family; explicit
  `.to_int()` keeps the conversion visible and its trap rule in one place.
- **Other text forms:** Rust's `{}` prints `1000.0` as `1000` and never uses
  an exponent; Go's `%v` writes a million as `1e+06`; C's `%g` keeps six
  digits. None matches the Python originals or always reads back as a float.
- **`%` on floats:** not provided. Languages disagree on the sign of the
  result (Python follows the divisor, C and Rust the dividend).
