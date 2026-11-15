# Change: fractional quantities

Some goods are now sold by weight. Change `stock` (spec in `DOCS/spec.md`, sources in `src/`) so that quantities may have a fractional part, and keep everything else as specified there.

- QTY is 1 to 5 digits, optionally followed by `.` and 1 to 3 digits: `2`, `2.5`, `0.125`. The N of `low N` takes the same form. PRICE stays 1 to 5 digits of whole cents per unit.
- Keep quantities exactly, in thousandths of a unit. Do not use floating point.
- `receive` adds QTY × PRICE to the cost, rounded down to a whole cent.
- `ship` and `count` keep their formulas (`cost × QTY / quantity`, rounded down), on exact quantities.
- QTY must still be above 0 for `receive` and `ship`; `0.000` is 0.
- `report` prints every quantity with exactly three decimals: `2.500`, `10.000`, `0.125`.

Test data keeps every intermediate value within 64-bit signed integers. Keep `./test` passing; hidden cases cover the whole spec as changed here.
