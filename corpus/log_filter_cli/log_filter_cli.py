import os, tempfile, shutil

LOG = [
    "09:58:01 INFO boot: starting api",
    "09:58:02 DEBUG boot: loading config",
    "10:00:15 INFO http: GET /health 200",
    "10:01:40 WARN db: slow query 1200ms",
    "10:02:03 ERROR http: GET /orders 500",
    "10:02:04 INFO http: GET /orders 200",
    "10:03:30 ERROR db: connection lost",
    "10:03:31 WARN db: retrying connection",
    "10:05:00 INFO boot: ready",
    "this line is garbled",
]

RANKS = {"DEBUG": 0, "INFO": 1, "WARN": 2, "ERROR": 3}


def parse_line(line):
    parts = line.split(" ", 2)
    if len(parts) != 3 or parts[1] not in RANKS:
        return None
    return (parts[0], parts[1], parts[2])


def accepts(q, e):
    min_rank, needle, after, before = q
    return RANKS[e[1]] >= min_rank and needle in e[2] and e[0] >= after and e[0] < before


def filter_file(p, q):
    kept = []
    with open(p, newline="") as f:
        for line in f:
            e = parse_line(line.rstrip())
            if e is None:
                continue
            if accepts(q, e):
                kept.append(e)
    return kept


def show(title, got):
    print(f"{title}: {len(got)} match")
    for t, lv, m in got:
        print(f"  {t} {lv:<5} {m}")


def main():
    d = tempfile.mkdtemp(prefix="corpus_log_filter_cli")
    p = os.path.join(d, "service.log")
    with open(p, "w", newline="") as f:
        for line in LOG:
            f.write(line + "\n")
    everything = (0, "", "00:00:00", "99:99:99")
    show("all valid", filter_file(p, everything))
    show("warn and up", filter_file(p, (2, "", "00:00:00", "99:99:99")))
    show("db entries", filter_file(p, (0, "db:", "00:00:00", "99:99:99")))
    show("10:02 to 10:04", filter_file(p, (0, "", "10:02:00", "10:04:00")))
    counts = {}
    for e in filter_file(p, everything):
        counts[e[1]] = counts.get(e[1], 0) + 1
    for level in sorted(counts):
        print(f"{level:<5} {counts[level]}")
    shutil.rmtree(d)
    print(f"cleaned up: {str(not os.path.exists(d)).lower()}")


main()
