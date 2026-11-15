#include "journal.h"

#include <stdlib.h>
#include <string.h>

#include "grow.h"

#define MAXF 8

/* Splits `line` at single spaces into at most MAXF fields of fewer than 4096 bytes each.
   Returns the number of fields, or -1 if there are more or one is too long. */
static int split(const char *line, char f[MAXF][4096]) {
    int n = 0;
    const char *p = line;
    for (;;) {
        const char *sp = strchr(p, ' ');
        size_t len = sp ? (size_t)(sp - p) : strlen(p);
        if (n == MAXF || len >= 4096) return -1;
        memcpy(f[n], p, len);
        f[n][len] = 0;
        n++;
        if (!sp) return n;
        p = sp + 1;
    }
}

/* A number of 1 to `width` digits that is at least `least`. */
static int num(const char *s, int width, long long least, long long *out) {
    return digits(s, width, out) && *out >= least;
}

static void copy(char dst[FIELD], const char *src) {
    strncpy(dst, src, FIELD - 1);
    dst[FIELD - 1] = 0;
}

/* The items of an `order` entry: SKU:QTY separated by commas, no SKU twice. */
static int parse_wants(const char *s, struct entry *e) {
    size_t cap = 0;
    const char *p = s;
    for (;;) {
        const char *comma = strchr(p, ',');
        size_t len = comma ? (size_t)(comma - p) : strlen(p);
        char part[256];
        if (len >= sizeof part) return 0;
        memcpy(part, p, len);
        part[len] = 0;
        char *colon = strchr(part, ':');
        if (!colon || strchr(colon + 1, ':')) return 0;
        *colon = 0;
        struct want w;
        if (!is_sku(part) || !num(colon + 1, 5, 1, &w.qty)) return 0;
        for (size_t i = 0; i < e->nwants; i++)
            if (strcmp(e->wants[i].sku, part) == 0) return 0;
        copy(w.sku, part);
        GROW(e->wants, e->nwants, cap);
        e->wants[e->nwants++] = w;
        if (!comma) return 1;
        p = comma + 1;
    }
}

/* The goods of a `buy` entry: SKU:QTY@CENTS separated by commas, no SKU twice. */
static int parse_buys(const char *s, struct entry *e) {
    size_t cap = 0;
    const char *p = s;
    for (;;) {
        const char *comma = strchr(p, ',');
        size_t len = comma ? (size_t)(comma - p) : strlen(p);
        char part[256];
        if (len >= sizeof part) return 0;
        memcpy(part, p, len);
        part[len] = 0;
        char *at = strchr(part, '@');
        if (!at || strchr(at + 1, '@')) return 0;
        *at = 0;
        char *colon = strchr(part, ':');
        if (!colon || strchr(colon + 1, ':')) return 0;
        *colon = 0;
        struct buy b;
        if (!is_sku(part) || !num(colon + 1, 5, 1, &b.qty) || !num(at + 1, 7, 0, &b.cents)) return 0;
        for (size_t i = 0; i < e->nbuys; i++)
            if (strcmp(e->buys[i].sku, part) == 0) return 0;
        copy(b.sku, part);
        GROW(e->buys, e->nbuys, cap);
        e->buys[e->nbuys++] = b;
        if (!comma) return 1;
        p = comma + 1;
    }
}

static int parse_fields(char f[MAXF][4096], int n, struct entry *e) {
    const char *kw = f[0];
    if (strcmp(kw, "item") == 0 && n == 4 && is_sku(f[1]) && is_cat(f[2])) {
        e->kind = E_ITEM;
        copy(e->sku, f[1]);
        copy(e->cat, f[2]);
        return num(f[3], 7, 0, &e->cents);
    }
    if (strcmp(kw, "tax") == 0 && n == 3 && is_cat(f[1])) {
        e->kind = E_TAX;
        copy(e->cat, f[1]);
        return num(f[2], 2, 0, &e->pct);
    }
    if (strcmp(kw, "restock") == 0 && n == 3 && is_sku(f[1])) {
        e->kind = E_RESTOCK;
        copy(e->sku, f[1]);
        return num(f[2], 5, 1, &e->qty);
    }
    if (strcmp(kw, "adjust") == 0 && n == 3 && is_sku(f[1])) {
        e->kind = E_ADJUST;
        copy(e->sku, f[1]);
        return num(f[2], 5, 0, &e->qty);
    }
    if (strcmp(kw, "account") == 0 && n == 3 && is_id(f[1]) && is_tier(f[2])) {
        e->kind = E_ACCOUNT;
        copy(e->account, f[1]);
        copy(e->tier, f[2]);
        return 1;
    }
    if (strcmp(kw, "promo") == 0 && n == 5 && is_code(f[1]) && is_cat(f[2])) {
        e->kind = E_PROMO;
        copy(e->code, f[1]);
        copy(e->cat, f[2]);
        return num(f[3], 2, 0, &e->pct) && num(f[4], 3, 1, &e->uses);
    }
    if (strcmp(kw, "buy") == 0 && n == 3 && is_po(f[1])) {
        e->kind = E_PURCHASE;
        copy(e->po, f[1]);
        return parse_buys(f[2], e);
    }
    if (strcmp(kw, "receive") == 0 && n == 2 && is_po(f[1])) {
        e->kind = E_RECEIVE;
        copy(e->po, f[1]);
        return 1;
    }
    if (strcmp(kw, "gift") == 0 && n == 3 && is_card(f[1])) {
        e->kind = E_GIFT;
        copy(e->card, f[1]);
        return num(f[2], 7, 1, &e->cents);
    }
    if (strcmp(kw, "order") == 0 && n == 4 && is_order(f[1]) && is_id(f[2])) {
        e->kind = E_ORDER;
        copy(e->id, f[1]);
        copy(e->account, f[2]);
        return parse_wants(f[3], e);
    }
    if (strcmp(kw, "apply") == 0 && n == 3 && is_order(f[1]) && is_code(f[2])) {
        e->kind = E_APPLY;
        copy(e->id, f[1]);
        copy(e->code, f[2]);
        return 1;
    }
    if (strcmp(kw, "pay") == 0 && n == 3 && is_order(f[1])) {
        copy(e->id, f[1]);
        if (strcmp(f[2], "credit") == 0) {
            e->kind = E_PAY_CREDIT;
            return 1;
        }
        if (is_card(f[2])) {
            e->kind = E_PAY_GIFT;
            copy(e->card, f[2]);
            return 1;
        }
        e->kind = E_PAY;
        return num(f[2], 7, 1, &e->cents);
    }
    if (strcmp(kw, "ship") == 0 && n == 2 && is_order(f[1])) {
        e->kind = E_SHIP;
        copy(e->id, f[1]);
        return 1;
    }
    if (strcmp(kw, "cancel") == 0 && n == 2 && is_order(f[1])) {
        e->kind = E_CANCEL;
        copy(e->id, f[1]);
        return 1;
    }
    return 0;
}

int journal_parse(const char *line, struct entry *e) {
    memset(e, 0, sizeof *e);
    char f[MAXF][4096];
    int n = split(line, f);
    if (n > 0 && parse_fields(f, n, e)) return 1;
    entry_free(e);
    return 0;
}

void entry_free(struct entry *e) {
    free(e->wants);
    free(e->buys);
    e->wants = NULL;
    e->buys = NULL;
    e->nwants = e->nbuys = 0;
}
