/* till: a shop's back office. Reads the journal on stdin and prints one report (spec.md). */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "cli.h"
#include "journal.h"
#include "reports.h"
#include "shop.h"

/* All of stdin, NUL-terminated. */
static char *read_all(void) {
    size_t cap = 1 << 16, len = 0;
    char *buf = malloc(cap);
    if (!buf) abort();
    size_t got;
    while ((got = fread(buf + len, 1, cap - len - 1, stdin)) > 0) {
        len += got;
        if (cap - len < 2) {
            cap *= 2;
            char *t = realloc(buf, cap);
            if (!t) abort();
            buf = t;
        }
    }
    buf[len] = 0;
    return buf;
}

int main(int argc, char **argv) {
    struct request r = cli_parse(argc - 1, argv + 1);
    if (r.cmd == C_USAGE) {
        fprintf(stderr, "%s\n", USAGE);
        return 1;
    }
    char *text = read_all();
    size_t len = strlen(text);
    if (len > 0 && text[len - 1] == '\n') text[--len] = 0; /* a trailing newline is not an extra line */
    struct shop s;
    shop_init(&s);
    int bad = 0;
    char *line = text;
    for (long k = 1; line; k++) {
        char *nl = strchr(line, '\n');
        if (nl) *nl = 0;
        if (line[0] && line[0] != '#') {
            struct entry e;
            char why[128];
            if (!journal_parse(line, &e)) {
                snprintf(why, sizeof why, "bad line");
                fprintf(stderr, "error: line %ld: %s\n", k, why);
                bad++;
            } else {
                if (!shop_apply(&s, &e, why, sizeof why)) {
                    fprintf(stderr, "error: line %ld: %s\n", k, why);
                    bad++;
                }
                entry_free(&e);
            }
        }
        line = nl ? nl + 1 : NULL;
    }
    int code = bad ? 2 : 0;
    switch (r.cmd) {
    case C_ORDERS: report_orders(&s); break;
    case C_INVOICE:
        if (!report_invoice(&s, r.arg)) {
            fprintf(stderr, "error: no order %s\n", r.arg);
            code = 3;
        }
        break;
    case C_STOCK: report_stock(&s); break;
    case C_LOW: report_low(&s, r.n); break;
    case C_CUSTOMERS: report_customers(&s); break;
    case C_HISTORY:
        if (!report_history(&s, r.arg)) {
            fprintf(stderr, "error: no customer %s\n", r.arg);
            code = 3;
        }
        break;
    case C_SALES: report_sales(&s); break;
    case C_MARGIN: report_margin(&s); break;
    case C_PURCHASES: report_purchases(&s); break;
    case C_GIFTS: report_gifts(&s); break;
    case C_PROMOS: report_promos(&s); break;
    case C_USAGE: break;
    }
    shop_free(&s);
    free(text);
    return code;
}
