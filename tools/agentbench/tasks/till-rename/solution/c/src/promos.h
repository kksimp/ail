/* Promo codes: a percentage off the lines of one category, on a limited number of orders. */
#ifndef PROMOS_H
#define PROMOS_H

#include <stddef.h>

#include "fields.h"

struct promo {
    char code[FIELD];
    char cat[FIELD];
    long long pct;
    long long uses;
    long long used; /* orders the code is on now; a cancelled order gives its use back */
};

/* Kept in ascending byte order of code. */
struct promos {
    struct promo *all;
    size_t len, cap;
};

void promos_init(struct promos *ps);
void promos_free(struct promos *ps);
struct promo *promos_find(const struct promos *ps, const char *code);
void promos_add(struct promos *ps, const struct promo *p);

/* The promo discount on what is left of a line after its tier discount. */
long long promo_discount(long long amount, long long pct);

#endif
