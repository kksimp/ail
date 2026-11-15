/* The shop's state and the journal entries that change it (spec "Journal").
   Each entry checks its rules in the spec's order and changes nothing when one fails. */
#ifndef SHOP_H
#define SHOP_H

#include <stddef.h>

#include "catalog.h"
#include "accounts.h"
#include "gifts.h"
#include "journal.h"
#include "orders.h"
#include "promos.h"
#include "purchasing.h"

struct shop {
    struct catalog stock;
    struct accounts people;
    struct promos deals;
    struct orders book;
    struct purchases buying;
    struct cards cards;
};

void shop_init(struct shop *s);
void shop_free(struct shop *s);

/* Applies one entry. Returns 1 if it was applied; 0 if a rule rejected it, with the error
   message (without `error: line K: `) in `why`. A rejected entry changes nothing. */
int shop_apply(struct shop *s, const struct entry *e, char *why, size_t n);

#endif
