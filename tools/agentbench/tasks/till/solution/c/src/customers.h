/* Registered customers: their tier and their credit. */
#ifndef CUSTOMERS_H
#define CUSTOMERS_H

#include <stddef.h>

#include "fields.h"

struct customer {
    char id[FIELD];
    char tier[FIELD];
    long long credit; /* cents the shop owes the customer: overpayments and refunds of cancelled orders */
};

/* Kept in ascending byte order of id. */
struct customers {
    struct customer *all;
    size_t len, cap;
};

void customers_init(struct customers *cs);
void customers_free(struct customers *cs);

/* The customer `id`, or NULL if not registered. */
struct customer *customers_find(const struct customers *cs, const char *id);
/* Registers `id` with no credit, or changes the tier of a registered customer. */
void customers_set_tier(struct customers *cs, const char *id, const char *tier);

/* The tier discount in percent: 0 for basic, 5 for silver, 10 for gold. */
long long tier_pct(const char *tier);
/* The tier discount on a line's gross amount. */
long long tier_discount(long long gross, long long pct);

#endif
