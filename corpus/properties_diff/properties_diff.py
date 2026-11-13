def sep_index(line):
    for i, c in enumerate(line):
        if c == "=" or c == ":":
            return i
    return None


def logical_lines(text):
    out = []
    pending = ""
    for raw in text.split("\n"):
        line = raw.strip() if pending == "" else raw.lstrip()
        if line.endswith("\\"):
            pending = pending + line[:-1]
            continue
        out.append(pending + line)
        pending = ""
    if pending != "":
        out.append(pending)
    return out


def parse(text):
    props = {}
    for line in logical_lines(text):
        if line == "" or line.startswith("#") or line.startswith("!"):
            continue
        i = sep_index(line)
        if i is None:
            continue
        props[line[:i].strip()] = line[i + 1:].strip()
    return props


def diff(old, new):
    names = set(old) | set(new)
    out = []
    for k in sorted(names):
        if k not in new:
            out.append(f"- {k} = {old[k]}")
        elif k not in old:
            out.append(f"+ {k} = {new[k]}")
        elif old[k] != new[k]:
            out.append(f"~ {k}: {old[k]} -> {new[k]}")
    return out


def old_text():
    lines = [
        "# service config", "name = billing", "port: 8080", "! legacy",
        "hosts = a.example.org, \\", "        b.example.org", "debug=true", "timeout = 30",
    ]
    return "\n".join(lines)


def new_text():
    lines = [
        "# service config", "name = billing", "port: 9090", "hosts = a.example.org, \\",
        "        c.example.org", "timeout = 30", "retries = 5", "region:eu",
    ]
    return "\n".join(lines)


old = parse(old_text())
new = parse(new_text())
changes = diff(old, new)
for c in changes:
    print(c)
same = 0
for k in old:
    if new.get(k) == old.get(k):
        same += 1
print(f"old {len(old)}, new {len(new)}, changed lines {len(changes)}, unchanged {same}")
