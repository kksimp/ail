#include "reports.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "grow.h"
#include "money.h"

void report_orders(const struct shop *s) {
    char a[32], b[32];
    for (size_t i = 0; i < s->book.len; i++) {
        const struct order *o = &s->book.all[i];
        printf("%s\t%s\t%s\t%s\t%s\n", o->id, o->cust, status_text(o->status), money_show(order_total(o), a),
               money_show(o->paid, b));
    }
}

int report_invoice(const struct shop *s, const char *id) {
    const struct order *o = orders_find(&s->book, id);
    if (!o) return 0;
    char m[5][32];
    printf("order %s %s %s\n", o->id, o->cust, status_text(o->status));
    long long gross = 0, discount = 0, tax = 0;
    for (size_t i = 0; i < o->nlines; i++) {
        const struct line *l = &o->lines[i];
        printf("%s\t%lld\t%s\t%s\t%s\t%s\t%s\n", l->sku, l->qty, money_show(l->price, m[0]), money_show(l->gross, m[1]),
               money_show(l->discount, m[2]), money_show(l->tax, m[3]), money_show(l->total, m[4]));
        gross += l->gross;
        discount += l->discount;
        tax += l->tax;
    }
    if (o->promo[0]) printf("promo\t%s\n", o->promo);
    printf("subtotal\t%s\n", money_show(gross, m[0]));
    printf("discount\t%s\n", money_show(discount, m[0]));
    printf("tax\t%s\n", money_show(tax, m[0]));
    printf("total\t%s\n", money_show(order_total(o), m[0]));
    if (order_refunded(o) > 0) printf("returned\t%s\n", money_show(order_refunded(o), m[0]));
    printf("paid\t%s\n", money_show(o->paid, m[0]));
    printf("due\t%s\n", money_show(o->status == OPEN ? order_due(o) : 0, m[0]));
    return 1;
}

void report_stock(const struct shop *s) {
    char a[32], b[32];
    for (size_t i = 0; i < s->stock.len; i++) {
        const struct item *it = &s->stock.items[i];
        printf("%s\t%s\t%s\t%lld\t%lld\t%lld\t%s\n", it->sku, it->cat, money_show(it->price, a), it->onhand, it->reserved,
               item_available(it), money_show(it->value, b));
    }
}

void report_low(const struct shop *s, long long n) {
    for (size_t i = 0; i < s->stock.len; i++) {
        const struct item *it = &s->stock.items[i];
        if (item_available(it) < n) printf("%s\t%lld\n", it->sku, item_available(it));
    }
}

void report_customers(const struct shop *s) {
    char a[32], b[32];
    for (size_t i = 0; i < s->people.len; i++) {
        const struct customer *c = &s->people.all[i];
        long long placed = 0, spent = 0;
        for (size_t j = 0; j < s->book.len; j++) {
            const struct order *o = &s->book.all[j];
            if (strcmp(o->cust, c->id) != 0) continue;
            if (o->status != CANCELLED) placed++;
            if (order_sold(o)) spent += order_total(o) - order_refunded(o);
        }
        printf("%s\t%s\t%lld\t%s\t%s\n", c->id, c->tier, placed, money_show(spent, a), money_show(c->credit, b));
    }
}

int report_history(const struct shop *s, const char *cust) {
    const struct customer *c = customers_find(&s->people, cust);
    if (!c) return 0;
    char a[32];
    for (size_t j = 0; j < s->book.len; j++) {
        const struct order *o = &s->book.all[j];
        if (strcmp(o->cust, cust) == 0) printf("%s\t%s\t%s\n", o->id, status_text(o->status), money_show(order_total(o), a));
    }
    printf("credit\t%s\n", money_show(c->credit, a));
    return 1;
}

/* Sums over the shipped lines of one category, or over all of them, less what was returned. */
struct tally {
    char cat[FIELD];
    long long units, net, tax, cost;
};

static int by_cat(const void *a, const void *b) {
    return strcmp(((const struct tally *)a)->cat, ((const struct tally *)b)->cat);
}

/* The tallies of every category with a shipped line, in ascending byte order, and their sum in *all.
   The caller frees the array. */
static struct tally *shipped_tallies(const struct shop *s, size_t *len, struct tally *all) {
    struct tally *t = NULL;
    size_t cap = 0;
    *len = 0;
    memset(all, 0, sizeof *all);
    for (size_t i = 0; i < s->book.len; i++) {
        const struct order *o = &s->book.all[i];
        if (!order_sold(o)) continue;
        for (size_t j = 0; j < o->nlines; j++) {
            const struct line *l = &o->lines[j];
            size_t k = 0;
            while (k < *len && strcmp(t[k].cat, l->cat) != 0) k++;
            if (k == *len) {
                GROW(t, *len, cap);
                memset(&t[k], 0, sizeof t[k]);
                strcpy(t[k].cat, l->cat);
                (*len)++;
            }
            t[k].units += l->qty - l->returned;
            t[k].net += l->net - l->rnet;
            t[k].tax += l->tax - l->rtax;
            t[k].cost += l->cost - l->rcost;
            all->units += l->qty - l->returned;
            all->net += l->net - l->rnet;
            all->tax += l->tax - l->rtax;
            all->cost += l->cost - l->rcost;
        }
    }
    if (*len) qsort(t, *len, sizeof *t, by_cat);
    return t;
}

void report_sales(const struct shop *s) {
    size_t len;
    struct tally all;
    struct tally *t = shipped_tallies(s, &len, &all);
    char a[32], b[32];
    for (size_t k = 0; k < len; k++) printf("%s\t%lld\t%s\t%s\n", t[k].cat, t[k].units, money_show(t[k].net, a), money_show(t[k].tax, b));
    printf("total\t%lld\t%s\t%s\n", all.units, money_show(all.net, a), money_show(all.tax, b));
    free(t);
}

static void margin_line(const char *name, const struct tally *t) {
    char a[32], b[32], c[32];
    printf("%s\t%s\t%s\t%s\n", name, money_show(t->net, a), money_show(t->cost, b), money_show(t->net - t->cost, c));
}

void report_margin(const struct shop *s) {
    size_t len;
    struct tally all;
    struct tally *t = shipped_tallies(s, &len, &all);
    for (size_t k = 0; k < len; k++) margin_line(t[k].cat, &t[k]);
    margin_line("total", &all);
    free(t);
}

void report_purchases(const struct shop *s) {
    char a[32];
    for (size_t i = 0; i < s->buying.len; i++) {
        const struct purchase *p = &s->buying.all[i];
        printf("%s\t%s\t%lld\t%s\n", p->po, po_status_text(p->status), purchase_units(p), money_show(purchase_cost(p), a));
    }
}

void report_gifts(const struct shop *s) {
    char a[32], b[32];
    for (size_t i = 0; i < s->cards.len; i++) {
        const struct card *c = &s->cards.all[i];
        printf("%s\t%s\t%s\n", c->card, money_show(c->sold, a), money_show(c->balance, b));
    }
}

void report_promos(const struct shop *s) {
    for (size_t i = 0; i < s->deals.len; i++) {
        const struct promo *p = &s->deals.all[i];
        printf("%s\t%s\t%lld%%\t%lld/%lld\n", p->code, p->cat, p->pct, p->used, p->uses);
    }
}
