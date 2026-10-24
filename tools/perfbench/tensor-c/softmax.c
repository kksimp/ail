// C twin of softmax.ail: row softmax of a 512x512 tensor, 100 rounds each for double and float.
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
int main(void) {
    int n = 512;
    double *x = malloc(sizeof(double) * n * n), *y = malloc(sizeof(double) * n * n);
    float *f = malloc(sizeof(float) * n * n), *z = malloc(sizeof(float) * n * n);
    for (int i = 0; i < n * n; i++) { x[i] = (double)((i * 7919) % 1009) * 0.01; f[i] = (float)x[i]; }
    long hits = 0, mass = 0;
    for (int r = 0; r < 100; r++) {
        for (int i = 0; i < n; i++) {
            double *xr = x + i * n, *yr = y + i * n, mx = xr[0], s = 0.0;
            int am = 0;
            for (int j = 1; j < n; j++) if (xr[j] > mx) { mx = xr[j]; am = j; }
            for (int j = 0; j < n; j++) { yr[j] = exp(xr[j] - mx); s += yr[j]; }
            double t = 0.0;
            for (int j = 0; j < n; j++) { yr[j] /= s; t += yr[j]; }
            hits += am; mass += (long)llround(t * 1000.0);
            float *fr = f + i * n, *zr = z + i * n, fm = fr[0], fs = 0.0f;
            int fa = 0;
            for (int j = 1; j < n; j++) if (fr[j] > fm) { fm = fr[j]; fa = j; }
            for (int j = 0; j < n; j++) { zr[j] = expf(fr[j] - fm); fs += zr[j]; }
            float ft = 0.0f;
            for (int j = 0; j < n; j++) { zr[j] /= fs; ft += zr[j]; }
            hits += fa; mass += (long)llroundf(ft * 1000.0f);
        }
    }
    printf("%ld\n%ld\n", hits, mass);
    return 0;
}
