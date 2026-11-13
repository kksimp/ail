def wanted(flt, topic):
    return flt == "*" or flt == topic


def mix(s, m):
    return (s * 31 + m[1] * len(m[2])) % 1000003


def make_messages(n):
    topics = ["news", "sport", "weather"]
    msgs = []
    for i in range(1, n + 1):
        t = topics[i % 3]
        msgs.append((t, i, f"{t}-update-{i * i}"))
    return msgs


def subscriber(name, flt, msgs):
    got = 0
    s = 0
    last = "-"
    for m in msgs:
        if wanted(flt, m[0]):
            got += 1
            s = mix(s, m)
            last = m[2]
    return (name, flt, got, s, last)


def main():
    msgs = make_messages(30)
    names = ["alice", "bob", "carol", "dave"]
    filters = ["*", "news", "sport", "tides"]
    print(f"publishing {len(msgs)} messages to {len(names)} subscribers")
    reports = sorted((subscriber(n, f, msgs) for n, f in zip(names, filters)), key=lambda r: r[0])
    for name, flt, got, s, last in reports:
        print(f"{name:<6}{flt:<7}{got:>3} msgs  sum {s:>7}  last {last}")
    print(f"total deliveries: {sum(r[2] for r in reports)}")


main()
