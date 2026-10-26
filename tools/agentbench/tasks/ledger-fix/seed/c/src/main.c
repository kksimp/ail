#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    const char *date, *account, *memo;
    long long amount;
    size_t order;
} Row;

static void money(char *out, size_t n, long long c) {
    unsigned long long a = c < 0 ? 0ULL - (unsigned long long)c : (unsigned long long)c;
    snprintf(out, n, "%lld.%02llu", c / 100, a % 100);
}

static int all_digits(const char *s, size_t n) {
    if (n == 0) return 0;
    for (size_t i = 0; i < n; i++)
        if (s[i] < '0' || s[i] > '9') return 0;
    return 1;
}

static int valid_date(const char *d) {
    if (strlen(d) != 10 || d[4] != '-' || d[7] != '-') return 0;
    if (!all_digits(d, 4) || !all_digits(d + 5, 2) || !all_digits(d + 8, 2)) return 0;
    int m = (d[5] - '0') * 10 + (d[6] - '0'), day = (d[8] - '0') * 10 + (d[9] - '0');
    return m >= 1 && m <= 12 && day >= 1 && day <= 30;
}

static int cmp_abs(const void *pa, const void *pb) {
    const Row *a = pa, *b = pb;
    unsigned long long x = a->amount < 0 ? 0ULL - (unsigned long long)a->amount : (unsigned long long)a->amount;
    unsigned long long y = b->amount < 0 ? 0ULL - (unsigned long long)b->amount : (unsigned long long)b->amount;
    if (x != y) return x > y ? -1 : 1;
    return a->order < b->order ? -1 : a->order > b->order;
}

typedef struct { char *key; long long sum; } Tot;

static int cmp_tot(const void *pa, const void *pb) { return strcmp(((const Tot *)pa)->key, ((const Tot *)pb)->key); }

static int usage(void) {
    fputs("usage: ledger balance|top N|monthly\n", stderr);
    return 1;
}

int main(int argc, char **argv) {
    int top = 0;
    unsigned long long n = 0;
    if (argc == 2 && (!strcmp(argv[1], "balance") || !strcmp(argv[1], "monthly"))) {
    } else if (argc == 3 && !strcmp(argv[1], "top") && all_digits(argv[2], strlen(argv[2]))) {
        top = 1;
        errno = 0;
        n = strtoull(argv[2], NULL, 10);
        if (errno == ERANGE) n = ~0ULL;
    } else {
        return usage();
    }
    size_t cap = 4096, len = 0;
    char *buf = malloc(cap);
    size_t got;
    while ((got = fread(buf + len, 1, cap - len - 1, stdin)) > 0) {
        len += got;
        if (len + 1 >= cap) buf = realloc(buf, cap *= 2);
    }
    buf[len] = 0;
    if (len > 0 && buf[len - 1] == '\n') buf[--len] = 0;
    const char *hdr = "date,account,amount_cents,memo";
    char *save = buf;
    char *line = len ? buf : NULL;
    if (!line || strncmp(buf, hdr, strlen(hdr)) != 0 || (buf[strlen(hdr)] != '\n' && buf[strlen(hdr)] != 0)) {
        fputs("error: line 1: bad header\n", stderr);
        free(save);
        return 2;
    }
    Row *rows = calloc(len / 2 + 2, sizeof(Row));
    size_t nrows = 0;
    int bad = 0, lineno = 1;
    char *next = strchr(buf, '\n');
    while (next) {
        line = next + 1;
        lineno++;
        next = strchr(line, '\n');
        if (next) *next = 0;
        char *f[4];
        int nf = 0;
        char *p = line;
        f[nf++] = p;
        while (*p) {
            if (*p == ',') {
                *p = 0;
                if (nf == 4) { nf = 5; break; }
                f[nf++] = p + 1;
            }
            p++;
        }
        const char *why = NULL;
        long long amt = 0;
        if (nf != 4) why = "expected 4 fields";
        else if (!valid_date(f[0])) why = "bad date";
        else if (!f[1][0]) why = "empty account";
        else {
            const char *d = f[2][0] == '-' ? f[2] + 1 : f[2];
            errno = 0;
            if (!all_digits(d, strlen(d))) why = "bad amount";
            else {
                amt = strtoll(f[2], NULL, 10);
                if (errno == ERANGE) why = "bad amount";
            }
        }
        if (why) {
            fprintf(stderr, "error: line %d: %s\n", lineno, why);
            bad = 1;
        } else {
            rows[nrows] = (Row){f[0], f[1], f[3], amt, nrows};
            nrows++;
        }
    }
    char m[40];
    if (top) {
        qsort(rows, nrows, sizeof(Row), cmp_abs);
        for (size_t i = 0; i < nrows && i < n; i++) {
            money(m, sizeof m, rows[i].amount);
            printf("%s\t%s\t%s\t%s\n", rows[i].date, rows[i].account, m, rows[i].memo);
        }
    } else {
        int by_account = !strcmp(argv[1], "balance");
        Tot *t = calloc(nrows + 1, sizeof(Tot));
        size_t nt = 0;
        for (size_t i = 0; i < nrows; i++) {
            char *key = by_account ? strdup(rows[i].account) : strndup(rows[i].date, 7);
            size_t j;
            for (j = 0; j < nt; j++)
                if (!strcmp(t[j].key, key)) break;
            if (j == nt) t[nt++] = (Tot){key, 0};
            else free(key);
            t[j].sum += rows[i].amount;
        }
        qsort(t, nt, sizeof(Tot), cmp_tot);
        for (size_t j = 0; j < nt; j++) {
            money(m, sizeof m, t[j].sum);
            printf("%s\t%s\n", t[j].key, m);
            free(t[j].key);
        }
        free(t);
    }
    free(rows);
    free(save);
    return bad ? 2 : 0;
}
