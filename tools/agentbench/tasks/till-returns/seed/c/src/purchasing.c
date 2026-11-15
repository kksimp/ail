#include "purchasing.h"

#include <stdlib.h>
#include <string.h>

#include "grow.h"

const char *po_status_text(enum po_status s) {
    return s == ORDERED ? "ordered" : "received";
}

long long purchase_units(const struct purchase *p) {
    long long sum = 0;
    for (size_t i = 0; i < p->nbuys; i++) sum += p->buys[i].qty;
    return sum;
}

long long purchase_cost(const struct purchase *p) {
    long long sum = 0;
    for (size_t i = 0; i < p->nbuys; i++) sum += p->buys[i].qty * p->buys[i].cents;
    return sum;
}

void purchases_init(struct purchases *ps) {
    memset(ps, 0, sizeof *ps);
}

void purchases_free(struct purchases *ps) {
    for (size_t i = 0; i < ps->len; i++) free(ps->all[i].buys);
    free(ps->all);
    purchases_init(ps);
}

struct purchase *purchases_find(const struct purchases *ps, const char *po) {
    for (size_t i = 0; i < ps->len; i++)
        if (strcmp(ps->all[i].po, po) == 0) return &ps->all[i];
    return NULL;
}

void purchases_add(struct purchases *ps, const char *po, const struct buy *buys, size_t nbuys) {
    GROW(ps->all, ps->len, ps->cap);
    struct purchase *p = &ps->all[ps->len++];
    memset(p, 0, sizeof *p);
    strcpy(p->po, po);
    p->status = ORDERED;
    p->buys = malloc(nbuys * sizeof *buys);
    if (!p->buys) abort();
    memcpy(p->buys, buys, nbuys * sizeof *buys);
    p->nbuys = nbuys;
}
