/* One line of the journal as an entry (spec "Journal"). Only the syntax is checked here;
   the rules that need the shop's state are in shop.c. */
#ifndef JOURNAL_H
#define JOURNAL_H

#include <stddef.h>

#include "fields.h"

/* One SKU:QTY of an `order` entry. */
struct want {
    char sku[FIELD];
    long long qty;
};

/* One SKU:QTY@CENTS of a `buy` entry. */
struct buy {
    char sku[FIELD];
    long long qty;
    long long cents;
};

enum entry_kind {
    E_ITEM, E_TAX, E_RESTOCK, E_ADJUST, E_CUSTOMER, E_PROMO, E_PURCHASE, E_RECEIVE, E_GIFT,
    E_ORDER, E_APPLY, E_PAY, E_PAY_CREDIT, E_PAY_GIFT, E_SHIP, E_CANCEL, E_RETURN
};

/* The fields an entry uses; the others are empty or 0. */
struct entry {
    enum entry_kind kind;
    char sku[FIELD], cat[FIELD], cust[FIELD], tier[FIELD], code[FIELD];
    char id[FIELD];   /* ORDER */
    char po[FIELD];
    char card[FIELD];
    long long cents, qty, pct, uses;
    struct want *wants; /* `order` */
    size_t nwants;
    struct buy *buys;   /* `buy` */
    size_t nbuys;
};

/* Parses one line (without its newline) into *e: 1 for a valid entry, 0 for a bad line.
   A valid entry must be released with entry_free. */
int journal_parse(const char *line, struct entry *e);
void entry_free(struct entry *e);

#endif
