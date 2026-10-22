/* C twin of nbody.ail with OpenMP: accelerations per body, then the update. */
#include <stdio.h>
#include <stdlib.h>
#include <math.h>

int main(int argc, char **argv) {
    long n = argc > 1 ? atol(argv[1]) : 1500;
    double *px = calloc(n, 8), *py = calloc(n, 8), *pz = calloc(n, 8);
    double *vx = calloc(n, 8), *vy = calloc(n, 8), *vz = calloc(n, 8);
    double *ax = calloc(n, 8), *ay = calloc(n, 8), *az = calloc(n, 8), *mass = calloc(n, 8);
    for (long i = 0; i < n; i++) {
        double t = (double)i * 0.37;
        px[i] = 100.0 * (double)(i % 17) + t;
        py[i] = 90.0 * (double)(i % 13) - t;
        pz[i] = 80.0 * (double)(i % 11) + 0.5 * t;
        mass[i] = 1.0 + (double)(i % 5);
    }
    for (int step = 0; step < 10; step++) {
#pragma omp parallel for schedule(static)
        for (long i = 0; i < n; i++) {
            double sx = 0.0, sy = 0.0, sz = 0.0;
            for (long j = 0; j < n; j++) {
                if (j != i) {
                    double dx = px[j] - px[i], dy = py[j] - py[i], dz = pz[j] - pz[i];
                    double d2 = dx * dx + dy * dy + dz * dz + 0.01;
                    double inv = mass[j] / (d2 * sqrt(d2));
                    sx += dx * inv;
                    sy += dy * inv;
                    sz += dz * inv;
                }
            }
            ax[i] = sx;
            ay[i] = sy;
            az[i] = sz;
        }
#pragma omp parallel for schedule(static)
        for (long i = 0; i < n; i++) {
            vx[i] = vx[i] + 0.01 * ax[i];
            vy[i] = vy[i] + 0.01 * ay[i];
            vz[i] = vz[i] + 0.01 * az[i];
            px[i] = px[i] + 0.01 * vx[i];
            py[i] = py[i] + 0.01 * vy[i];
            pz[i] = pz[i] + 0.01 * vz[i];
        }
    }
    double sum = 0.0;
    for (long i = 0; i < n; i++) sum += px[i] + py[i] + pz[i];
    printf("%.6f\n", sum);
    return 0;
}
