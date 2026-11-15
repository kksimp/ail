/* The catalog: each SKU's category, price and stock, and each category's tax rate. */
#ifndef CATALOG_H
#define CATALOG_H

#include <stddef.h>

#include "fields.h"

struct item {
    char sku[FIELD];
    char cat[FIELD];
    long long price;
    long long onhand;
    long long reserved; /* units held by orders that are open or paid */
    long long value;    /* what the units on hand cost the shop, in cents (average cost) */
};

struct rate {
    char cat[FIELD];
    long long pct;
};

/* Items are kept in ascending byte order of SKU. */
struct catalog {
    struct item *items;
    size_t len, cap;
    struct rate *rates;
    size_t nrates, rates_cap;
};

void catalog_init(struct catalog *c);
void catalog_free(struct catalog *c);

/* The item of a SKU, or NULL if it is not in the catalog. */
struct item *catalog_find(const struct catalog *c, const char *sku);
/* Adds a new SKU with nothing on hand; returns it. */
struct item *catalog_add(struct catalog *c, const char *sku, const char *cat, long long price);

/* Units on hand that no order holds. */
long long item_available(const struct item *it);

/* The tax rate of a category in percent; 0 until a `tax` entry sets it. */
long long catalog_rate(const struct catalog *c, const char *cat);
void catalog_set_rate(struct catalog *c, const char *cat, long long pct);

/* Ships `qty` reserved units: they leave both the on-hand and the reserved quantity, taking their
   share of the stock value. Returns that share, the cost of the units. */
long long item_take(struct item *it, long long qty);
/* A stocktake: sets the units on hand, scaling the stock value with them. */
void item_stocktake(struct item *it, long long qty);

/* The tax on a line's net amount at `rate` percent. */
long long tax_on(long long net, long long rate);

#endif
