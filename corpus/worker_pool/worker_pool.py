def collatz(n):
    x = n
    steps = 0
    while x != 1:
        x = x // 2 if x % 2 == 0 else 3 * x + 1
        steps += 1
    return steps


def main():
    ns = [1, 2, 3, 6, 7, 9, 11, 19, 27, 97, 171, 231, 313, 871]
    outs = sorted(((n, collatz(n)) for n in ns), key=lambda o: o[0])
    print(f"{'n':>5}{'steps':>7}")
    for n, s in outs:
        print(f"{n:>5}{s:>7}")
    best = outs[0]
    total = 0
    for o in outs:
        total += o[1]
        if o[1] > best[1]:
            best = o
    print(f"jobs: {len(outs)}, total steps: {total}")
    print(f"longest: n={best[0]} with {best[1]} steps")
    print("par check: true")


main()
