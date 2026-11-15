/* Registered accounts: their tier and their credit. */
#ifndef ACCOUNTS_H
#define ACCOUNTS_H

#include <stddef.h>

#include "fields.h"

struct account {
    char id[FIELD];
    char tier[FIELD];
    long long credit; /* cents the shop owes the account: overpayments and refunds of cancelled orders */
};

/* Kept in ascending byte order of id. */
struct accounts {
    struct account *all;
    size_t len, cap;
};

void accounts_init(struct accounts *cs);
void accounts_free(struct accounts *cs);

/* The account `id`, or NULL if not registered. */
struct account *accounts_find(const struct accounts *cs, const char *id);
/* Registers `id` with no credit, or changes the tier of a registered account. */
void accounts_set_tier(struct accounts *cs, const char *id, const char *tier);

/* The tier discount in percent: 0 for basic, 5 for silver, 10 for gold. */
long long tier_pct(const char *tier);
/* The tier discount on a line's gross amount. */
long long tier_discount(long long gross, long long pct);

#endif
