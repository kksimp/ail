TAGS = ["FIXME", "TODO", "HACK", "NOTE"]


def comment_text(line):
    a = line.find("//")
    b = line.find("#")
    if a >= 0 and (b < 0 or a < b):
        return line[a + 2:]
    if b >= 0:
        return line[b + 1:]
    return None


def parse_note(comment):
    body = comment.strip()
    for tag in TAGS:
        if not body.startswith(tag):
            continue
        rest = body[len(tag):]
        owner = ""
        msg = rest
        if rest.startswith("("):
            rp = rest.find(")")
            if rp < 0:
                return None
            owner = rest[1:rp]
            msg = rest[rp + 1:]
        elif rest != "" and not rest.startswith(":") and not rest.startswith(" "):
            continue
        if msg.startswith(":"):
            msg = msg[1:]
        return (tag, owner, msg.strip())
    return None


def scan(name, fileno, text):
    out = []
    n = 0
    for raw in text.split("\n"):
        n += 1
        c = comment_text(raw)
        if c is None:
            continue
        found = parse_note(c)
        if found is None:
            continue
        rank = TAGS.index(found[0])
        out.append((name, n, found[0], found[1], found[2], rank * 100000 + fileno * 1000 + n))
    return out


def main():
    go_src = [
        "package main",
        "// TODO(kim): handle EOF",
        "func f() {",
        "    s := \"TODO later\"   // FIXME: leaks the buffer",
        "    // TODOS are plain words",
        "}",
    ]
    py_src = [
        "import sys",
        "# HACK(lee): sleep until the port opens",
        "x = 1  # NOTE keep this in sync",
        "# TODO",
        "# FIXME(kim) off by one",
    ]
    notes = []
    notes.extend(scan("main.go", 0, "\n".join(go_src)))
    notes.extend(scan("tool.py", 1, "\n".join(py_src)))
    for nt in sorted(notes, key=lambda q: q[5]):
        who = "-" if nt[3] == "" else nt[3]
        print(f"{nt[2]:<5} {nt[0]}:{nt[1]} [{who}] {nt[4]}")
    for tag in TAGS:
        count = 0
        for nt in notes:
            if nt[2] == tag:
                count += 1
        print(f"{tag}: {count}")


main()
