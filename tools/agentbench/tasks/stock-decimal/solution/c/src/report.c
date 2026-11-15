#include "report.h"

#include <stdio.h>

void report_money(char *out, size_t n, long long c) {
    unsigned long long a = c < 0 ? 0ULL - (unsigned long long)c : (unsigned long long)c;
    snprintf(out, n, "%s%llu.%02llu", c < 0 ? "-" : "", a / 100, a % 100);
}

void report_report(const struct store *s) {
    char m[32];
    for (size_t i = 0; i < s->len; i++) {
        report_money(m, sizeof m, s->items[i].cost);
        long long q = s->items[i].qty;
        printf("%s\t%lld.%03lld\t%s\n", s->items[i].sku, q / 1000, q % 1000, m);
    }
}

void report_low(const struct store *s, long long n) {
    for (size_t i = 0; i < s->len; i++)
        if (s->items[i].qty < n) printf("%s\n", s->items[i].sku);
}

void report_total(const struct store *s) {
    long long sum = 0;
    char m[32];
    for (size_t i = 0; i < s->len; i++) sum += s->items[i].cost;
    report_money(m, sizeof m, sum);
    printf("%s\n", m);
}
