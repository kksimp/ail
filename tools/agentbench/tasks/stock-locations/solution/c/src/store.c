#include "store.h"

#include <stdlib.h>
#include <string.h>

void store_init(struct store *s) {
    s->items = NULL;
    s->len = s->cap = 0;
}

void store_free(struct store *s) {
    for (size_t i = 0; i < s->len; i++) free(s->items[i].at);
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

long long entry_qty(const struct entry *it) {
    long long sum = 0;
    for (size_t i = 0; i < it->nat; i++) sum += it->at[i].qty;
    return sum;
}

/* The units at loc, creating the location (with 0 units) if it is new. */
static long long *place(struct entry *it, const char *loc) {
    size_t i = 0;
    while (i < it->nat && strcmp(it->at[i].loc, loc) < 0) i++;
    if (i < it->nat && !strcmp(it->at[i].loc, loc)) return &it->at[i].qty;
    if (it->nat == it->capat) {
        size_t cap = it->capat ? it->capat * 2 : 4;
        struct place *t = realloc(it->at, cap * sizeof *t);
        if (!t) abort();
        it->at = t;
        it->capat = cap;
    }
    memmove(it->at + i + 1, it->at + i, (it->nat - i) * sizeof *it->at);
    it->nat++;
    strcpy(it->at[i].loc, loc);
    it->at[i].qty = 0;
    return &it->at[i].qty;
}

const struct entry *store_get(const struct store *s, const char *sku) {
    int found;
    size_t at = find(s, sku, &found);
    return found ? &s->items[at] : NULL;
}

enum problem store_apply(struct store *s, const struct event *e) {
    int found;
    size_t at = find(s, e->sku, &found);
    if (e->kind == RECEIVE) {
        struct entry *it = found ? &s->items[at] : insert(s, at, e->sku);
        *place(it, e->loc) += e->qty;
        it->cost += e->qty * e->price;
        return OK;
    }
    if (!found) return UNKNOWN;
    struct entry *it = &s->items[at];
    long long total = entry_qty(it);
    long long *here = place(it, e->loc);
    if (e->kind == SHIP) {
        if (e->qty > *here) return SHORT;
        it->cost -= it->cost * e->qty / total;
        *here -= e->qty;
    } else if (e->kind == MOVE) {
        if (e->qty > *here) return SHORT;
        *here -= e->qty;
        *place(it, e->to) += e->qty; /* may move the array: `here` is not used after this */
    } else {
        long long new_total = total - *here + e->qty;
        if (total > 0) it->cost = it->cost * new_total / total;
        *here = e->qty;
    }
    return OK;
}
