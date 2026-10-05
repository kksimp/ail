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

Candidates to measure next: field-init shorthand (`ParseError(line, col,
msg)` when locals share the field names), positional construction of
one-field variants, and `else` as an expression (OQ-040, −38% on the sheet's
aggregate arms).
