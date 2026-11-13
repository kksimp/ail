import json

SAMPLE = "\n".join([
    "; deployment settings",
    "name = shop",
    "debug = false",
    "",
    "[server]",
    "host = example.org",
    "port = 8080",
    'motto = say "hi"',
    "",
    "[limits]",
    "# per minute",
    "requests = 600",
    "burst=40",
    "ignored line",
])


def parse(text):
    out = []
    current = ""
    for raw in text.splitlines():
        line = raw.strip()
        if line == "" or line.startswith("#") or line.startswith(";"):
            continue
        if line.startswith("[") and line.endswith("]"):
            current = line[1:-1].strip()
            continue
        idx = line.find("=")
        if idx < 0:
            continue
        out.append((current, line[:idx].strip(), line[idx + 1:].strip()))
    return out


def quote(s):
    esc = s.replace("\\", "\\\\").replace('"', '\\"')
    return '"' + esc + '"'


def scalar(v):
    if v == "true" or v == "false":
        return v
    try:
        return str(int(v))
    except ValueError:
        return quote(v)


def sections(entries):
    names = []
    for sec, _, _ in entries:
        if sec != "" and sec not in names:
            names.append(sec)
    return names


def member_items(entries, sec, pad):
    return [[f"{pad}{quote(n)}: {scalar(v)}"] for s, n, v in entries if s == sec]


def comma_last(block, more):
    if not more:
        return block
    return block[:-1] + [block[-1] + ","]


def flatten(items):
    out = []
    for i, item in enumerate(items):
        out.extend(comma_last(item, i < len(items) - 1))
    return out


def render(entries):
    top = member_items(entries, "", "  ")
    for s in sections(entries):
        inner = ["  " + quote(s) + ": {"]
        inner.extend(flatten(member_items(entries, s, "    ")))
        inner.append("  }")
        top.append(inner)
    out = ["{"]
    out.extend(flatten(top))
    out.append("}")
    return out


def main():
    lines = render(parse(SAMPLE))
    for l in lines:
        print(l)
    doc = json.loads("\n".join(lines))
    print(f"valid json, {len(lines)} lines")
    print(f"name: {doc['name']}, debug: {str(doc['debug']).lower()}")
    srv = doc["server"]
    print(f"port: {srv['port']}")
    print(f"motto: {srv['motto']}")
    print(f"burst: {doc['limits']['burst']}")


main()
