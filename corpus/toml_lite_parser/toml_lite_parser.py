import re


def to_int(s):
    return int(s) if re.fullmatch(r"-?[0-9]+", s) else None


def classify(raw):
    if len(raw) >= 2 and raw.startswith('"') and raw.endswith('"'):
        return ("string", raw[1:-1])
    if raw == "true" or raw == "false":
        return ("bool", raw)
    if raw.startswith("[") and raw.endswith("]"):
        inner = raw[1:-1].strip()
        if inner == "":
            return ("array", "0")
        return ("array", str(len(inner.split(","))))
    n = to_int(raw)
    if n is None:
        return ("invalid", raw)
    return ("int", str(n))


def strip_comment(v):
    if v.startswith('"'):
        qi = v[1:].find('"')
        if qi < 0:
            return v
        return v[: qi + 2]
    i = v.find("#")
    if i < 0:
        return v
    return v[:i].strip()


def parse(text):
    out = []
    table = ""
    for raw in text.split("\n"):
        line = raw.strip()
        if line == "" or line.startswith("#"):
            continue
        if line.startswith("[") and line.endswith("]"):
            table = line[1:-1].strip()
            continue
        idx = line.find("=")
        if idx < 0:
            continue
        name = line[:idx].strip()
        value = strip_comment(line[idx + 1:].strip())
        k, shown = classify(value)
        full = name if table == "" else f"{table}.{name}"
        out.append((full, k, shown))
    return out


def sample():
    lines = [
        'title = "AIL demo"',
        "debug = true",
        "",
        "[server]",
        'host = "localhost"   # primary',
        "port = 8080",
        'tags = ["a", "b", "c"]',
        "",
        "[limits.rate]",
        "max = 100  # per minute",
        "ratio = 1.5",
        "empty = []",
        'note = "has # inside"',
        "no equals here",
    ]
    return "\n".join(lines)


items = parse(sample())
for key, kind, text in items:
    print(f"{key:<14} {kind:<8} {text}")
counts = {}
for _, kind, _ in items:
    counts[kind] = counts.get(kind, 0) + 1
for k in sorted(counts):
    print(f"{k}: {counts[k]}")
