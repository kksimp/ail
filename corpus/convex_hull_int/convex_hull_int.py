def cross(o, a, b):
    return (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0])


def unique_sorted(pts):
    ordered = sorted(pts, key=lambda q: q[0] * 100000 + q[1])
    out = []
    for p in ordered:
        if not out or out[len(out) - 1] != p:
            out.append(p)
    return out


def chain(pts):
    hull = []
    for p in pts:
        while len(hull) >= 2 and cross(hull[len(hull) - 2], hull[len(hull) - 1], p) <= 0:
            hull.pop()
        hull.append(p)
    return hull


def convex_hull(pts):
    sorted_pts = unique_sorted(pts)
    if len(sorted_pts) < 3:
        return sorted_pts
    lower = chain(sorted_pts)
    upper = chain(list(reversed(sorted_pts)))
    lower.pop()
    upper.pop()
    lower.extend(upper)
    return lower


def twice_area(hull):
    total = 0
    for i in range(len(hull)):
        a = hull[i]
        b = hull[(i + 1) % len(hull)]
        total += a[0] * b[1] - b[0] * a[1]
    return total


def label(p):
    return f"({p[0]},{p[1]})"


def report(name, pts):
    hull = convex_hull(pts)
    print(f"{name}: {len(pts)} points, hull has {len(hull)}")
    print(f"  {' '.join(label(p) for p in hull)}")
    print(f"  doubled area {twice_area(hull)}")


def main():
    scattered = [(i * 7 % 11, i * 5 % 13) for i in range(20)]
    report("scattered", scattered)
    square = [(0, 0), (4, 0), (4, 4), (0, 4),
              (2, 2), (2, 0), (4, 2), (1, 3), (0, 0)]
    report("square with extras", square)
    line = [(3, 3), (1, 1), (2, 2), (5, 5)]
    report("collinear", line)
    report("pair", [(1, 2), (1, 2), (3, 4)])
    ring = [(5, 0), (3, 4), (-3, 4), (-5, 0),
            (-3, -4), (3, -4), (0, 0), (1, 1)]
    report("ring", ring)


main()
