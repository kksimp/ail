#include "accounts.h"

#include <stdlib.h>
#include <string.h>

#include "grow.h"
#include "money.h"

void accounts_init(struct accounts *cs) {
    memset(cs, 0, sizeof *cs);
}

void accounts_free(struct accounts *cs) {
    free(cs->all);
    accounts_init(cs);
}

struct account *accounts_find(const struct accounts *cs, const char *id) {
    for (size_t i = 0; i < cs->len; i++)
        if (strcmp(cs->all[i].id, id) == 0) return &cs->all[i];
    return NULL;
}

void accounts_set_tier(struct accounts *cs, const char *id, const char *tier) {
    struct account *c = accounts_find(cs, id);
    if (!c) {
        size_t at = 0;
        while (at < cs->len && strcmp(cs->all[at].id, id) < 0) at++;
        GROW(cs->all, cs->len, cs->cap);
        memmove(cs->all + at + 1, cs->all + at, (cs->len - at) * sizeof *cs->all);
        cs->len++;
        c = &cs->all[at];
        memset(c, 0, sizeof *c);
        strcpy(c->id, id);
    }
    strcpy(c->tier, tier);
}

long long tier_pct(const char *tier) {
    if (strcmp(tier, "gold") == 0) return 10;
    if (strcmp(tier, "silver") == 0) return 5;
    return 0;
}

long long tier_discount(long long gross, long long pct) {
    return percent(gross, pct);
}
