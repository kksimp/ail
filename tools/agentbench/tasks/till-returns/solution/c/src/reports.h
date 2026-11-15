/* The reports, printed on stdout (spec "Reports"). */
#ifndef REPORTS_H
#define REPORTS_H

#include "shop.h"

void report_orders(const struct shop *s);
/* Prints the invoice of order `id`; returns 0 (printing nothing) if it was never placed. */
int report_invoice(const struct shop *s, const char *id);
void report_stock(const struct shop *s);
void report_low(const struct shop *s, long long n);
void report_customers(const struct shop *s);
/* Prints a customer's orders and credit; returns 0 (printing nothing) for a customer never registered. */
int report_history(const struct shop *s, const char *cust);
void report_sales(const struct shop *s);
void report_margin(const struct shop *s);
void report_purchases(const struct shop *s);
void report_gifts(const struct shop *s);
void report_promos(const struct shop *s);

#endif
