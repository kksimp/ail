/* Orders: their lines, promo, payments and status. */
#ifndef ORDERS_H
#define ORDERS_H

#include <stddef.h>

#include "fields.h"
#include "pricing.h"

enum status { OPEN, PAID, SHIPPED, CANCELLED };

/* The status as the reports print it. */
const char *status_text(enum status s);

struct order {
    char id[FIELD];
    char cust[FIELD];
    enum status status;
    struct line *lines; /* in the order the `order` entry listed them */
    size_t nlines;
    char promo[FIELD];  /* the promo code on the order, or "" */
    long long paid;     /* what has been paid on the order; never more than its total */
};

/* The sum of the line totals. */
long long order_total(const struct order *o);
/* What is still to pay. */
long long order_due(const struct order *o);
/* Whether a line of the order is in category `cat`. */
int order_has_cat(const struct order *o, const char *cat);

/* In the order they were placed. */
struct orders {
    struct order *all;
    size_t len, cap;
};

void orders_init(struct orders *os);
void orders_free(struct orders *os);
/* The order `id`, or NULL if it was never placed. */
struct order *orders_find(const struct orders *os, const char *id);
/* Adds an order after the others, taking ownership of its lines; returns it. */
struct order *orders_add(struct orders *os, const struct order *o);

#endif
