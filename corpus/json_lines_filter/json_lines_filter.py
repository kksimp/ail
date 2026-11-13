import os, tempfile


def parse_filter(spec):
    idx = spec.find(">")
    if idx >= 0:
        return (spec[:idx], ">", spec[idx + 1:])
    eq = spec.find("=")
    if eq < 0:
        return None
    return (spec[:eq], "=", spec[eq + 1:])


def unquote(s):
    t = s.strip()
    if len(t) >= 2 and t.startswith('"') and t.endswith('"'):
        return t[1:-1]
    return t


def parse_record(line):
    t = line.strip()
    if len(t) < 2 or not t.startswith("{") or not t.endswith("}"):
        return None
    rec = {}
    inner = t[1:-1]
    if inner.strip() == "":
        return rec
    for pair in inner.split(","):
        at = pair.find(":")
        if at < 0:
            return None
        rec[unquote(pair[:at])] = unquote(pair[at + 1:])
    return rec


def matches(rec, f):
    key, op, val = f
    if key not in rec:
        return False
    v = rec[key]
    if op == "=":
        return v == val
    try:
        n = int(v)
        limit = int(val)
    except ValueError:
        return False
    return n > limit


def keep(rec, filters):
    return all(matches(rec, f) for f in filters)


def main():
    d = tempfile.mkdtemp(prefix="corpus_json_lines_filter")
    src = os.path.join(d, "events.jsonl")
    dst = os.path.join(d, "hits.jsonl")
    lines = [
        '{"level":"info","user":"ann","ms":120}',
        '{"level":"warn","user":"bob","ms":340}',
        "not json at all",
        '{"level":"error","user":"ann","ms":15}',
        '{"level":"warn","user":"cy","ms":95}',
        '{"level":"warn","user":"ann","ms":410}',
        '{"level":"warn","user":"dee"}',
    ]
    with open(src, "w", newline="") as w:
        for l in lines:
            w.write(l + "\n")
    filters = []
    for spec in ["level=warn", "ms>100"]:
        f = parse_filter(spec)
        if f is not None:
            filters.append(f)
    print(f"filters: {len(filters)}")
    total = bad = kept = 0
    with open(src, newline="") as r, open(dst, "w", newline="") as w:
        for line in r:
            line = line.rstrip("\n")
            total += 1
            rec = parse_record(line)
            if rec is not None:
                if keep(rec, filters):
                    kept += 1
                    w.write(line + "\n")
            else:
                bad += 1
    print(f"read {total} lines, {bad} malformed, {kept} kept")
    with open(dst, newline="") as f:
        for row in f.read().splitlines():
            hit = parse_record(row)
            if hit is None:
                continue
            print(f"  {hit['user']} took {hit['ms']} ms")
    os.remove(src)
    os.remove(dst)
    os.rmdir(d)
    print(f"cleaned up: {str(not os.path.exists(d)).lower()}")


main()
