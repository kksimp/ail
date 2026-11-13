def sum_to(n):
    return sum(range(1, n + 1))


def factorial(n):
    t = 1
    for i in range(2, n + 1):
        t *= i
    return t


def fib(n):
    a, b = 0, 1
    for _ in range(n):
        a, b = b, a + b
    return a


def run_job(idx, spec):
    parts = spec.split(":")
    if len(parts) != 2:
        return (idx, spec, False, "malformed spec")
    try:
        n = int(parts[1])
    except ValueError:
        return (idx, spec, False, f"bad argument '{parts[1]}'")
    if n < 0 or n > 90:
        return (idx, spec, False, f"argument {n} out of range")
    if parts[0] == "fact" and n > 20:
        return (idx, spec, False, f"factorial overflows at {n}")
    kind = parts[0]
    if kind == "sum":
        return (idx, spec, True, str(sum_to(n)))
    if kind == "fact":
        return (idx, spec, True, str(factorial(n)))
    if kind == "fib":
        return (idx, spec, True, str(fib(n)))
    return (idx, spec, False, f"unknown kind '{kind}'")


def run_batch(first, specs):
    return sorted((run_job(first + i, s) for i, s in enumerate(specs)), key=lambda r: r[0])


def main():
    specs = ["sum:80", "fact:10", "fib:30", "fact:25", "pow:2", "sum:abc", "fib:10", "sum", "fact:5"]
    size = 3
    failed = 0
    start = 0
    batch_no = 1
    while start < len(specs):
        stop = min(start + size, len(specs))
        print(f"batch {batch_no}")
        for idx, spec, ok, out in run_batch(start, specs[start:stop]):
            status = "ok  " if ok else "FAIL"
            print(f"  job {idx} {status} {spec:<8} {out}")
            if not ok:
                failed += 1
        start = stop
        batch_no += 1
    print(f"{len(specs) - failed} ok, {failed} failed")


main()
