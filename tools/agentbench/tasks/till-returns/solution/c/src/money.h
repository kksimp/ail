/* Money is whole cents; reports print it as dollars (spec "Prices"). */
#ifndef MONEY_H
#define MONEY_H

/* `a × p / 100` rounded to the nearest cent, halves up. `a` is never negative. */
long long percent(long long a, long long p);

/* Writes cents as dollars with exactly two decimals (`1234.05`, `0.50`, `-0.50`) into buf; returns buf. */
char *money_show(long long c, char buf[32]);

#endif
