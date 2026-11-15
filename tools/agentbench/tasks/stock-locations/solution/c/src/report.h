/* The three outputs, written to stdout. */
#ifndef REPORT_H
#define REPORT_H

#include <stddef.h>

#include "store.h"

/* Cents as dollars with two decimals: "1234.05", "-0.50". */
void report_money(char *out, size_t n, long long cents);

void report_report(const struct store *s);
void report_low(const struct store *s, long long n);
void report_total(const struct store *s);

/* The locations holding units of sku. Returns 0 (printing nothing) for a SKU never received. */
int report_where(const struct store *s, const char *sku);

#endif
