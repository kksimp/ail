/* FASTA-style DNA, same allocation as fasta.ail: each line is a growing array
   of bases (doubling as they are pushed) that is then joined into a fresh
   string, printed and freed. (fasta.c reuses one buffer, best effort.) */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define WIDTH 60

static void emit(char *line, int used) {
    char *joined = malloc(used + 1);
    memcpy(joined, line, used);
    joined[used] = '\0';
    puts(joined);
    free(joined);
}

int main(int argc, char **argv) {
    long long n = argc > 1 ? atoll(argv[1]) : 100000;
    const char letters[] = "acgt";
    long long seed = 42;
    long long counts[4] = {0, 0, 0, 0};
    char *line = NULL;
    int used = 0;
    int cap = 0;
    for (long long i = 0; i < n; i++) {
        seed = (seed * 3877 + 29573) % 139968;
        long long t = seed * 100 / 139968;
        int base = 3;
        if (t < 30) {
            base = 0;
        } else if (t < 50) {
            base = 1;
        } else if (t < 70) {
            base = 2;
        }
        counts[base] += 1;
        if (used == cap) {
            cap = cap ? cap * 2 : 4;
            line = realloc(line, cap);
        }
        line[used++] = letters[base];
        if (used == WIDTH) {
            emit(line, used);
            free(line);
            line = NULL;
            used = 0;
            cap = 0;
        }
    }
    if (used > 0) {
        emit(line, used);
        free(line);
    }
    printf("a %lld c %lld g %lld t %lld\n", counts[0], counts[1], counts[2], counts[3]);
    return 0;
}
