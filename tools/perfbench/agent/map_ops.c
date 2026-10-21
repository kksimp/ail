/* Maps and sorting: the same generator as map_ops.ail. Keys keep insertion
   order, as AIL's Map does, so ties break the way a stable sort would. */
#include <stdio.h>
#include <string.h>

#define CAP 65536

static char keys[20000][16];
static long vals[20000];
static int slot[CAP];
static int nkeys = 0;

static unsigned long hash(const char *s) {
    unsigned long h = 1469598103934665603UL;
    while (*s) {
        h = (h ^ (unsigned char)*s++) * 1099511628211UL;
    }
    return h;
}

static int find(const char *k, int insert) {
    unsigned long i = hash(k) & (CAP - 1);
    while (slot[i]) {
        if (strcmp(keys[slot[i] - 1], k) == 0) {
            return slot[i] - 1;
        }
        i = (i + 1) & (CAP - 1);
    }
    if (!insert) {
        return -1;
    }
    strcpy(keys[nkeys], k);
    vals[nkeys] = 0;
    slot[i] = ++nkeys;
    return nkeys - 1;
}

int main(void) {
    long seed = 5;
    char k[16];
    for (int n = 0; n < 200000; n++) {
        seed = (seed * 3877 + 29573) % 139968;
        snprintf(k, sizeof k, "key%ld", seed % 20000);
        vals[find(k, 1)] += 1;
    }
    long hits = 0;
    for (int i = 0; i < 20000; i++) {
        snprintf(k, sizeof k, "key%d", i);
        int j = find(k, 0);
        if (j >= 0) {
            hits += vals[j];
        }
    }
    static char used[20000];
    for (int t = 0; t < 5; t++) {
        int best = -1;
        for (int j = 0; j < nkeys; j++) {
            if (!used[j] && (best < 0 || vals[j] > vals[best])) {
                best = j;
            }
        }
        used[best] = 1;
        printf("%s %ld\n", keys[best], vals[best]);
    }
    printf("distinct %d hits %ld\n", nkeys, hits);
    return 0;
}
