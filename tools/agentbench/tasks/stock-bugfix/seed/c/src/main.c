#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "event.h"
#include "report.h"
#include "store.h"

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
        if (line[0] != '\0' && line[0] != '#') {
            lineno++;
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
