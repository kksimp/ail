/* C twin of race_smoke.ail. */
#include <stdio.h>

static long long collatz(long long start) {
    long long n = start, steps = 0;
    while (n != 1) {
        if (n % 2 == 0) n = n / 2; else n = 3 * n + 1;
        steps++;
    }
    return steps;
}

int main(void) {
    long long total = 0;
    for (long long i = 1; i < 30000; i++) total += collatz(i);
    printf("%lld\n", total);
    return 0;
}
