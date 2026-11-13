import re

LINE = re.compile(r"^(\d{4}-\d{2}-\d{2}) (\d{2}):(\d{2}):(\d{2}) \[(\w+)\] ([a-z]+): (.*)$", re.ASCII)
KV = re.compile(r"(\w+)=([\w.]+)", re.ASCII)


def to_int(s):
    return int(s) if re.fullmatch(r"-?[0-9]+", s) else None


def parse_line(line):
    c = LINE.search(line)
    if c is None:
        return None
    return (c.group(1), int(c.group(2)), c.group(5), c.group(6), c.group(7))


def fields(msg):
    out = {}
    for m in KV.finditer(msg):
        t = m.group(0)
        i = t.find("=")
        out[t[:i]] = t[i + 1:]
    return out


def sample():
    return [
        "2024-03-05 12:01:33 [INFO] web: GET /home status=200 took=35ms",
        "2024-03-05 12:01:40 [WARN] auth: login failed user=bob ip=10.0.0.7",
        "this line is garbage",
        "2024-03-05 12:02:01 [ERROR] db: query timeout table=orders took=5000ms",
        "2024-03-05 13:15:09 [INFO] web: POST /cart status=201 took=120ms",
        "2024-03-05 13:15:10 [INFO] auth: login ok user=ann ip=10.0.0.9",
        "2024-03-05 25:99 [INFO] web: truncated",
        "2024-03-06 00:00:01 [INFO] web: GET /about status=404 took=7ms",
    ]


levels = {}
comps = {}
hours = {}
total = 0
timed = 0
skipped = 0
for line in sample():
    e = parse_line(line)
    if e is None:
        skipped += 1
        print(f"skip: {line}")
        continue
    date, hour, level, comp, msg = e
    levels[level] = levels.get(level, 0) + 1
    comps[comp] = comps.get(comp, 0) + 1
    hours[hour] = hours.get(hour, 0) + 1
    kv = fields(msg)
    took = kv.get("took", "")
    ms = to_int(took.replace("ms", ""))
    if ms is None:
        ms = -1
    if ms >= 0:
        total += ms
        timed += 1
    if "user" in kv:
        print(f"{date} user {kv['user']} from {kv.get('ip', '?')}")
for k in sorted(levels):
    print(f"level {k}: {levels[k]}")
for k in sorted(comps):
    print(f"component {k}: {comps[k]}")
for h in sorted(hours):
    print(f"hour {h:02}: {hours[h]}")
print(f"skipped {skipped}, timed {timed}, total {total}ms, mean {total // timed}ms")
