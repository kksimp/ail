#include "event.h"

#include <string.h>

int event_number(const char *s, size_t n, long long *out) {
    if (n == 0 || n > 5) return 0;
    long long v = 0;
    for (size_t i = 0; i < n; i++) {
        if (s[i] < '0' || s[i] > '9') return 0;
        v = v * 10 + (s[i] - '0');
    }
    *out = v;
    return 1;
}

static int valid_sku(const char *s, size_t n) {
    if (n == 0 || n >= sizeof ((struct event *)0)->sku) return 0;
    for (size_t i = 0; i < n; i++) {
        char c = s[i];
        if (!((c >= 'A' && c <= 'Z') || (c >= '0' && c <= '9') || c == '-')) return 0;
    }
    return 1;
}

int event_parse(const char *line, struct event *e) {
    const char *f[5];
    size_t len[5];
    int nf = 0;
    const char *p = line;
    for (;;) {
        const char *sp = strchr(p, ' ');
        size_t n = sp ? (size_t)(sp - p) : strlen(p);
        if (nf == 5) return 0;
        f[nf] = p;
        len[nf++] = n;
        if (!sp) break;
        p = sp + 1;
    }
    if (nf == 4 && len[0] == 7 && !strncmp(f[0], "receive", 7)) e->kind = RECEIVE;
    else if (nf == 3 && len[0] == 4 && !strncmp(f[0], "ship", 4)) e->kind = SHIP;
    else if (nf == 3 && len[0] == 5 && !strncmp(f[0], "count", 5)) e->kind = COUNT;
    else return 0;
    if (!valid_sku(f[1], len[1])) return 0;
    memcpy(e->sku, f[1], len[1]);
    e->sku[len[1]] = '\0';
    if (!event_number(f[2], len[2], &e->qty)) return 0;
    e->price = 0;
    if (e->kind == RECEIVE && !event_number(f[3], len[3], &e->price)) return 0;
    if (e->qty == 0 && e->kind != COUNT) return 0;
    return 1;
}
