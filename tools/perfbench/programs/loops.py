# Python twin of loops.ail, for perfbench's ratio column.

def collatz(start):
    n = start
    steps = 0
    while n != 1:
        if n % 2 == 0:
            n = n // 2
        else:
            n = 3 * n + 1
        steps += 1
    return steps


def main():
    total = 0
    for i in range(1, 30000):
        total += collatz(i)
    print(total)

    xs = []
    for i in range(0, 200000):
        xs.append(i * 3 % 1000)
    s = 0
    for x in xs:
        s += x
    print(s)

    counts = {0: 0}
    for i in range(0, 100000):
        k = i % 500
        counts[k] = counts.get(k, 0) + 1
    print(len(counts))

    t = ""
    for i in range(0, 20000):
        t = f"{t}{i % 10}"
    print(len(t))


main()
