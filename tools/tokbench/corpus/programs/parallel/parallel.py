WIDTH = 48
HEIGHT = 16
LIMIT = 200


def escape(cr, ci):
    zr = 0.0
    zi = 0.0
    n = 0
    while n < LIMIT and zr * zr + zi * zi <= 4.0:
        t = zr * zr - zi * zi + cr
        zi = 2.0 * zr * zi + ci
        zr = t
        n += 1
    return n


def shade(n):
    if n >= LIMIT:
        return "#"
    if n > 12:
        return "+"
    if n > 4:
        return "."
    return " "


def main():
    rows = []
    for y in range(HEIGHT):
        line = ""
        for x in range(WIDTH):
            cr = -2.2 + 3.0 * float(x) / float(WIDTH)
            ci = -1.2 + 2.4 * float(y) / float(HEIGHT)
            line = line + shade(escape(cr, ci))
        rows.append(line)
    for row in rows:
        print(row)
    steps = 0
    for y in range(HEIGHT):
        steps += escape(-0.5, float(y) / float(HEIGHT))
    print(f"steps: {steps}")


if __name__ == "__main__":
    main()
