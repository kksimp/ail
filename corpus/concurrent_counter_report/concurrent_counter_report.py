def level_of(i):
    levels = ["info", "warn", "error", "debug"]
    return levels[(i * 7 + i // 3) % 4]


def size_of(i):
    return 10 + (i * 13) % 50


def main():
    total = 240
    workers = 4
    hist = {}
    sums = []
    byte_total = 0
    longest = 0
    for wid in range(workers):
        events = 0
        mine = 0
        for i in range(total):
            if i % workers != wid:
                continue
            events += 1
            sz = size_of(i)
            mine += sz
            hist[level_of(i)] = hist.get(level_of(i), 0) + 1
            byte_total += sz
            longest = max(longest, sz)
        sums.append((wid, events, mine))
    for wid, events, mine in sums:
        print(f"worker {wid}: {events} events, {mine} bytes")
    for level in sorted(hist):
        print(f"{level:<6}{hist[level]:>4}")
    print(f"total events {sum(hist.values())}, bytes {byte_total}, longest {longest}")
    seq = {}
    for i in range(total):
        seq[level_of(i)] = seq.get(level_of(i), 0) + 1
    print(f"matches sequential pass: {str(seq == hist).lower()}")


main()
