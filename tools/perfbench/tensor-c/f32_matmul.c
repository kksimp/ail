// C twin of f32_matmul.ail: a 512x512 f32 product, ten rounds, a plain triple loop (i, k, j).
#include <stdio.h>
#include <stdlib.h>
#ifdef BLAS
#include <Accelerate/Accelerate.h>
#endif
int main(void) {
    int n = 512;
    float *a = malloc(sizeof(float) * n * n), *b = malloc(sizeof(float) * n * n), *c = malloc(sizeof(float) * n * n);
    for (int i = 0; i < n * n; i++) { a[i] = (float)(i % 7) * 0.25f; b[i] = (float)(i % 5) * 0.5f; }
    double total = 0.0;
    for (int r = 0; r < 10; r++) {
#ifdef BLAS
        cblas_sgemm(CblasRowMajor, CblasNoTrans, CblasNoTrans, n, n, n, 1.0f, a, n, b, n, 0.0f, c, n);
#else
        for (int i = 0; i < n * n; i++) c[i] = 0.0f;
        for (int i = 0; i < n; i++)
            for (int k = 0; k < n; k++) {
                float x = a[i * n + k];
                for (int j = 0; j < n; j++) c[i * n + j] += x * b[k * n + j];
            }
#endif
        double s = 0.0, m = -1e300;
        for (int i = 0; i < n * n; i++) { s += c[i]; if (c[i] > m) m = c[i]; }
        total += s + m;
    }
    printf("%.1f\n", total);
    return 0;
}
