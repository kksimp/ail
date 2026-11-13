def parse(text):
    pts = []
    for tok in text.split():
        p = tok.split(",")
        pts.append((int(p[0]), int(p[1])))
    return pts


def gcd(a, b):
    x = abs(a)
    y = abs(b)
    while y != 0:
        t = x % y
        x = y
        y = t
    return x


def twice_area(pts):
    total = 0
    n = len(pts)
    for i in range(n):
        a = pts[i]
        b = pts[(i + 1) % n]
        total += a[0] * b[1] - b[0] * a[1]
    return total


def boundary_points(pts):
    total = 0
    n = len(pts)
    for i in range(n):
        a = pts[i]
        b = pts[(i + 1) % n]
        total += gcd(b[0] - a[0], b[1] - a[1])
    return total


def is_convex(pts):
    n = len(pts)
    pos = 0
    neg = 0
    for i in range(n):
        a = pts[i]
        b = pts[(i + 1) % n]
        c = pts[(i + 2) % n]
        turn = (b[0] - a[0]) * (c[1] - b[1]) - (b[1] - a[1]) * (c[0] - b[0])
        if turn > 0:
            pos += 1
        elif turn < 0:
            neg += 1
    return pos == 0 or neg == 0


def halves(twice):
    a = abs(twice)
    if a % 2 == 0:
        return f"{a // 2}"
    return f"{a // 2}.5"


def report(name, text):
    pts = parse(text)
    s = twice_area(pts)
    b = boundary_points(pts)
    inner = (abs(s) - b + 2) // 2
    way = "counter-clockwise" if s > 0 else "clockwise"
    shape = "convex" if is_convex(pts) else "concave"
    print(f"{name}: {len(pts)} vertices, {way}, {shape}")
    print(f"  area {halves(s)}, boundary {b}, interior {inner}")


def main():
    report("rectangle", "0,0 4,0 4,3 0,3")
    report("triangle", "0,0 5,0 0,5")
    report("slanted triangle", "1,1 6,3 2,7")
    report("L shape", "0,0 6,0 6,2 2,2 2,5 0,5")
    report("diamond", "3,0 6,3 3,6 0,3")
    report("clockwise square", "0,0 0,4 4,4 4,0")
    report("arrow", "0,0 8,4 0,8 3,4")


main()
