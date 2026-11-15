/* Gift cards: sold with a balance that payments draw down. */
#ifndef GIFTS_H
#define GIFTS_H

#include <stddef.h>

#include "fields.h"

struct card {
    char card[FIELD];
    long long sold;
    long long balance;
};

/* Kept in ascending byte order of card. */
struct cards {
    struct card *all;
    size_t len, cap;
};

void cards_init(struct cards *cs);
void cards_free(struct cards *cs);
struct card *cards_find(const struct cards *cs, const char *card);
void cards_sell(struct cards *cs, const char *card, long long cents);

#endif
