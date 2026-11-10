/* C twin of raytrace.ail. */
#include <math.h>
#include <stdio.h>
#include <stdlib.h>

typedef struct { double x, y, z; } Vec;
typedef struct { Vec c; double r; Vec col; double refl; } Sphere;

static Vec V(double x, double y, double z) { Vec v = {x, y, z}; return v; }
static Vec add(Vec a, Vec o) { return V(a.x + o.x, a.y + o.y, a.z + o.z); }
static Vec sub(Vec a, Vec o) { return V(a.x - o.x, a.y - o.y, a.z - o.z); }
static Vec scale(Vec a, double k) { return V(a.x * k, a.y * k, a.z * k); }
static double dot(Vec a, Vec o) { return a.x * o.x + a.y * o.y + a.z * o.z; }
static Vec unit(Vec a) { double k = 1.0 / sqrt(dot(a, a)); return scale(a, k); }

static double hit(const Sphere *s, Vec o, Vec d) {
    Vec oc = sub(o, s->c);
    double b = dot(oc, d);
    double c = dot(oc, oc) - s->r * s->r;
    double disc = b * b - c;
    if (disc < 0.0) return -1.0;
    double sq = sqrt(disc);
    double t = -b - sq;
    if (t > 0.001) return t;
    double t2 = -b + sq;
    if (t2 > 0.001) return t2;
    return -1.0;
}

static Sphere spheres[4];

static Vec trace(Vec light, Vec o, Vec d, int depth) {
    double best = -1.0;
    int idx = -1;
    for (int i = 0; i < 4; i++) {
        double t = hit(&spheres[i], o, d);
        if (t > 0.0 && (best < 0.0 || t < best)) { best = t; idx = i; }
    }
    if (idx < 0) {
        double k = 0.5 * (d.y + 1.0);
        return add(scale(V(1.0, 1.0, 1.0), 1.0 - k), scale(V(0.5, 0.7, 1.0), k));
    }
    Sphere s = spheres[idx];
    Vec p = add(o, scale(d, best));
    Vec n = unit(sub(p, s.c));
    Vec base = s.col;
    if (idx == 3) {
        int parity = ((int)(p.x + 100.0) + (int)(p.z + 100.0)) % 2;
        if (parity == 0) base = V(0.2, 0.2, 0.2);
    }
    int lit = 0;
    for (int i = 0; i < 4; i++) if (hit(&spheres[i], p, light) > 0.0) lit = 1;
    double diff = dot(n, light);
    if (diff < 0.0 || lit) diff = 0.0;
    Vec col = scale(base, 0.15 + 0.85 * diff);
    if (depth < 2 && s.refl > 0.0) {
        Vec r = sub(d, scale(n, 2.0 * dot(d, n)));
        Vec rc = trace(light, p, r, depth + 1);
        col = add(scale(col, 1.0 - s.refl), scale(rc, s.refl));
    }
    return col;
}

static long long chan(double v) {
    double c = v;
    if (c < 0.0) c = 0.0;
    if (c > 1.0) c = 1.0;
    return (long long)(c * 255.0);
}

int main(int argc, char **argv) {
    long long n = argc > 1 ? atoll(argv[1]) : 400;
    spheres[0] = (Sphere){V(0.0, 0.0, -5.0), 1.0, V(0.9, 0.2, 0.2), 0.3};
    spheres[1] = (Sphere){V(2.0, 0.2, -6.0), 1.2, V(0.2, 0.8, 0.3), 0.5};
    spheres[2] = (Sphere){V(-2.2, -0.3, -4.5), 0.7, V(0.3, 0.3, 0.9), 0.2};
    spheres[3] = (Sphere){V(0.0, -1001.0, -5.0), 1000.0, V(0.8, 0.8, 0.8), 0.25};
    Vec light = unit(V(0.5, 1.0, 0.3));
    Vec eye = V(0.0, 0.0, 0.0);
    long long cks = 0, total = 0;
    for (long long j = 0; j < n; j++)
        for (long long i = 0; i < n; i++) {
            double u = ((double)i + 0.5) / (double)n * 2.0 - 1.0;
            double v = 1.0 - ((double)j + 0.5) / (double)n * 2.0;
            Vec d = unit(V(u, v, -1.5));
            Vec c = trace(light, eye, d, 0);
            long long r = chan(c.x), g = chan(c.y), b = chan(c.z);
            total += r + g + b;
            cks = (cks * 31 + r * 65536 + g * 256 + b) % 1000000007;
        }
    printf("size %lld\ntotal %lld\nchecksum %lld\n", n, total, cks);
    return 0;
}
