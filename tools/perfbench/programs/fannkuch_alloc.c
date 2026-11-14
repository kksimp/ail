/* Fannkuch-redux, same allocation and widths as fannkuch.ail: 64-bit
   elements and a fresh `perm` array for every permutation. (fannkuch.c hoists
   one buffer and uses 32-bit ints, best effort.) */
#include <stdio.h>
#include <stdlib.h>

int main(int argc, char **argv) {
    int n = argc > 1 ? atoi(argv[1]) : 8;
    long long *perm1 = malloc(n * sizeof(long long));
    long long *perm;
    long long *count = malloc(n * sizeof(long long));
    for (int i = 0; i < n; i++) {
        perm1[i] = i;
        count[i] = 0;
    }
    int maxflips = 0;
    long long checksum = 0;
    long long permcount = 0;
    int r = n;
    int done = 0;
    while (!done) {
        while (r != 1) {
            count[r - 1] = r;
            r -= 1;
        }
        perm = malloc(n * sizeof(long long));
        for (int i = 0; i < n; i++) {
            perm[i] = perm1[i];
        }
        int flips = 0;
        long long k = perm[0];
        while (k != 0) {
            int i = 0;
            long long j = k;
            while (i < j) {
                long long t = perm[i];
                perm[i] = perm[j];
                perm[j] = t;
                i += 1;
                j -= 1;
            }
            flips += 1;
            k = perm[0];
        }
        free(perm);
        if (flips > maxflips) {
            maxflips = flips;
        }
        if (permcount % 2 == 0) {
            checksum += flips;
        } else {
            checksum -= flips;
        }
        int advanced = 0;
        while (!advanced) {
            if (r == n) {
                done = 1;
                break;
            }
            long long first = perm1[0];
            for (int i = 0; i < r; i++) {
                perm1[i] = perm1[i + 1];
            }
            perm1[r] = first;
            count[r] -= 1;
            if (count[r] > 0) {
                advanced = 1;
            } else {
                r += 1;
            }
        }
        permcount += 1;
    }
    printf("%lld\n", checksum);
    printf("Pfannkuchen(%d) = %d\n", n, maxflips);
    return 0;
}
