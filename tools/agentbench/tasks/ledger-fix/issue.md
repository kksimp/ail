# Issue: ledger prints wrong amounts for small debits, and rejects 31st-of-month lines

`ledger` (spec in `DOCS/spec.md`; sources in `src/`) misbehaves on real exports.

1. A debit under one dollar loses its minus sign. With input
   ```
   date,account,amount_cents,memo
   2026-01-05,cash,1000,pay
   2026-01-06,food,-50,gum
   ```
   `./ledger balance` prints `food	0.50`; it should print `food	-0.50`.
2. Lines dated the 31st (`2026-01-31,cash,100,end of month`) are reported as `bad date` and skipped, so month totals are too low.

Find the causes, fix them, and keep `./test` passing. Hidden cases cover more than the examples above. Do not change the spec's behaviour elsewhere.
