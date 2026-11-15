#include "orders.h"

#include <stdlib.h>
#include <string.h>

#include "grow.h"

const char *status_text(enum status s) {
    switch (s) {
    case OPEN: return "open";
    case PAID: return "paid";
    case SHIPPED: return "shipped";
    case CANCELLED: return "cancelled";
    }
    return "?";
}

long long order_total(const struct order *o) {
    long long sum = 0;
    for (size_t i = 0; i < o->nlines; i++) sum += o->lines[i].total;
    return sum;
}

long long order_due(const struct order *o) {
    return order_total(o) - o->paid;
}

int order_has_cat(const struct order *o, const char *cat) {
    for (size_t i = 0; i < o->nlines; i++)
        if (strcmp(o->lines[i].cat, cat) == 0) return 1;
    return 0;
}

void orders_init(struct orders *os) {
    memset(os, 0, sizeof *os);
}

void orders_free(struct orders *os) {
    for (size_t i = 0; i < os->len; i++) free(os->all[i].lines);
    free(os->all);
    orders_init(os);
}

struct order *orders_find(const struct orders *os, const char *id) {
    for (size_t i = 0; i < os->len; i++)
        if (strcmp(os->all[i].id, id) == 0) return &os->all[i];
    return NULL;
}

struct order *orders_add(struct orders *os, const struct order *o) {
    GROW(os->all, os->len, os->cap);
    os->all[os->len] = *o;
    return &os->all[os->len++];
}
