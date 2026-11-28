#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static int cmp_int(const void *a, const void *b) {
    long x = *(const long *)a, y = *(const long *)b;
    return (x > y) - (x < y);
}
static int cmp_str(const void *a, const void *b) { return strcmp(*(char *const *)a, *(char *const *)b); }

int main(int argc, char **argv) {
    long n = argc > 1 ? atol(argv[1]) : 1000000, seed = 42;
    long *xs = malloc(n * sizeof(long));
    for (long i = 0; i < n; i++) {
        seed = (seed * 3877 + 29573) % 139968;
        xs[i] = seed * 7 + i % 13;
    }
    qsort(xs, n, sizeof(long), cmp_int);
    long chk = 0;
    for (long i = 0; i < n; i++) chk = (chk * 31 + xs[i]) % 1000000007;
    long m = n / 4;
    char **ss = malloc(m * sizeof(char *));
    for (long j = 0; j < m; j++) {
        seed = (seed * 3877 + 29573) % 139968;
        ss[j] = malloc(32);
        snprintf(ss[j], 32, "k%ld-%ld", seed, j % 1000);
    }
    qsort(ss, m, sizeof(char *), cmp_str);
    printf("ints %ld, check %ld, first %ld, last %ld\n", n, chk, xs[0], xs[n - 1]);
    printf("strings %ld, first %s, last %s\n", m, ss[0], ss[m - 1]);
    return 0;
}
