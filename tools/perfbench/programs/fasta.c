/* FASTA-style DNA: the same LCG and weights as fasta.ail. */
#include <stdio.h>
#include <stdlib.h>

#define WIDTH 60

int main(int argc, char **argv) {
    long n = argc > 1 ? atol(argv[1]) : 100000;
    const char letters[] = "acgt";
    long seed = 42;
    long counts[4] = {0, 0, 0, 0};
    char line[WIDTH + 1];
    int used = 0;
    for (long i = 0; i < n; i++) {
        seed = (seed * 3877 + 29573) % 139968;
        long t = seed * 100 / 139968;
        int base = 3;
        if (t < 30) {
            base = 0;
        } else if (t < 50) {
            base = 1;
        } else if (t < 70) {
            base = 2;
        }
        counts[base] += 1;
        line[used++] = letters[base];
        if (used == WIDTH) {
            line[used] = '\0';
            puts(line);
            used = 0;
        }
    }
    if (used > 0) {
        line[used] = '\0';
        puts(line);
    }
    printf("a %ld c %ld g %ld t %ld\n", counts[0], counts[1], counts[2], counts[3]);
    return 0;
}
