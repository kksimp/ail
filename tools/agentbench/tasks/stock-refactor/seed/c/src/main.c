#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* One line of the stock log. */


enum kind { RECEIVE, SHIP, COUNT };

struct event {
    enum kind kind;
    char sku[64];
    long long qty;
    long long price;
};

/* 1 to 5 ASCII digits of s[0..n), as a number in *out. Returns 1 on success. */
int event_number(const char *s, size_t n, long long *out);

/* Parses one line (no newline). Returns 1 and fills *e for a valid event, 0 otherwise. */
int event_parse(const char *line, struct event *e);

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

/* On-hand quantities and average cost per SKU. */



struct entry {
    char sku[64];
    long long qty;
    long long cost;
};

/* Entries are kept in ascending byte order of SKU. */
struct store {
    struct entry *items;
    size_t len, cap;
};

enum problem { OK, UNKNOWN, SHORT };

void store_init(struct store *s);
void store_free(struct store *s);

/* Applies one event; a rejected event changes nothing and returns UNKNOWN or SHORT. */
enum problem store_apply(struct store *s, const struct event *e);

void store_init(struct store *s) {
    s->items = NULL;
    s->len = s->cap = 0;
}

void store_free(struct store *s) {
    free(s->items);
    store_init(s);
}

/* The index of sku, or of where it would be inserted; *found says which. */
static size_t find(const struct store *s, const char *sku, int *found) {
    size_t lo = 0, hi = s->len;
    while (lo < hi) {
        size_t mid = lo + (hi - lo) / 2;
        int c = strcmp(s->items[mid].sku, sku);
        if (c == 0) {
            *found = 1;
            return mid;
        }
        if (c < 0) lo = mid + 1;
        else hi = mid;
    }
    *found = 0;
    return lo;
}

static struct entry *insert(struct store *s, size_t at, const char *sku) {
    if (s->len == s->cap) {
        size_t cap = s->cap ? s->cap * 2 : 16;
        struct entry *t = realloc(s->items, cap * sizeof *t);
        if (!t) abort();
        s->items = t;
        s->cap = cap;
    }
    memmove(s->items + at + 1, s->items + at, (s->len - at) * sizeof *s->items);
    s->len++;
    struct entry *it = &s->items[at];
    memset(it, 0, sizeof *it);
    strcpy(it->sku, sku);
    return it;
}

enum problem store_apply(struct store *s, const struct event *e) {
    int found;
    size_t at = find(s, e->sku, &found);
    if (e->kind == RECEIVE) {
        struct entry *it = found ? &s->items[at] : insert(s, at, e->sku);
        it->qty += e->qty;
        it->cost += e->qty * e->price;
        return OK;
    }
    if (!found) return UNKNOWN;
    struct entry *it = &s->items[at];
    if (e->kind == SHIP) {
        if (e->qty > it->qty) return SHORT;
        it->cost -= it->cost * e->qty / it->qty;
        it->qty -= e->qty;
    } else {
        if (it->qty > 0) it->cost = it->cost * e->qty / it->qty;
        it->qty = e->qty;
    }
    return OK;
}

/* The three outputs, written to stdout. */



/* Cents as dollars with two decimals: "1234.05", "-0.50". */
void report_money(char *out, size_t n, long long cents);

void report_report(const struct store *s);
void report_low(const struct store *s, long long n);
void report_total(const struct store *s);

void report_money(char *out, size_t n, long long c) {
    unsigned long long a = c < 0 ? 0ULL - (unsigned long long)c : (unsigned long long)c;
    snprintf(out, n, "%s%llu.%02llu", c < 0 ? "-" : "", a / 100, a % 100);
}

void report_report(const struct store *s) {
    char m[32];
    for (size_t i = 0; i < s->len; i++) {
        report_money(m, sizeof m, s->items[i].cost);
        printf("%s\t%lld\t%s\n", s->items[i].sku, s->items[i].qty, m);
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

static char *read_all(FILE *in, size_t *len) {
    size_t cap = 4096, n = 0, got;
    char *buf = malloc(cap);
    if (!buf) abort();
    while ((got = fread(buf + n, 1, cap - n - 1, in)) > 0) {
        n += got;
        if (n + 1 >= cap) {
            char *t = realloc(buf, cap *= 2);
            if (!t) abort();
            buf = t;
        }
    }
    buf[n] = '\0';
    *len = n;
    return buf;
}

int main(int argc, char **argv) {
    long long low = 0;
    int cmd;
    if (argc == 2 && !strcmp(argv[1], "report")) cmd = 0;
    else if (argc == 2 && !strcmp(argv[1], "total")) cmd = 1;
    else if (argc == 3 && !strcmp(argv[1], "low") && event_number(argv[2], strlen(argv[2]), &low)) cmd = 2;
    else {
        fputs("usage: stock report|low N|total\n", stderr);
        return 1;
    }
    size_t len;
    char *buf = read_all(stdin, &len);
    if (len > 0 && buf[len - 1] == '\n') buf[--len] = '\0';
    struct store s;
    store_init(&s);
    int bad = 0, lineno = 0;
    for (char *line = buf; len > 0;) {
        char *nl = strchr(line, '\n');
        if (nl) *nl = '\0';
        lineno++;
        if (line[0] != '\0' && line[0] != '#') {
            struct event e;
            if (!event_parse(line, &e)) {
                fprintf(stderr, "error: line %d: bad line\n", lineno);
                bad = 1;
            } else {
                enum problem p = store_apply(&s, &e);
                if (p != OK) {
                    fprintf(stderr, "error: line %d: %s %s\n", lineno, p == UNKNOWN ? "unknown" : "short", e.sku);
                    bad = 1;
                }
            }
        }
        if (!nl) break;
        line = nl + 1;
    }
    if (cmd == 0) report_report(&s);
    else if (cmd == 1) report_total(&s);
    else report_low(&s, low);
    store_free(&s);
    free(buf);
    return bad ? 2 : 0;
}
