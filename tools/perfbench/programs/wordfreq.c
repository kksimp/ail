/* Word frequencies: the same generator and vocabulary as wordfreq.ail. */
#include <stdio.h>
#include <stdlib.h>

static const char *WORDS[] = {"the", "of", "and", "to", "in", "fox",
                              "jumps", "lazy", "dog", "river", "stone", "moss"};
#define NWORDS 12

int main(int argc, char **argv) {
    long n = argc > 1 ? atol(argv[1]) : 200000;
    long counts[NWORDS] = {0};
    long seed = 7;
    for (long i = 0; i < n; i++) {
        seed = (seed * 3877 + 29573) % 139968;
        counts[seed % NWORDS] += 1;
    }
    int distinct = 0;
    for (int k = 0; k < NWORDS; k++) {
        printf("%s %ld\n", WORDS[k], counts[k]);
        if (counts[k] > 0) {
            distinct += 1;
        }
    }
    printf("distinct %d\n", distinct);
    return 0;
}
