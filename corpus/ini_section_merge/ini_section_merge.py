def parse(text):
    out = []
    current = ""
    for raw in text.split("\n"):
        line = raw.strip()
        if line == "" or line.startswith("#") or line.startswith(";"):
            continue
        if line.startswith("[") and line.endswith("]"):
            current = line[1:len(line) - 1].strip()
            continue
        pos = line.find("=")
        if pos < 0:
            continue
        out.append([current, line[:pos].strip(), line[pos + 1:].strip()])
    return out


def index_of_key(entries, sec, name):
    for i in range(len(entries)):
        if entries[i][0] == sec and entries[i][1] == name:
            return i
    return -1


def label(e):
    if e[0] == "":
        return e[1]
    return f"{e[0]}.{e[1]}"


def merge(base, over):
    merged = [list(e) for e in base]
    changes = []
    for o in over:
        i = index_of_key(merged, o[0], o[1])
        if o[2] == "~":
            if i >= 0:
                changes.append(f"- {label(o)}")
                del merged[i]
        elif i < 0:
            changes.append(f"+ {label(o)} = {o[2]}")
            merged.append(list(o))
        elif merged[i][2] != o[2]:
            changes.append(f"~ {label(o)}: {merged[i][2]} -> {o[2]}")
            merged[i][2] = o[2]
    return merged, changes


def render(entries):
    out = []
    for x in entries:
        if x[0] == "":
            out.append(f"{x[1]} = {x[2]}")
    seen = set()
    for e in entries:
        if e[0] != "" and e[0] not in seen:
            seen.add(e[0])
            if out:
                out.append("")
            out.append(f"[{e[0]}]")
            for y in entries:
                if y[0] == e[0]:
                    out.append(f"{y[1]} = {y[2]}")
    return out


def main():
    base = parse("name = demo\nmode = dev\n[db]\nhost = localhost\nport = 5432\npool = 4\n[cache]\nttl = 60\n")
    over = parse("mode = prod\n[db]\nhost = db.internal\nport = 5432\npool = ~\n[log]\nlevel = warn\n[cache]\nttl = ~\nsize = 128\n")
    merged, changes = merge(base, over)
    for line in changes:
        print(line)
    print("---")
    for line in render(merged):
        print(line)


main()
