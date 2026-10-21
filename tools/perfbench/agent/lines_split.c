/* Log analysis: the same generator as lines_split.ail. */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(void) {
    static const char *levels[] = {"info", "warn", "error", "debug"};
    long seed = 11;
    size_t cap = 200000 * 96, len = 0;
    char *text = malloc(cap);
    for (long i = 0; i < 200000; i++) {
        seed = (seed * 3877 + 29573) % 139968;
        len += sprintf(text + len, "%s2026-10-21T07:%02ld %s user=%ld ms=%ld GET /api/v%ld/items",
                       i ? "\n" : "", i % 60, levels[seed % 4], seed % 97, seed % 1000, seed % 5);
    }
    long by_level[4] = {0}, by_user[97] = {0}, total = 0;
    long users = 0;
    char *p = text, *end = text + len;
    while (p < end) {
        char *nl = memchr(p, '\n', end - p);
        if (!nl) {
            nl = end;
        }
        *nl = 0;
        char *f1 = strchr(p, ' ') + 1;
        char *f2 = strchr(f1, ' ') + 1;
        char *f3 = strchr(f2, ' ') + 1;
        int lv = f1[0] == 'i' ? 0 : f1[0] == 'w' ? 1 : f1[0] == 'e' ? 2 : 3;
        by_level[lv]++;
        long u = atol(f2 + 5);
        if (by_user[u]++ == 0) {
            users++;
        }
        total += atol(f3 + 3);
        p = nl + 1;
    }
    /* sorted by name: debug, error, info, warn */
    printf("debug %ld\nerror %ld\ninfo %ld\nwarn %ld\n", by_level[3], by_level[2], by_level[0], by_level[1]);
    printf("users %ld total_ms %ld\n", users, total);
    free(text);
    return 0;
}
