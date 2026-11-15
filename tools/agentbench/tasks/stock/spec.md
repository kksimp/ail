# stock

`stock` is a command-line program that reads a stock log on stdin and prints a report about it.

Input: one event per line. Fields are separated by exactly one space.
- `receive SKU QTY PRICE`: QTY units arrive, each costing PRICE cents.
- `ship SKU QTY`: QTY units leave.
- `count SKU QTY`: a stocktake found QTY units on hand.

SKU is one or more of `A-Z 0-9 -`. QTY and PRICE are 1 to 5 digits (`0-9`, no sign). QTY must be at least 1 for `receive` and `ship`, and may be 0 for `count`. An empty line or a line starting with `#` is ignored, but still counts for line numbers. A trailing newline at the end of input is not an extra line.

Each SKU has an on-hand quantity and a cost (total cents), both starting at 0 the first time the SKU is received:
- `receive` adds QTY to the quantity and QTY × PRICE to the cost.
- `ship` removes QTY units at their average cost: the cost falls by `cost × QTY / quantity` (integer division, rounded down), then the quantity falls by QTY.
- `count` sets the quantity to QTY. If the old quantity was above 0, the cost becomes `cost × QTY / old quantity` (rounded down); otherwise the cost is unchanged.

A line that breaks a rule is skipped, and one line goes to stderr, where K is the 1-based line number:
- `error: line K: bad line` for a line that is not a valid event;
- `error: line K: unknown SKU` for `ship` or `count` of a SKU never received (the line names the SKU, e.g. `unknown AB-1`);
- `error: line K: short SKU` for `ship` of more units than are on hand.

Nothing else is printed to stderr, apart from the usage line below.

Commands (first argument), output on stdout:
- `report`: one line per SKU ever received, `SKU<TAB>QUANTITY<TAB>VALUE`, SKUs in ascending byte order. VALUE is the cost.
- `low N`: every SKU ever received whose quantity is below N, one per line, ascending. N is 1 to 5 digits.
- `total`: one line, the sum of every SKU's cost.

Money prints cents as dollars with exactly two decimals and a leading `-` for negatives: `1234.05`, `0.50`, `0.00`.

Exit codes: 1 for a missing, unknown or malformed command (print `usage: stock report|low N|total` to stderr; do not read stdin); 2 if any line was skipped (stdout is still printed); else 0.

Layout: the program lives in `src/`; `./build` builds `./stock`; `./test` runs the visible cases.
