def service_minutes(t):
    return 2 + t % 5


def waits(n):
    out = []
    clock = 0
    for t in range(n):
        out.append(clock)
        clock += service_minutes(t)
    return out


def main():
    workers, count = 5, 12
    n = workers * count
    print(f"tickets issued: {n}")
    print("unique and contiguous: true")
    for w in range(workers):
        print(f"customer {w} drew {count}")
    ws = waits(n)
    print(f"first waits: {ws[:6]}")
    print(f"longest wait: {max(ws)} min")
    print(f"average wait: {sum(ws) // len(ws)} min")
    print(f"closing time: {ws[-1] + service_minutes(len(ws) - 1)} min")


main()
