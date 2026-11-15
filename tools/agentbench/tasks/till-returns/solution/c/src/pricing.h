/* Pricing of order lines (spec "Prices"). The amounts are computed from the fields an order
   fixes when it is placed (price, tax rate, tier discount) and the order's promo. */
#ifndef PRICING_H
#define PRICING_H

#include "fields.h"

struct line {
    char sku[FIELD];
    long long qty;
    long long price;
    char cat[FIELD];
    long long rate;     /* tax rate of the category, in percent, when the order was placed */
    long long tier;     /* tier discount of the customer, in percent, when the order was placed */
    long long gross;
    long long discount; /* tier discount plus promo discount */
    long long net;
    long long tax;
    long long total;
    long long cost;     /* what the shipped units cost the shop; 0 until the order ships */
    long long returned; /* units returned so far, and the net, tax and cost they took back */
    long long rnet, rtax, rcost;
};

/* The share of `amount` that units `lo` up to `hi` of the line's qty take back (each step rounded
   down, so that returning every unit gives back exactly `amount`). */
long long line_share(const struct line *l, long long amount, long long lo, long long hi);

/* Computes the line's amounts. The promo (`promo_pct` off lines of `promo_cat`) applies only to
   lines of that category; pass "" when the order has no promo. */
void price_line(struct line *l, const char *promo_cat, long long promo_pct);

#endif
