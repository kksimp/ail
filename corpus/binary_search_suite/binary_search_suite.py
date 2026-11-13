def lower_bound(xs, target):
    lo, hi = 0, len(xs)
    while lo < hi:
        mid = lo + (hi - lo) // 2
        if xs[mid] < target:
            lo = mid + 1
        else:
            hi = mid
    return lo


def upper_bound(xs, target):
    lo, hi = 0, len(xs)
    while lo < hi:
        mid = lo + (hi - lo) // 2
        if xs[mid] <= target:
            lo = mid + 1
        else:
            hi = mid
    return lo


def find_first(xs, target):
    i = lower_bound(xs, target)
    if i < len(xs) and xs[i] == target:
        return i
    return None


def count_of(xs, target):
    return upper_bound(xs, target) - lower_bound(xs, target)


def isqrt(n):
    lo, hi = 0, n + 1
    while hi - lo > 1:
        mid = lo + (hi - lo) // 2
        if mid * mid <= n:
            lo = mid
        else:
            hi = mid
    return lo


def days_needed(weights, cap):
    days = 1
    load = 0
    for w in weights:
        if load + w > cap:
            days += 1
            load = 0
        load += w
    return days


def min_capacity(weights, deadline):
    lo = max(weights)
    hi = sum(weights)
    while lo < hi:
        mid = lo + (hi - lo) // 2
        if days_needed(weights, mid) <= deadline:
            hi = mid
        else:
            lo = mid + 1
    return lo


def main():
    xs = [1, 3, 3, 3, 5, 8, 8, 13, 21, 21, 21, 21, 34]
    print("data: " + " ".join(str(x) for x in xs))
    for t in [0, 1, 3, 4, 8, 21, 34, 50]:
        i = find_first(xs, t)
        where = "missing" if i is None else f"first at {i}"
        print(f"{t}: lower={lower_bound(xs, t)} upper={upper_bound(xs, t)} count={count_of(xs, t)} {where}")
    for n in [0, 1, 15, 16, 99, 100, 1000000, 999999999999]:
        print(f"isqrt({n}) = {isqrt(n)}")
    weights = [3, 2, 2, 4, 1, 4, 5, 6, 2, 3]
    for d in [1, 2, 3, 5, 10]:
        print(f"{d} days: capacity {min_capacity(weights, d)}")


main()
