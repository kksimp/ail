# 0136. List methods, open-ended slices that trap, and `xs.sum()`

- **Status:** Accepted
- **Date:** 2026-10-05
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 4 — Type system; 21 — Standard library
- **Spec rules:** [type.coll.4](../spec/04-types.md) (amended), [type.coll.9–12](../spec/04-types.md), [expr.cmp.3](../spec/05-expressions.md)
- **Extends:** [ADR 0090](0090-collections.md) (lists, indexing and slices), [ADR 0116](0116-core-library-basics.md) (`len`, `push`)
- **Relates to:** [ADR 0128](0128-methods-on-generic-types.md) (`sum` is a special case that doesn't change it), [ADR 0137](0137-integer-methods-and-min-max.md) (`xs.min()` and Ord's `a.min(b)`)
- **Evidence:** `examples/interp.ail`, `examples/json.ail`, `examples/sheet.ail`; `tools/tokbench` (mean of five tokenizers)

## Context

Three programs (an interpreter, a JSON tool and a spreadsheet engine, 2,036
lines that run identically to their Python versions) needed more of a list
than `len`, `push`, `swap`, indexing and `a..b` slices (ADR 0090, 0116, 0119):
popping a scope, the last element, searching, joining text, slicing from or up
to a position, sorting, and the least, greatest and total of a list.

Python and Rust habits cause three silent failures:
- Python slices clamp (`s[:20]` on a short string is fine) and accept `xs[-1]`.
  A trap is the right answer in AIL, but `xs[-1]` should fail at compile time.
- JavaScript, Java, Kotlin and Go return `-1` from `indexOf`. AIL's `index_of`
  returns `int?`, and `xs.index_of(x) == -1` type-checks (the `-1` is wrapped,
  `type.opt.5`) and is always false. `index_of(x) < 0` is already an error,
  because optionals aren't `Ord` (ADR 0127).
- Python 3.12's `sum()` over floats is compensated; a loop is not.

## Options considered

Where tokens differ they are the mean of five tokenizers.

| Question | Option | Tokens |
|---|---|---|
| Join chars into a string (7 sites) | `str.from_chars(xs)` | 122.4 |
| | **`xs.join("")` (chosen)** | **106.2** (−13%) |
| Total of a list (sheet, whole file) | a user `total(xs)` helper | 6042.0 |
| | **`xs.sum()` (chosen)** | **6019.4** (−0.4%) |
| Slice out of range | clamp like Python | none |
| | **trap (chosen)** | none |

## Deciding criterion

- **`join`:** token usage decides, though only weakly: −2.3 tokens per site,
  under 0.1% of any file, so formally a tie. **AI understanding** decides the
  rest: one way to build text (`join` works on any list), not two.
- **`sum`:** −0.4% is a tie. **AI understanding** decides: `sum` is the name
  agents know, the sheet draft reached for it, and the alternative is a helper
  that every program re-declares.
- **Slices and `index_of`:** no token difference. **AI understanding** decides
  (ADR 0076 criterion 2): a clamped slice or an always-false comparison is a
  wrong answer with no diagnostic, so each is a trap or a compile-time error
  with a help.
- `pop`, `last`, `contains`, `index_of`, `sorted`, `sort`, `min`, `max` tie on
  tokens; the names are the common ones in Rust, Kotlin and Swift (`last`,
  `sorted`) and Python (`pop`).

## Decision

1. **type.coll.4 (amended).** `xs[a..b]` traps unless `0 <= a <= b <= len`.
   There's no clamping. A negative integer literal as an index or slice bound
   is a compile-time error, with help `xs[xs.len() - 1]` or `xs.last()`.
2. **type.coll.9.** List methods: `pop() -> T?` (`mut self`; removes and
   returns the last element), `last() -> T?`, `contains(x) -> bool` and
   `index_of(x) -> int?` (`T Eq`; none if absent), and `join(sep) -> str` for a
   list of any `T`. `join` uses each element's text form; `str` and `char`
   elements are unquoted at the top level, and nested values use the quoted
   form (`[['a']].join("")` is `['a']`).
3. **type.coll.10.** Open-ended slices: `xs[a..]`, `xs[..b]`, `xs[..=b]`, with
   the same bounds rule (`..=b` ends at `b + 1`, computed without overflow). An
   open range exists only inside `[]`; `xs[..]` is an error (write `xs`).
4. **type.coll.11.** `sorted() -> List[T]` and `sort()` (`mut self`) are stable
   and compare with `cmp`. `min()` and `max()` return `T?`: the first least or
   greatest element, none if the list is empty. Binding the result of `sort()`
   gets the help "use `sorted()`".
5. **type.coll.12.** `xs.sum()` exists only for `List[int]` (int) and
   `List[float]` (float). It starts at `0` or `0.0` and adds left to right with
   `+`, so an int overflow traps at the first overflowing partial sum and an
   empty list gives `0` or `0.0`. It is a built-in special case: user code
   can't declare a method for one instantiation, so [ADR 0128](0128-methods-on-generic-types.md)
   (and `gen.method.*`) stand unchanged.
6. **expr.cmp.3.** `==` and `!=` between an optional and a negative integer
   literal is an error, with help "`index_of` returns `none` when absent, not
   -1".

```ail
xs := [3, 1, 2]
last := xs.last()                   # 2, an int?
if i := xs.index_of(1)
    print(xs[i..])                  # [1, 2]
line := xs.sorted().join(", ")      # "1, 2, 3"
total := xs.sum()                   # 6
# xs[-1]                            # error: use xs[xs.len() - 1] or xs.last()
# xs.index_of(9) == -1              # error (expr.cmp.3)
```

## Consequences

- `sum` is the first library method that user code couldn't write. A numeric
  trait would replace the special case later (arithmetic traits are still to be
  named, trait.op.1); until then it's recorded as the one exception.
- Float `sum` matches a Python loop and Python 3.11, not 3.12's compensated
  `sum()`. `[-0.0].sum()` is `0.0`.
- Float `min()` and `max()` use the total order (type.num.11): NaN sorts last,
  so `min()` skips NaN and `max()` returns it.
- `xs.min()` and `xs.max()` don't clash with Ord's `a.min(b)` on a list,
  because of `trait.res.2` (ADR 0137).
- Slices that clamp in Python port as traps; the quick reference says so.
- `s.chars()` and `join("")` replace `str.from_chars` (ADR 0135).

## Rejected alternatives

- **Clamped slices (Python):** silent wrong answers, and `xs[a..b]` with
  `a > b` has no sensible result.
- **`str.from_chars(xs)`:** +13% tokens on the 7 call sites, and a second way
  to join; `join("")` is also the one name for text from any list.
- **A free `sum(xs)`:** `sum` is a common local name, and a prelude function
  would make it illegal (ADR 0084). It would be just as much a special case as
  the method.
- **Allowing methods for one instantiation in user code, or a numeric trait,
  now:** each widens a core generics rule (ADR 0128) for one method; a built-in
  special case is smaller and reversible.
- **`index_of` returning `-1`:** an in-band value that `T?` exists to avoid
  (ADR 0048).
