/* Purchase orders to suppliers: goods ordered, then received into stock. */
#ifndef PURCHASING_H
#define PURCHASING_H

#include <stddef.h>

#include "fields.h"
#include "journal.h"

enum po_status { ORDERED, RECEIVED };

const char *po_status_text(enum po_status s);

struct purchase {
    char po[FIELD];
    enum po_status status;
    struct buy *buys;
    size_t nbuys;
};

/* The units of every SKU on the purchase order. */
long long purchase_units(const struct purchase *p);
/* What the goods cost: QTY × CENTS over every SKU. */
long long purchase_cost(const struct purchase *p);

/* In the order they were entered. */
struct purchases {
    struct purchase *all;
    size_t len, cap;
};

void purchases_init(struct purchases *ps);
void purchases_free(struct purchases *ps);
struct purchase *purchases_find(const struct purchases *ps, const char *po);
/* Adds a purchase order after the others with a copy of `buys`. */
void purchases_add(struct purchases *ps, const char *po, const struct buy *buys, size_t nbuys);

#endif
