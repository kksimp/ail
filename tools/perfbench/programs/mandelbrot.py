# Python twin of mandelbrot.ail (CPython 3, one thread).
import sys

SHADES = " .:-=+*#%@"
MAX_ITER = 50


def escape(cr, ci):
    zr = 0.0
    zi = 0.0
    k = 0
    while k < MAX_ITER:
        if zr * zr + zi * zi > 4.0:
            break
        t = zr * zr - zi * zi + cr
        zi = 2.0 * zr * zi + ci
        zr = t
        k += 1
    return k


def main():
    rows = int(sys.argv[1]) if len(sys.argv) > 1 else 160
    cols = rows * 2
    for y in range(rows):
        ci = -1.2 + 2.4 * y / rows
        line = []
        for x in range(cols):
            cr = -2.0 + 3.0 * x / cols
            k = escape(cr, ci)
            line.append(SHADES[k * 9 // MAX_ITER])
        print("".join(line))


main()
