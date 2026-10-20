# Python twin of race_smoke.ail.

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


main()
