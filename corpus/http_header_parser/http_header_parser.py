import re


def split_once(s, sep):
    i = s.find(sep)
    if i < 0:
        return (s, "")
    return (s[:i], s[i + len(sep):])


def to_int(s):
    return int(s) if re.fullmatch(r"-?[0-9]+", s) else None


def parse_q(text):
    whole, frac = split_once(text, ".")
    w = to_int(whole)
    if w is None:
        return 1000
    f = to_int(frac.ljust(3, "0")[:3])
    if f is None:
        f = 0
    return w * 1000 + f


def parse_accept(value):
    prefs = []
    for part in value.split(","):
        media, params = split_once(part.strip(), ";")
        q = 1000
        for p in params.split(";"):
            k, v = split_once(p.strip(), "=")
            if k == "q":
                q = parse_q(v)
        prefs.append((media.strip(), q))
    return sorted(prefs, key=lambda x: -x[1])


def parse_cookies(value):
    jar = {}
    for part in value.split(";"):
        k, v = split_once(part.strip(), "=")
        if k != "":
            jar[k] = v
    return jar


lines = [
    "POST /submit?x=1 HTTP/1.1",
    "Host: example.org",
    "ACCEPT: text/html, application/json;q=0.9, */*;q=0.1, text/plain;q=0.9",
    "X-Trace: a",
    "x-trace: b",
    "Cookie: sid=abc123; theme=dark; lang=en",
    "X-Long: first part",
    "   second part",
    "Content-Length: 11",
    "bad header line",
    "",
    "hello world",
]
names = []
values = []
bad = 0
body = ""
in_body = False
for i in range(1, len(lines)):
    line = lines[i]
    if in_body:
        body += line
    elif line == "":
        in_body = True
    elif line.startswith(" ") or line.startswith("\t"):
        values[-1] = values[-1] + " " + line.strip()
    else:
        name, value = split_once(line, ":")
        if ":" not in line:
            bad += 1
            continue
        names.append(name.strip().lower())
        values.append(value.strip())
merged = {}
for i in range(len(names)):
    if names[i] in merged:
        merged[names[i]] = merged[names[i]] + ", " + values[i]
    else:
        merged[names[i]] = values[i]
method, rest = split_once(lines[0], " ")
target, version = split_once(rest, " ")
print(f"{method} {target} {version}")
for k in sorted(merged):
    print(f"{k}: {merged[k]}")
print(f"bad lines: {bad}")
print(f"body: {len(body)} chars, declared {merged.get('content-length', '?')}")
jar = parse_cookies(merged.get("cookie", ""))
for k in sorted(jar):
    print(f"cookie {k} = {jar[k]}")
for media, q in parse_accept(merged.get("accept", "")):
    print(f"accept {media} q={q}")
