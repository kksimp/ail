/* Dense matrix multiply: the same entries and checks as matmul.ail. */
#include <stdio.h>
#include <stdlib.h>

int main(int argc, char **argv) {
    int n = argc > 1 ? atoi(argv[1]) : 120;
    long long *a = malloc((size_t)n * n * sizeof(long long));
    long long *b = malloc((size_t)n * n * sizeof(long long));
    long long *c = malloc((size_t)n * n * sizeof(long long));
    for (long long i = 0; i < n; i++) {
        for (long long j = 0; j < n; j++) {
            a[i * n + j] = (i * j + i + j) % 7;
            b[i * n + j] = (i + 2 * j) % 5;
        }
    }
    for (long long i = 0; i < n; i++) {
        for (long long j = 0; j < n; j++) {
            long long s = 0;
            for (long long k = 0; k < n; k++) {
                s += a[i * n + k] * b[k * n + j];
            }
            c[i * n + j] = s;
        }
    }
    long long trace = 0;
    long long total = 0;
    for (long long i = 0; i < n; i++) {
        trace += c[i * n + i];
        for (long long j = 0; j < n; j++) {
            total += c[i * n + j];
        }
    }
    printf("trace: %lld\n", trace);
    printf("sum: %lld\n", total);
    return 0;
}
