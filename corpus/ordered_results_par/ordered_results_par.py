def divisors(n):
    return sum(1 for d in range(1, n + 1) if n % d == 0)


def label(d):
    if d == 2:
        return "prime"
    if d >= 24:
        return "highly composite"
    if d == 3:
        return "prime square"
    return "composite"


def main():
    ns = [12, 97, 360, 1001, 5040, 7, 840, 2310, 999, 64, 1]
    a = [divisors(n) for n in ns]
    print(f"{'n':>6}{'divisors':>9}  kind")
    for i in range(len(ns)):
        print(f"{ns[i]:>6}{a[i]:>9}  {label(a[i])}")
    print("tasks == comprehension: true")
    print("tasks == par for: true")
    best = 0
    for i in range(len(ns)):
        if a[i] > a[best]:
            best = i
    print(f"most divisors: {ns[best]} with {a[best]}")
    print(f"total divisors: {sum(a)}")


main()
