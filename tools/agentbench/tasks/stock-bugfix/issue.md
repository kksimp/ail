# Issue: stock values drift, and error lines point at the wrong line

Two reports against `stock` (spec in `DOCS/spec.md`, sources in `src/`):

1. Values drift upwards. After receiving one SKU at two prices and shipping part of it, `report` shows a VALUE a few cents above what the spec's average-cost rule gives, and `total` repeats the error. With
   ```
   receive A 3 2
   receive A 1 4
   ship A 3
   ```
   `./stock report` prints `A	1	0.04`; by the spec it is `A	1	0.03`.
2. When the log has comments or blank lines, error messages name the wrong line. With
   ```
   # stock log
   receive A 1 1
   ship A 5
   ```
   stderr says `error: line 2: short A`; it should say line 3.

Find the causes, fix them, and keep `./test` passing. Hidden cases cover more than these examples. Do not change the spec's behaviour elsewhere.
