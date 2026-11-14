/* Collatz: total steps and the longest chain for starts 1 up to N (exclusive). */
#include <stdio.h>
#include <stdlib.h>

static long long steps(long long n) {
    long long k = 0;
    while (n != 1) {
        if (n % 2 == 0) {
            n = n / 2;
        } else {
            n = 3 * n + 1;
        }
        k += 1;
    }
    return k;
}

int main(int argc, char **argv) {
    long long limit = argc > 1 ? atoll(argv[1]) : 30000;
    long long total = 0;
    long long best = 1;
    long long best_steps = 0;
    for (long long i = 1; i < limit; i++) {
        long long s = steps(i);
        total += s;
        if (s > best_steps) {
            best = i;
            best_steps = s;
        }
    }
    printf("total steps: %lld\n", total);
    printf("longest: %lld with %lld steps\n", best, best_steps);
    return 0;
}
