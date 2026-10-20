# Python twin of parsum.ail (one thread, as CPython runs it).
def steps(n):
    x = n
    k = 0
    while x != 1:
        if x % 2 == 0:
            x = x // 2
        else:
            x = 3 * x + 1
        k += 1
    return k


def main():
    n = 100000
    lengths = [0] * n
    total = 0
    for i in range(n):
        s = steps(i + 1)
        lengths[i] = s
        total += s
    longest = 0
    for i in range(n):
        if lengths[i] > lengths[longest]:
            longest = i
    print(f"total steps: {total}")
    print(f"longest: {longest + 1} with {lengths[longest]} steps")


main()
