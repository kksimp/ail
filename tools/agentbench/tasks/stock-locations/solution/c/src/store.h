/* Quantities per location and average cost per SKU. */
#ifndef STORE_H
#define STORE_H

#include <stddef.h>

#include "event.h"

struct place {
    char loc[32];
    long long qty;
};

struct entry {
    char sku[64];
    struct place *at; /* units per location, in ascending byte order of location */
    size_t nat, capat;
    long long cost;
};

/* Units of an entry over all locations. */
long long entry_qty(const struct entry *it);

/* Entries are kept in ascending byte order of SKU. */
struct store {
    struct entry *items;
    size_t len, cap;
};

enum problem { OK, UNKNOWN, SHORT };

void store_init(struct store *s);
void store_free(struct store *s);

/* Applies one event; a rejected event changes nothing and returns UNKNOWN or SHORT. */
enum problem store_apply(struct store *s, const struct event *e);

/* The entry of sku, or NULL for a SKU never received. */
const struct entry *store_get(const struct store *s, const char *sku);

#endif
