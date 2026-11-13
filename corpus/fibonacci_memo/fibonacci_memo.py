def fib(n, memo):
    if n < 2:
        return n
    if n in memo:
        return memo[n]
    v = fib(n - 1, memo) + fib(n - 2, memo)
    memo[n] = v
    return v


def fib_loop(n):
    a = 0
    b = 1
    for _ in range(n):
        a, b = b, a + b
    return a


def naive_calls(n):
    if n < 2:
        return 1
    return 1 + naive_calls(n - 1) + naive_calls(n - 2)


def zeckendorf(n):
    fibs = [1, 2]
    while fibs[-1] <= n:
        fibs.append(fibs[-2] + fibs[-1])
    parts = []
    left = n
    for k in range(len(fibs) - 1, -1, -1):
        if fibs[k] <= left:
            parts.append(fibs[k])
            left -= fibs[k]
    return parts


def even_sum(limit):
    a = 1
    b = 2
    total = 0
    while b < limit:
        if b % 2 == 0:
            total += b
        a, b = b, a + b
    return total


def main():
    memo = {}
    firsts = []
    for i in range(15):
        firsts.append(fib(i, memo))
    print("first 15: " + " ".join(str(x) for x in firsts))
    f50 = fib(50, memo)
    f90 = fib(90, memo)
    print(f"fib(50) = {f50}")
    print(f"fib(90) = {f90}")
    print(f"cache entries: {len(memo)}")
    agrees = fib_loop(90) == f90
    print(f"loop agrees: {'true' if agrees else 'false'}")
    print(f"naive calls for fib(20): {naive_calls(20)}")
    for n in [4, 100, 1000, 12345]:
        parts = zeckendorf(n)
        print(f"{n} = " + " + ".join(str(x) for x in parts))
    print(f"even terms below 4000000 sum to {even_sum(4000000)}")


main()
