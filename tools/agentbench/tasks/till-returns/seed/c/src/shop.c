#include "shop.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

void shop_init(struct shop *s) {
    catalog_init(&s->stock);
    customers_init(&s->people);
    promos_init(&s->deals);
    orders_init(&s->book);
    purchases_init(&s->buying);
    cards_init(&s->cards);
}

void shop_free(struct shop *s) {
    catalog_free(&s->stock);
    customers_free(&s->people);
    promos_free(&s->deals);
    orders_free(&s->book);
    purchases_free(&s->buying);
    cards_free(&s->cards);
}

/* Each handler returns 1 when applied, or 0 after writing the error message. */
#define FAIL(...) return (snprintf(why, n, __VA_ARGS__), 0)

static int item(struct shop *s, const struct entry *e, char *why, size_t n) {
    struct item *it = catalog_find(&s->stock, e->sku);
    if (!it) {
        catalog_add(&s->stock, e->sku, e->cat, e->cents);
        return 1;
    }
    if (strcmp(it->cat, e->cat) != 0) FAIL("category changed %s", e->sku);
    it->price = e->cents;
    return 1;
}

static int restock(struct shop *s, const struct entry *e, char *why, size_t n) {
    struct item *it = catalog_find(&s->stock, e->sku);
    if (!it) FAIL("unknown item %s", e->sku);
    it->onhand += e->qty;
    return 1;
}

static int adjust(struct shop *s, const struct entry *e, char *why, size_t n) {
    struct item *it = catalog_find(&s->stock, e->sku);
    if (!it) FAIL("unknown item %s", e->sku);
    if (e->qty < it->reserved) FAIL("below reserved %s", e->sku);
    item_stocktake(it, e->qty);
    return 1;
}

static int promo(struct shop *s, const struct entry *e, char *why, size_t n) {
    if (promos_find(&s->deals, e->code)) FAIL("duplicate promo %s", e->code);
    struct promo p;
    memset(&p, 0, sizeof p);
    strcpy(p.code, e->code);
    strcpy(p.cat, e->cat);
    p.pct = e->pct;
    p.uses = e->uses;
    promos_add(&s->deals, &p);
    return 1;
}

static int purchase(struct shop *s, const struct entry *e, char *why, size_t n) {
    if (purchases_find(&s->buying, e->po)) FAIL("duplicate purchase %s", e->po);
    for (size_t i = 0; i < e->nbuys; i++)
        if (!catalog_find(&s->stock, e->buys[i].sku)) FAIL("unknown item %s", e->buys[i].sku);
    purchases_add(&s->buying, e->po, e->buys, e->nbuys);
    return 1;
}

static int receive(struct shop *s, const struct entry *e, char *why, size_t n) {
    struct purchase *p = purchases_find(&s->buying, e->po);
    if (!p) FAIL("unknown purchase %s", e->po);
    if (p->status != ORDERED) FAIL("purchase %s is %s", e->po, po_status_text(p->status));
    for (size_t i = 0; i < p->nbuys; i++) {
        struct item *it = catalog_find(&s->stock, p->buys[i].sku);
        it->onhand += p->buys[i].qty;
        it->value += p->buys[i].qty * p->buys[i].cents;
    }
    p->status = RECEIVED;
    return 1;
}

static int gift(struct shop *s, const struct entry *e, char *why, size_t n) {
    if (cards_find(&s->cards, e->card)) FAIL("duplicate gift %s", e->card);
    cards_sell(&s->cards, e->card, e->cents);
    return 1;
}

/* Prices every line of the order again under its promo, if it has one. */
static void reprice(struct shop *s, struct order *o) {
    struct promo *p = promos_find(&s->deals, o->promo);
    for (size_t i = 0; i < o->nlines; i++) price_line(&o->lines[i], p ? p->cat : "", p ? p->pct : 0);
}

static int order(struct shop *s, const struct entry *e, char *why, size_t n) {
    if (orders_find(&s->book, e->id)) FAIL("duplicate order %s", e->id);
    struct customer *c = customers_find(&s->people, e->cust);
    if (!c) FAIL("unknown customer %s", e->cust);
    for (size_t i = 0; i < e->nwants; i++)
        if (!catalog_find(&s->stock, e->wants[i].sku)) FAIL("unknown item %s", e->wants[i].sku);
    for (size_t i = 0; i < e->nwants; i++)
        if (item_available(catalog_find(&s->stock, e->wants[i].sku)) < e->wants[i].qty)
            FAIL("short %s", e->wants[i].sku);
    struct order o;
    memset(&o, 0, sizeof o);
    strcpy(o.id, e->id);
    strcpy(o.cust, e->cust);
    o.status = OPEN;
    o.lines = calloc(e->nwants, sizeof *o.lines);
    if (!o.lines) abort();
    o.nlines = e->nwants;
    long long tier = tier_pct(c->tier);
    for (size_t i = 0; i < e->nwants; i++) {
        struct item *it = catalog_find(&s->stock, e->wants[i].sku);
        struct line *l = &o.lines[i];
        strcpy(l->sku, it->sku);
        strcpy(l->cat, it->cat);
        l->qty = e->wants[i].qty;
        l->price = it->price;
        l->rate = catalog_rate(&s->stock, it->cat);
        l->tier = tier;
        it->reserved += l->qty;
    }
    reprice(s, &o);
    orders_add(&s->book, &o);
    return 1;
}

/* The order `id` if it has status `want`; NULL after writing the error message otherwise. */
static struct order *order_in(struct shop *s, const char *id, enum status want, char *why, size_t n) {
    struct order *o = orders_find(&s->book, id);
    if (!o) {
        snprintf(why, n, "unknown order %s", id);
        return NULL;
    }
    if (o->status != want) {
        snprintf(why, n, "order %s is %s", id, status_text(o->status));
        return NULL;
    }
    return o;
}

static int apply_promo(struct shop *s, const struct entry *e, char *why, size_t n) {
    struct order *o = order_in(s, e->id, OPEN, why, n);
    if (!o) return 0;
    struct promo *p = promos_find(&s->deals, e->code);
    if (!p) FAIL("unknown promo %s", e->code);
    if (o->promo[0]) FAIL("promo already on %s", e->id);
    if (p->used >= p->uses) FAIL("promo %s used up", e->code);
    if (!order_has_cat(o, p->cat)) FAIL("promo %s does not apply", e->code);
    p->used++;
    strcpy(o->promo, e->code);
    reprice(s, o);
    long long total = order_total(o);
    if (o->paid >= total) {
        customers_find(&s->people, o->cust)->credit += o->paid - total;
        o->paid = total;
        o->status = PAID;
    }
    return 1;
}

/* Counts `amount` towards what is due on `o`; any amount above it goes to the customer's credit. */
static void settle(struct shop *s, struct order *o, long long amount) {
    long long due = order_due(o);
    long long applied = amount < due ? amount : due;
    o->paid += applied;
    customers_find(&s->people, o->cust)->credit += amount - applied;
    if (o->paid == order_total(o)) o->status = PAID;
}

static int pay(struct shop *s, const struct entry *e, char *why, size_t n) {
    struct order *o = order_in(s, e->id, OPEN, why, n);
    if (!o) return 0;
    settle(s, o, e->cents);
    return 1;
}

static int pay_credit(struct shop *s, const struct entry *e, char *why, size_t n) {
    struct order *o = order_in(s, e->id, OPEN, why, n);
    if (!o) return 0;
    struct customer *c = customers_find(&s->people, o->cust);
    if (c->credit == 0) FAIL("no credit %s", o->cust);
    long long due = order_due(o);
    long long amount = c->credit < due ? c->credit : due;
    c->credit -= amount;
    settle(s, o, amount);
    return 1;
}

static int pay_gift(struct shop *s, const struct entry *e, char *why, size_t n) {
    struct order *o = order_in(s, e->id, OPEN, why, n);
    if (!o) return 0;
    struct card *c = cards_find(&s->cards, e->card);
    if (!c) FAIL("unknown gift %s", e->card);
    if (c->balance == 0) FAIL("gift %s is empty", e->card);
    long long due = order_due(o);
    long long amount = c->balance < due ? c->balance : due;
    c->balance -= amount;
    settle(s, o, amount);
    return 1;
}

static int ship(struct shop *s, const struct entry *e, char *why, size_t n) {
    struct order *o = order_in(s, e->id, PAID, why, n);
    if (!o) return 0;
    for (size_t i = 0; i < o->nlines; i++)
        o->lines[i].cost = item_take(catalog_find(&s->stock, o->lines[i].sku), o->lines[i].qty);
    o->status = SHIPPED;
    return 1;
}

static int cancel(struct shop *s, const struct entry *e, char *why, size_t n) {
    struct order *o = orders_find(&s->book, e->id);
    if (!o) FAIL("unknown order %s", e->id);
    if (o->status == SHIPPED || o->status == CANCELLED) FAIL("order %s is %s", e->id, status_text(o->status));
    for (size_t i = 0; i < o->nlines; i++) catalog_find(&s->stock, o->lines[i].sku)->reserved -= o->lines[i].qty;
    customers_find(&s->people, o->cust)->credit += o->paid;
    o->paid = 0;
    if (o->promo[0]) promos_find(&s->deals, o->promo)->used--;
    o->status = CANCELLED;
    return 1;
}

int shop_apply(struct shop *s, const struct entry *e, char *why, size_t n) {
    switch (e->kind) {
    case E_ITEM: return item(s, e, why, n);
    case E_TAX: catalog_set_rate(&s->stock, e->cat, e->pct); return 1;
    case E_RESTOCK: return restock(s, e, why, n);
    case E_ADJUST: return adjust(s, e, why, n);
    case E_CUSTOMER: customers_set_tier(&s->people, e->cust, e->tier); return 1;
    case E_PROMO: return promo(s, e, why, n);
    case E_PURCHASE: return purchase(s, e, why, n);
    case E_RECEIVE: return receive(s, e, why, n);
    case E_GIFT: return gift(s, e, why, n);
    case E_ORDER: return order(s, e, why, n);
    case E_APPLY: return apply_promo(s, e, why, n);
    case E_PAY: return pay(s, e, why, n);
    case E_PAY_CREDIT: return pay_credit(s, e, why, n);
    case E_PAY_GIFT: return pay_gift(s, e, why, n);
    case E_SHIP: return ship(s, e, why, n);
    case E_CANCEL: return cancel(s, e, why, n);
    }
    return 1;
}
