def next_seed(s):
    return (s * 1103515245 + 12345) % 2147483648


def delay_for(base, cap, k, seed):
    raw = min(base * (1 << k), cap)
    return raw + seed // 65536 % (raw // 4 + 1)


def run_retry(base, cap, max_attempts, fail_until, seed0):
    seed = seed0
    waited = 0
    delays = []
    for n in range(1, max_attempts + 1):
        if n >= fail_until:
            return True, n, waited, delays
        if n < max_attempts:
            seed = next_seed(seed)
            d = delay_for(base, cap, n - 1, seed)
            delays.append(d)
            waited += d
    return False, max_attempts, waited, delays


def show(label, o):
    ok, attempts, waited, delays = o
    status = "ok" if ok else "gave up"
    print(f"{label}: {status} after {attempts} attempts, waited {waited} ms")
    print(f"  delays {delays}")


show("first try", run_retry(100, 1000, 6, 1, 7))
show("third try", run_retry(100, 1000, 6, 3, 7))
show("last try", run_retry(100, 1000, 6, 6, 7))
show("hopeless", run_retry(100, 1000, 6, 99, 7))
show("capped", run_retry(50, 120, 5, 99, 11))
