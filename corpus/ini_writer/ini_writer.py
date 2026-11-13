def needs_quote(v):
    return v != v.strip() or ";" in v or "#" in v or '"' in v


def quote(v):
    body = ""
    for c in v:
        if c == '"' or c == "\\":
            body += "\\" + c
        else:
            body += c
    return '"' + body + '"'


def unquote(raw):
    if len(raw) >= 2 and raw.startswith('"') and raw.endswith('"'):
        out = ""
        esc = False
        for c in raw[1:-1]:
            if esc:
                out += c
                esc = False
            elif c == "\\":
                esc = True
            else:
                out += c
        return out
    return raw


def write_ini(items):
    secs = []
    for entry in items:
        if entry[0] not in secs:
            secs.append(entry[0])
    lines = []
    for sec in sorted(secs):
        if sec != "":
            if lines:
                lines.append("")
            lines.append("[%s]" % sec)
        for item in items:
            if item[0] == sec:
                v = quote(item[2]) if needs_quote(item[2]) else item[2]
                lines.append(item[1] + " =" if v == "" else item[1] + " = " + v)
    return "\n".join(lines)


def read_ini(text):
    out = []
    current = ""
    for raw in text.split("\n"):
        line = raw.strip()
        if line == "" or line.startswith(";") or line.startswith("#"):
            continue
        if line.startswith("[") and line.endswith("]"):
            current = line[1:-1]
            continue
        idx = line.find("=")
        if idx < 0:
            continue
        out.append((current, line[:idx].strip(), unquote(line[idx + 1:].strip())))
    return out


items = [
    ("", "name", "demo app"),
    ("server", "host", "localhost"),
    ("server", "port", "8080"),
    ("", "debug", "true"),
    ("paths", "root", " /srv/app "),
    ("paths", "note", "a;b # c"),
    ("server", "motto", 'say "hi" now'),
    ("paths", "empty", ""),
    ("db", "dsn", "user=ail;pass=x"),
    ("db", "dir", r"C:\data"),
]
text = write_ini(items)
print(text)
print("---")
back = read_ini(text)
expected = sorted(items, key=lambda e: e[0])
print(f"items: {len(items)}, read back: {len(back)}")
print("round trip: " + ("ok" if back == expected else "MISMATCH"))
