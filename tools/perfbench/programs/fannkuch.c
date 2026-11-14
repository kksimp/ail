/* Fannkuch-redux: the same permutation walk as fannkuch.ail. */
#include <stdio.h>
#include <stdlib.h>

int main(int argc, char **argv) {
    int n = argc > 1 ? atoi(argv[1]) : 8;
    int *perm1 = malloc(n * sizeof(int));
    int *perm = malloc(n * sizeof(int));
    int *count = malloc(n * sizeof(int));
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
        for (int i = 0; i < n; i++) {
            perm[i] = perm1[i];
        }
        int flips = 0;
        int k = perm[0];
        while (k != 0) {
            int i = 0;
            int j = k;
            while (i < j) {
                int t = perm[i];
                perm[i] = perm[j];
                perm[j] = t;
                i += 1;
                j -= 1;
            }
            flips += 1;
            k = perm[0];
        }
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
            int first = perm1[0];
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
