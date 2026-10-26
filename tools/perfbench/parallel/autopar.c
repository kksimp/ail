/* C twin of autopar.ail with OpenMP: lengths per number, summed. */
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>

static int64_t steps(int64_t n) {
    int64_t x = n, k = 0;
    while (x != 1) {
        x = x % 2 == 0 ? x / 2 : 3 * x + 1;
        k += 1;
    }
    return k;
}

int main(int argc, char **argv) {
    int64_t n = argc > 1 ? atoll(argv[1]) : 3000000;
    int64_t *lengths = calloc(n, sizeof *lengths);
    int64_t total = 0;
#pragma omp parallel for reduction(+ : total) schedule(static)
    for (int64_t i = 0; i < n; i++) {
        int64_t s = steps(i + 1);
        lengths[i] = s;
        total += s;
    }
    int64_t longest = 0;
    for (int64_t i = 0; i < n; i++)
        if (lengths[i] > lengths[longest]) longest = i;
    printf("total steps: %lld\n", (long long)total);
    printf("longest: %lld with %lld steps\n", (long long)(longest + 1), (long long)lengths[longest]);
    return 0;
}
