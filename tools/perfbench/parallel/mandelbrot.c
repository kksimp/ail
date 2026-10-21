/* C twin of mandelbrot.ail with OpenMP: one row per iteration. */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define MAX_ITER 500
static const char *CHARS = " .:-=+*#%@";

static int escape(double cr, double ci) {
    double zr = 0.0, zi = 0.0;
    int k = 0;
    while (k < MAX_ITER) {
        if (zr * zr + zi * zi > 4.0) break;
        double t = zr * zr - zi * zi + cr;
        zi = 2.0 * zr * zi + ci;
        zr = t;
        k += 1;
    }
    return k;
}

int main(int argc, char **argv) {
    long rows = argc > 1 ? atol(argv[1]) : 1000;
    long cols = rows * 2;
    char *buf = malloc(rows * (cols + 1));
#pragma omp parallel for schedule(dynamic, 4)
    for (long y = 0; y < rows; y++) {
        double ci = -1.2 + 2.4 * (double)y / (double)rows;
        char *line = buf + y * (cols + 1);
        for (long x = 0; x < cols; x++) {
            double cr = -2.0 + 3.0 * (double)x / (double)cols;
            int k = escape(cr, ci);
            line[x] = CHARS[k * 9 / MAX_ITER];
        }
        line[cols] = '\n';
    }
    fwrite(buf, 1, rows * (cols + 1), stdout);
    return 0;
}
