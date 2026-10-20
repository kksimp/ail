/* Spectral norm: the same power iteration as spectralnorm.ail. */
#include <stdio.h>
#include <stdlib.h>
#include <math.h>

static double eval_a(long i, long j) {
    long d = (i + j) * (i + j + 1) / 2 + i + 1;
    return 1.0 / (double)d;
}

static void mul_av(const double *v, double *out, long n) {
    for (long i = 0; i < n; i++) {
        double sum = 0.0;
        for (long j = 0; j < n; j++) {
            sum += eval_a(i, j) * v[j];
        }
        out[i] = sum;
    }
}

static void mul_atv(const double *v, double *out, long n) {
    for (long i = 0; i < n; i++) {
        double sum = 0.0;
        for (long j = 0; j < n; j++) {
            sum += eval_a(j, i) * v[j];
        }
        out[i] = sum;
    }
}

static void mul_ata(const double *v, double *out, double *tmp, long n) {
    mul_av(v, tmp, n);
    mul_atv(tmp, out, n);
}

int main(int argc, char **argv) {
    long n = argc > 1 ? atol(argv[1]) : 200;
    double *u = malloc(n * sizeof(double));
    double *v = malloc(n * sizeof(double));
    double *tmp = malloc(n * sizeof(double));
    for (long i = 0; i < n; i++) {
        u[i] = 1.0;
        v[i] = 0.0;
    }
    for (int k = 0; k < 10; k++) {
        mul_ata(u, v, tmp, n);
        mul_ata(v, u, tmp, n);
    }
    double vbv = 0.0;
    double vv = 0.0;
    for (long i = 0; i < n; i++) {
        vbv += u[i] * v[i];
        vv += v[i] * v[i];
    }
    printf("%.9f\n", sqrt(vbv / vv));
    return 0;
}
