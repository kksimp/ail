/* C twin of spectralnorm.ail with OpenMP: one matrix row per iteration. */
#include <stdio.h>
#include <stdlib.h>
#include <math.h>

static double eval_a(long i, long j) {
    long d = (i + j) * (i + j + 1) / 2 + i + 1;
    return 1.0 / (double)d;
}

static void mul_av(const double *v, double *out, long n) {
#pragma omp parallel for schedule(static)
    for (long i = 0; i < n; i++) {
        double sum = 0.0;
        for (long j = 0; j < n; j++) sum += eval_a(i, j) * v[j];
        out[i] = sum;
    }
}

static void mul_atv(const double *v, double *out, long n) {
#pragma omp parallel for schedule(static)
    for (long i = 0; i < n; i++) {
        double sum = 0.0;
        for (long j = 0; j < n; j++) sum += eval_a(j, i) * v[j];
        out[i] = sum;
    }
}

static void mul_ata(const double *v, double *out, double *tmp, long n) {
    mul_av(v, tmp, n);
    mul_atv(tmp, out, n);
}

int main(int argc, char **argv) {
    long n = argc > 1 ? atol(argv[1]) : 1500;
    double *u = malloc(n * sizeof *u), *v = calloc(n, sizeof *v), *tmp = malloc(n * sizeof *tmp);
    for (long i = 0; i < n; i++) u[i] = 1.0;
    for (int it = 0; it < 10; it++) {
        mul_ata(u, v, tmp, n);
        mul_ata(v, u, tmp, n);
    }
    double vbv = 0.0, vv = 0.0;
    for (long i = 0; i < n; i++) {
        vbv += u[i] * v[i];
        vv += v[i] * v[i];
    }
    printf("%.9f\n", sqrt(vbv / vv));
    return 0;
}
