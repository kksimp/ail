# Python twin of nbody.ail (CPython 3, one thread).
import math
import sys

PI = 3.141592653589793
SOLAR_MASS = 4.0 * PI * PI
DAYS = 365.24


def bodies():
    return [
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
    ]


def offset_momentum(s):
    px = 0.0
    py = 0.0
    pz = 0.0
    for b in range(5):
        m = s[b * 7 + 6]
        px += s[b * 7 + 3] * m
        py += s[b * 7 + 4] * m
        pz += s[b * 7 + 5] * m
    s[3] = -px / SOLAR_MASS
    s[4] = -py / SOLAR_MASS
    s[5] = -pz / SOLAR_MASS


def energy(s):
    e = 0.0
    for i in range(5):
        bi = i * 7
        vx = s[bi + 3]
        vy = s[bi + 4]
        vz = s[bi + 5]
        e += 0.5 * s[bi + 6] * (vx * vx + vy * vy + vz * vz)
        for j in range(i + 1, 5):
            bj = j * 7
            dx = s[bi] - s[bj]
            dy = s[bi + 1] - s[bj + 1]
            dz = s[bi + 2] - s[bj + 2]
            d = math.sqrt(dx * dx + dy * dy + dz * dz)
            e -= s[bi + 6] * s[bj + 6] / d
    return e


def advance(s, dt):
    for i in range(5):
        bi = i * 7
        for j in range(i + 1, 5):
            bj = j * 7
            dx = s[bi] - s[bj]
            dy = s[bi + 1] - s[bj + 1]
            dz = s[bi + 2] - s[bj + 2]
            d2 = dx * dx + dy * dy + dz * dz
            mag = dt / (d2 * math.sqrt(d2))
            mi = s[bi + 6] * mag
            mj = s[bj + 6] * mag
            s[bi + 3] -= dx * mj
            s[bi + 4] -= dy * mj
            s[bi + 5] -= dz * mj
            s[bj + 3] += dx * mi
            s[bj + 4] += dy * mi
            s[bj + 5] += dz * mi
    for b in range(5):
        bb = b * 7
        s[bb] += dt * s[bb + 3]
        s[bb + 1] += dt * s[bb + 4]
        s[bb + 2] += dt * s[bb + 5]


def main():
    steps = int(sys.argv[1]) if len(sys.argv) > 1 else 20000
    s = bodies()
    offset_momentum(s)
    print(f"{energy(s):.9f}")
    for _ in range(steps):
        advance(s, 0.01)
    print(f"{energy(s):.9f}")


main()
