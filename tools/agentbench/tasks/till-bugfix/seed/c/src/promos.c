#include "promos.h"

#include <stdlib.h>
#include <string.h>

#include "grow.h"
#include "money.h"

void promos_init(struct promos *ps) {
    memset(ps, 0, sizeof *ps);
}

void promos_free(struct promos *ps) {
    free(ps->all);
    promos_init(ps);
}

struct promo *promos_find(const struct promos *ps, const char *code) {
    if (!code[0]) return NULL;
    for (size_t i = 0; i < ps->len; i++)
        if (strcmp(ps->all[i].code, code) == 0) return &ps->all[i];
    return NULL;
}

void promos_add(struct promos *ps, const struct promo *p) {
    size_t at = 0;
    while (at < ps->len && strcmp(ps->all[at].code, p->code) < 0) at++;
    GROW(ps->all, ps->len, ps->cap);
    memmove(ps->all + at + 1, ps->all + at, (ps->len - at) * sizeof *ps->all);
    ps->all[at] = *p;
    ps->len++;
}

long long promo_discount(long long amount, long long pct) {
    long long off = percent(amount, pct);
    if ((amount * pct) % 100 >= 50) return off + 1; /* halves round up */
    return off;
}
