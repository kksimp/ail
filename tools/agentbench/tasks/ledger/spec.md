# ledger

Write `ledger`, a command-line program. It reads a CSV of transactions from stdin and prints a report.

Input: the first line is exactly `date,account,amount_cents,memo`. Every other line has exactly four comma-separated fields (no quoting; memo may be empty and has no commas). `date` is `YYYY-MM-DD` (digits only, month 01-12, day 01-31). `account` is non-empty. `amount_cents` is an integer with an optional leading `-`. A line that breaks any rule is skipped and reported on stderr as `error: line K: REASON` (K is the 1-based line number, header is line 1). If any line was skipped the exit code is 2, else 0, but stdout is still printed. A bad header prints `error: line 1: bad header`, nothing on stdout, exit 2. Empty input is a bad header. A trailing blank line at the end of input is ignored.

Commands (first argument):
- `balance`: one line per account, `ACCOUNT<TAB>AMOUNT`, accounts in ascending byte order.
- `top N`: the N transactions with the largest absolute amount, one per line `DATE<TAB>ACCOUNT<TAB>AMOUNT<TAB>MEMO`, largest first, ties in input order. Fewer than N lines if fewer exist. N is a non-negative integer.
- `monthly`: one line per `YYYY-MM`, `MONTH<TAB>AMOUNT`, months ascending.

AMOUNT prints cents as dollars with exactly two decimals and a leading `-` for negatives: `1234.05`, `-0.50`, `0.00`. Amounts fit in 64-bit signed integers; sums may be large.

A missing, unknown or malformed command (including a bad N) prints `usage: ledger balance|top N|monthly` on stderr and exits 1 without reading stdin.

Layout: sources live in `src/` (`src/main.ail`, `src/main.c` or `src/main.rs`); `./build` builds `./ledger`; `./test` runs the visible cases.
