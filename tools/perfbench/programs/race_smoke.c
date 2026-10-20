/* C twin of race_smoke.ail. */
#include <stdio.h>

static long collatz(long start) {
    long n = start, steps = 0;
    while (n != 1) {
        if (n % 2 == 0) n = n / 2; else n = 3 * n + 1;
        steps++;
    }
    return steps;
}

int main(void) {
    long total = 0;
    for (long i = 1; i < 30000; i++) total += collatz(i);
    printf("%ld\n", total);
    return 0;
}
