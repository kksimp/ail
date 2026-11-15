#include "pricing.h"

#include <string.h>

#include "catalog.h"
#include "accounts.h"
#include "promos.h"

void price_line(struct line *l, const char *promo_cat, long long promo_pct) {
    l->gross = l->price * l->qty;
    long long tier_off = tier_discount(l->gross, l->tier);
    long long promo_off = 0;
    if (strcmp(promo_cat, l->cat) == 0)
        promo_off = promo_discount(l->gross - tier_off, promo_pct);
    l->discount = tier_off + promo_off;
    l->net = l->gross - l->discount;
    l->tax = tax_on(l->net, l->rate);
    l->total = l->net + l->tax;
}
