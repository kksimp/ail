/* N-body: five bodies under gravity, symplectic advance (benchmarks-game shape). */
#include <stdio.h>
#include <stdlib.h>
#include <math.h>

#define NB 5
#define PI 3.141592653589793
#define SOLAR_MASS (4.0 * PI * PI)
#define DAYS 365.24

/* Fields per body: 0-2 position, 3-5 velocity, 6 mass. */
static double s[NB * 7];

static void init_bodies(void) {
    double init[NB * 7] = {
        0.0, 0.0, 0.0, 0.0, 0.0, 0.0, SOLAR_MASS,
        4.84143144246472090e+00, -1.16032004402742839e+00, -1.03622044471123109e-01,
            1.66007664274403694e-03 * DAYS, 7.69901118419740425e-03 * DAYS,
            -6.90460016972063023e-05 * DAYS, 9.54791938424326609e-04 * SOLAR_MASS,
        8.34336671824457987e+00, 4.12479856412430479e+00, -4.03523417114321381e-01,
            -2.76742510726862411e-03 * DAYS, 4.99852801234917238e-03 * DAYS,
            2.30417297573763929e-05 * DAYS, 2.85885980666130812e-04 * SOLAR_MASS,
        1.28943695621391310e+01, -1.51111514016986312e+01, -2.23307578892655734e-01,
            2.96460137564761618e-03 * DAYS, 2.37847173959480950e-03 * DAYS,
            -2.96589568540237556e-05 * DAYS, 4.36624404335156298e-05 * SOLAR_MASS,
        1.53796971148509165e+01, -2.59193146099879641e+01, 1.79258772950371181e-01,
            2.68067772490389322e-03 * DAYS, 1.62824170038242295e-03 * DAYS,
            -9.51592254519715870e-05 * DAYS, 5.15138902046611451e-05 * SOLAR_MASS,
    };
    for (int i = 0; i < NB * 7; i++) {
        s[i] = init[i];
    }
}

static void offset_momentum(void) {
    double px = 0.0, py = 0.0, pz = 0.0;
    for (int b = 0; b < NB; b++) {
        double m = s[b * 7 + 6];
        px += s[b * 7 + 3] * m;
        py += s[b * 7 + 4] * m;
        pz += s[b * 7 + 5] * m;
    }
    s[3] = -px / SOLAR_MASS;
    s[4] = -py / SOLAR_MASS;
    s[5] = -pz / SOLAR_MASS;
}

static double energy(void) {
    double e = 0.0;
    for (int i = 0; i < NB; i++) {
        int bi = i * 7;
        double vx = s[bi + 3], vy = s[bi + 4], vz = s[bi + 5];
        e += 0.5 * s[bi + 6] * (vx * vx + vy * vy + vz * vz);
        for (int j = i + 1; j < NB; j++) {
            int bj = j * 7;
            double dx = s[bi] - s[bj];
            double dy = s[bi + 1] - s[bj + 1];
            double dz = s[bi + 2] - s[bj + 2];
            double d = sqrt(dx * dx + dy * dy + dz * dz);
            e -= s[bi + 6] * s[bj + 6] / d;
        }
    }
    return e;
}

static void advance(double dt) {
    for (int i = 0; i < NB; i++) {
        int bi = i * 7;
        for (int j = i + 1; j < NB; j++) {
            int bj = j * 7;
            double dx = s[bi] - s[bj];
            double dy = s[bi + 1] - s[bj + 1];
            double dz = s[bi + 2] - s[bj + 2];
            double d2 = dx * dx + dy * dy + dz * dz;
            double mag = dt / (d2 * sqrt(d2));
            double mi = s[bi + 6] * mag;
            double mj = s[bj + 6] * mag;
            s[bi + 3] -= dx * mj;
            s[bi + 4] -= dy * mj;
            s[bi + 5] -= dz * mj;
            s[bj + 3] += dx * mi;
            s[bj + 4] += dy * mi;
            s[bj + 5] += dz * mi;
        }
    }
    for (int b = 0; b < NB; b++) {
        int bb = b * 7;
        s[bb] += dt * s[bb + 3];
        s[bb + 1] += dt * s[bb + 4];
        s[bb + 2] += dt * s[bb + 5];
    }
}

int main(int argc, char **argv) {
    long long steps = argc > 1 ? atoll(argv[1]) : 20000;
    init_bodies();
    offset_momentum();
    printf("%.9f\n", energy());
    for (long long k = 0; k < steps; k++) {
        advance(0.01);
    }
    printf("%.9f\n", energy());
    return 0;
}
