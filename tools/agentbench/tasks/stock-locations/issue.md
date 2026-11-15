# Feature: storage locations

Warehouse staff now record where stock sits. Extend `stock` (spec in `DOCS/spec.md`, sources in `src/`) as follows, and keep everything else as specified there.

- `receive`, `ship` and `count` may end with one more field, a location `@LOC`, where LOC is one or more of `a-z 0-9 -`. Without it the location is `@main`.
- A new event, `move SKU QTY @FROM @TO` (QTY 1 to 5 digits, at least 1), moves units between locations. The SKU must be known (else `unknown SKU`) and @FROM must hold at least QTY units (else `short SKU`). It does not change the cost. @FROM and @TO may be the same.
- Quantities are kept per location. `ship` needs QTY units at its own location (else `short SKU`) and removes `cost × QTY / total quantity` (rounded down), where the total is over all locations. `count` sets the quantity at its location; if the SKU's old total was above 0, the cost becomes `cost × new total / old total` (rounded down).
- `report` and `low` use each SKU's total quantity over all locations.
- A new command, `where SKU`, prints one line per location holding more than 0 units of SKU, `LOC<TAB>QUANTITY` (LOC with its `@`), locations in ascending byte order. For a SKU never received it prints `error: unknown SKU` (naming the SKU) to stderr after any line errors, nothing on stdout, and exits 4, ahead of exit 2.
- The usage line becomes `usage: stock report|low N|total|where SKU`.

Keep `./test` passing; hidden cases cover the whole spec as changed here.
