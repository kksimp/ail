/* On-hand quantities and average cost per SKU. */
#ifndef STORE_H
#define STORE_H

#include <stddef.h>

#include "event.h"

struct entry {
    char sku[64];
    long long qty;
    long long cost;
};

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

#endif
