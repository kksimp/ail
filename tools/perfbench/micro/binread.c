/* Twin of binread.ail: the same records written and read back with memcpy (C11, libc only). */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(int argc, char **argv) {
    long n = argc > 1 ? atol(argv[1]) : 1000000;
    size_t len = (size_t)n * 22;
    unsigned char *b = malloc(len);
    int64_t seed = 12345;
    size_t o = 0;
    for (long i = 0; i < n; i++) {
        seed = (seed * 1103515245 + 12345) % 2147483648;
        uint32_t u = (uint32_t)seed;
        uint16_t h = __builtin_bswap16((uint16_t)(seed % 65536));
        int64_t v = seed - 1073741824;
        double f = (double)(seed % 1000) * 0.5;
        memcpy(b + o, &u, 4);
        memcpy(b + o + 4, &h, 2);
        memcpy(b + o + 6, &v, 8);
        memcpy(b + o + 14, &f, 8);
        o += 22;
    }
    int64_t isum = 0;
    double fsum = 0.0;
    for (int r = 0; r < 10; r++) {
        for (size_t at = 0; at < len; at += 22) {
            uint32_t u; uint16_t h; int64_t v; double f;
            memcpy(&u, b + at, 4);
            memcpy(&h, b + at + 4, 2);
            memcpy(&v, b + at + 6, 8);
            memcpy(&f, b + at + 14, 8);
            isum += (int64_t)u + __builtin_bswap16(h) + v;
            fsum += f;
        }
    }
    printf("%zu %lld %.1f\n", len, (long long)isum, fsum);
    free(b);
    return 0;
}
