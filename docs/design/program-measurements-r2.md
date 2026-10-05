# Program measurements, round 2: three 500–800 line programs

> **Non-normative design evidence,** 2026-10-05. The first measurement on
> programs large enough to need maps, sets, floats, characters, multi-line
> strings and constants (ADRs 0130–0140).

## Programs

Three programs were written first in Python, then in AIL, and run in the
bootstrap with output byte-identical to Python's. They are now
`examples/interp.ail`, `examples/json.ail` and `examples/sheet.ail`:
- `interp`: a tokenizer, parser and tree-walking evaluator for a small
  scripting language, with 16 embedded test scripts;
- `json`: a JSON parser, pretty-printer and path query tool over an embedded
  document;
- `sheet`: a spreadsheet engine with formulas, dependency tracking,
  recalculation, cycle detection and table output.

Go, Rust and TypeScript ports were written idiomatically from the Python
versions, formatted with each language's standard formatter, and run; all
match the expected output byte for byte. Sources, with comments removed, are
in `tools/tokbench/corpus/programs/{interp,json,sheet}/`. Counts are the mean
of five tokenizers (`tools/tokbench/run.sh`).

## Results

| Language | interp | json | sheet | Total | AIL vs. this language |
|---|---|---|---|---|---|
| **AIL** | **5978.6** | **3672.4** | **5151.8** | **14802.8** | — |
| Python | 5565.2 | 3233.2 | 4674.0 | 13472.4 | +10% |
| TypeScript | 6221.4 | 3684.2 | 5498.8 | 15404.4 | −4% |
| Rust | 7137.4 | 4336.4 | 6564.2 | 18038.0 | −18% |
| Go | 7452.0 | 4512.2 | 6466.8 | 18431.0 | −20% |

The first AIL drafts, written before these features existed, measured
15442.8 (+15% over Python). Rewriting them with one-line arms, or-patterns,
raw strings, `while true`, open slices and statement `else` saved 4.1%.

## Reading the numbers

AIL stays well ahead of the statically typed languages, as in round 1, but is
behind Python here, where round 1's small programs were 16% ahead of
type-hinted Python. The Python versions here are mostly unannotated (`json`
has no annotations at all), so the comparison is with dynamic Python, not
with type-hinted Python.

Where AIL spends more than Python (from the ports' and drafts' analyses):
- **Declared data shapes.** Python's `json` stores values as native `dict`,
  `list`, `str` and numbers; AIL declares `enum Json` with seven variants and
  builds them by name (`Str(s: text)`). The other programs' AST and value
  types cost the same way in every typed language.
- **Named constructor arguments.** `ParseError(line: line, col: col, msg: msg)`
  against Python's positional `ParseError(line, col, msg)`.
- **Typed signatures** on every function and method.
- **Absence handling at each use.** `c := self.peek() else return ...` where
  Python compares `peek()` with `""`.

Where AIL saves against the typed languages: no braces or `;`, errors
propagated with `?` (Go spends 13–24 `if err != nil` blocks per program),
bare variants, enums with data and `match` without per-arm boilerplate, and
`"{x}"` interpolation.

## Python parity round

Each candidate was applied to the three AIL programs by a script and
measured (tokens; 14,802.8 before):

| Change | Tokens | Change | Decision |
|---|---|---|---|
| Bare fields and methods inside methods (`peek()` for `self.peek()`) | 14,205.6 | −4.0% | adopted, [ADR 0141](../decisions/0141-implicit-self.md) |
| Bare fields only | 14,618.4 | −1.2% | rejected |
| Positional constructor arguments, always | 14,359.2 | −3.0% | rejected (silent swaps) |
| Positional when the fields filled have different types, plus shorthand | 14,375.4 | −2.9% | adopted, [ADR 0142](../decisions/0142-positional-constructor-arguments.md) |
| Shorthand only | 14,641.8 | −1.1% | rejected |
| `[]` and `{}` typed by their first use | 14,681.4 | −0.8% | adopted, [ADR 0143](../decisions/0143-empty-literals-typed-by-use.md) |
| `!T` instead of `T ! E` on private functions | 14,754.4 | −0.3% | style only |
| No `mut` on local declarations | 14,746.8 | −0.4% | rejected (ADR 0038) |
| An `in` operator for `contains` | 14,786.8 | −0.1% | tie |
| Methods grouped in a type block | 14,797.8 | −0.0% | tie |

With the three adopted changes together, the programs measure **13,658.2:
1.4% above Python** (a tie under ADR 0076's ~2% rule), 11% below TypeScript,
24% below Rust and 26% below Go. These figures come from scripted rewrites;
they are re-measured on the examples once the bootstrap implements the
changes.
