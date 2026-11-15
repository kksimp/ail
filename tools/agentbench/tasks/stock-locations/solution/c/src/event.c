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

int valid_sku(const char *s, size_t n) {
    if (n == 0 || n >= sizeof ((struct event *)0)->sku) return 0;
    for (size_t i = 0; i < n; i++) {
        char c = s[i];
        if (!((c >= 'A' && c <= 'Z') || (c >= '0' && c <= '9') || c == '-')) return 0;
    }
    return 1;
}

static int valid_loc(const char *s, size_t n) {
    if (n < 2 || n >= sizeof ((struct event *)0)->loc || s[0] != '@') return 0;
    for (size_t i = 1; i < n; i++) {
        char c = s[i];
        if (!((c >= 'a' && c <= 'z') || (c >= '0' && c <= '9') || c == '-')) return 0;
    }
    return 1;
}

static void copy_field(char *dst, const char *src, size_t n) {
    memcpy(dst, src, n);
    dst[n] = '\0';
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
    int want;
    if (len[0] == 7 && !strncmp(f[0], "receive", 7)) e->kind = RECEIVE, want = 4;
    else if (len[0] == 4 && !strncmp(f[0], "ship", 4)) e->kind = SHIP, want = 3;
    else if (len[0] == 5 && !strncmp(f[0], "count", 5)) e->kind = COUNT, want = 3;
    else if (len[0] == 4 && !strncmp(f[0], "move", 4)) e->kind = MOVE, want = 5;
    else return 0;
    strcpy(e->loc, "@main");
    e->to[0] = '\0';
    if (e->kind == MOVE) {
        if (nf != 5 || !valid_loc(f[3], len[3]) || !valid_loc(f[4], len[4])) return 0;
        copy_field(e->loc, f[3], len[3]);
        copy_field(e->to, f[4], len[4]);
    } else if (nf == want + 1 && valid_loc(f[want], len[want])) {
        copy_field(e->loc, f[want], len[want]);
    } else if (nf != want) {
        return 0;
    }
    if (!valid_sku(f[1], len[1])) return 0;
    copy_field(e->sku, f[1], len[1]);
    if (!event_number(f[2], len[2], &e->qty)) return 0;
    e->price = 0;
    if (e->kind == RECEIVE && !event_number(f[3], len[3], &e->price)) return 0;
    if (e->qty == 0 && e->kind != COUNT) return 0;
    return 1;
}
