#include "catalog.h"

#include <stdlib.h>
#include <string.h>

#include "grow.h"
#include "money.h"

void catalog_init(struct catalog *c) {
    memset(c, 0, sizeof *c);
}

void catalog_free(struct catalog *c) {
    free(c->items);
    free(c->rates);
    catalog_init(c);
}

/* The index of sku, or of where it would be inserted; *found says which. */
static size_t locate(const struct catalog *c, const char *sku, int *found) {
    size_t lo = 0, hi = c->len;
    while (lo < hi) {
        size_t mid = lo + (hi - lo) / 2;
        int d = strcmp(c->items[mid].sku, sku);
        if (d == 0) {
            *found = 1;
            return mid;
        }
        if (d < 0) lo = mid + 1;
        else hi = mid;
    }
    *found = 0;
    return lo;
}

struct item *catalog_find(const struct catalog *c, const char *sku) {
    int found;
    size_t at = locate(c, sku, &found);
    return found ? &c->items[at] : NULL;
}

struct item *catalog_add(struct catalog *c, const char *sku, const char *cat, long long price) {
    int found;
    size_t at = locate(c, sku, &found);
    GROW(c->items, c->len, c->cap);
    memmove(c->items + at + 1, c->items + at, (c->len - at) * sizeof *c->items);
    c->len++;
    struct item *it = &c->items[at];
    memset(it, 0, sizeof *it);
    strcpy(it->sku, sku);
    strcpy(it->cat, cat);
    it->price = price;
    return it;
}

long long item_available(const struct item *it) {
    return it->onhand - it->reserved;
}

long long catalog_rate(const struct catalog *c, const char *cat) {
    for (size_t i = 0; i < c->nrates; i++)
        if (strcmp(c->rates[i].cat, cat) == 0) return c->rates[i].pct;
    return 0;
}

void catalog_set_rate(struct catalog *c, const char *cat, long long pct) {
    for (size_t i = 0; i < c->nrates; i++)
        if (strcmp(c->rates[i].cat, cat) == 0) {
            c->rates[i].pct = pct;
            return;
        }
    GROW(c->rates, c->nrates, c->rates_cap);
    strcpy(c->rates[c->nrates].cat, cat);
    c->rates[c->nrates++].pct = pct;
}

long long item_take(struct item *it, long long qty) {
    long long cost = it->value * qty / it->onhand;
    it->value -= cost;
    it->onhand -= qty;
    it->reserved -= qty;
    return cost;
}

void item_stocktake(struct item *it, long long qty) {
    if (it->onhand > 0) it->value = it->value * qty / it->onhand;
    it->onhand = qty;
}

long long tax_on(long long net, long long rate) {
    return percent(net, rate);
}
