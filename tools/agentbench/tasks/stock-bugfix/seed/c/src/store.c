#include "store.h"

#include <stdlib.h>
#include <string.h>

void store_init(struct store *s) {
    s->items = NULL;
    s->len = s->cap = 0;
}

void store_free(struct store *s) {
    free(s->items);
    store_init(s);
}

/* The index of sku, or of where it would be inserted; *found says which. */
static size_t find(const struct store *s, const char *sku, int *found) {
    size_t lo = 0, hi = s->len;
    while (lo < hi) {
        size_t mid = lo + (hi - lo) / 2;
        int c = strcmp(s->items[mid].sku, sku);
        if (c == 0) {
            *found = 1;
            return mid;
        }
        if (c < 0) lo = mid + 1;
        else hi = mid;
    }
    *found = 0;
    return lo;
}

static struct entry *insert(struct store *s, size_t at, const char *sku) {
    if (s->len == s->cap) {
        size_t cap = s->cap ? s->cap * 2 : 16;
        struct entry *t = realloc(s->items, cap * sizeof *t);
        if (!t) abort();
        s->items = t;
        s->cap = cap;
    }
    memmove(s->items + at + 1, s->items + at, (s->len - at) * sizeof *s->items);
    s->len++;
    struct entry *it = &s->items[at];
    memset(it, 0, sizeof *it);
    strcpy(it->sku, sku);
    return it;
}

enum problem store_apply(struct store *s, const struct event *e) {
    int found;
    size_t at = find(s, e->sku, &found);
    if (e->kind == RECEIVE) {
        struct entry *it = found ? &s->items[at] : insert(s, at, e->sku);
        it->qty += e->qty;
        it->cost += e->qty * e->price;
        return OK;
    }
    if (!found) return UNKNOWN;
    struct entry *it = &s->items[at];
    if (e->kind == SHIP) {
        if (e->qty > it->qty) return SHORT;
        it->cost -= it->cost / it->qty * e->qty;
        it->qty -= e->qty;
    } else {
        if (it->qty > 0) it->cost = it->cost * e->qty / it->qty;
        it->qty = e->qty;
    }
    return OK;
}
