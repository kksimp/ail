// C twin of f32_chain.ail: an f32 elementwise chain on 2^22 elements, 50 rounds.
#include <stdio.h>
#include <stdlib.h>
int main(void) {
    int n = 4194304;
    float *x = malloc(sizeof(float) * n), *y = malloc(sizeof(float) * n);
    for (int i = 0; i < n; i++) x[i] = (float)(i % 17) * 0.25f;
    double total = 0.0;
    for (int r = 0; r < 50; r++) {
        for (int i = 0; i < n; i++) y[i] = (x[i] * x[i] + x[i]) * 0.5f - x[i];
        double s = 0.0;
        for (int i = 0; i < n; i++) s += y[i];
        total += s;
    }
    printf("%.1f\n", total);
    return 0;
}
