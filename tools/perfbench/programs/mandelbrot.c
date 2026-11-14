/* Mandelbrot as text: the same shades and pixel mapping as mandelbrot.ail. */
#include <stdio.h>
#include <stdlib.h>

#define MAX_ITER 50

static const char shades[] = " .:-=+*#%@";

static long long escape(double cr, double ci) {
    double zr = 0.0;
    double zi = 0.0;
    long long k = 0;
    while (k < MAX_ITER) {
        if (zr * zr + zi * zi > 4.0) {
            break;
        }
        double t = zr * zr - zi * zi + cr;
        zi = 2.0 * zr * zi + ci;
        zr = t;
        k += 1;
    }
    return k;
}

int main(int argc, char **argv) {
    long long rows = argc > 1 ? atoll(argv[1]) : 160;
    long long cols = rows * 2;
    char *line = malloc(cols + 2);
    for (long long y = 0; y < rows; y++) {
        double ci = -1.2 + 2.4 * (double)y / (double)rows;
        for (long long x = 0; x < cols; x++) {
            double cr = -2.0 + 3.0 * (double)x / (double)cols;
            long long k = escape(cr, ci);
            line[x] = shades[k * 9 / MAX_ITER];
        }
        line[cols] = '\n';
        line[cols + 1] = '\0';
        fputs(line, stdout);
    }
    free(line);
    return 0;
}
