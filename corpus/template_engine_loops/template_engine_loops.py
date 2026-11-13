def tag(line):
    t = line.strip()
    if t.startswith("[%") and t.endswith("%]"):
        return t[2:len(t) - 2].strip()
    return ""


def block_end(lines, start, opener, closer):
    depth = 0
    for i in range(start, len(lines)):
        t = tag(lines[i])
        if t.startswith(opener):
            depth += 1
        elif t == closer:
            depth -= 1
            if depth == 0:
                return i
    return len(lines)


def else_at(lines, start, stop):
    depth = 0
    for i in range(start + 1, stop):
        t = tag(lines[i])
        if t.startswith("if "):
            depth += 1
        elif t == "endif":
            depth -= 1
        elif t == "else" and depth == 0:
            return i
    return -1


def substitute(line, vars_):
    out = ""
    rest = line
    while True:
        i = rest.find("[[")
        if i < 0:
            break
        tail = rest[i + 2:]
        j = tail.find("]]")
        if j < 0:
            break
        key = tail[:j].strip()
        val = vars_.get(key, "")
        out = f"{out}{rest[:i]}{val}"
        rest = tail[j + 2:]
    return f"{out}{rest}"


def truthy(cond, vars_, lists):
    words = cond.split()
    name = words[-1]
    if name in lists:
        yes = len(lists[name]) > 0
    else:
        v = vars_.get(name, "")
        yes = v != "" and v != "false"
    if len(words) == 2 and words[0] == "not":
        return not yes
    return yes


def render(lines, vars_, lists):
    out = []
    i = 0
    while i < len(lines):
        t = tag(lines[i])
        if t.startswith("for "):
            w = t.split()
            fin = block_end(lines, i, "for ", "endfor")
            body = lines[i + 1:fin]
            items = lists.get(w[3], [])
            n = 0
            for item in items:
                n += 1
                inner = dict(vars_)
                inner[w[1]] = item
                inner["loop.index"] = f"{n}"
                out.extend(render(body, inner, lists))
            i = fin + 1
        elif t.startswith("if "):
            done = block_end(lines, i, "if ", "endif")
            mid = else_at(lines, i, done)
            stop = done if mid < 0 else mid
            if truthy(t[3:], vars_, lists):
                out.extend(render(lines[i + 1:stop], vars_, lists))
            elif mid >= 0:
                out.extend(render(lines[mid + 1:done], vars_, lists))
            i = done + 1
        else:
            out.append(substitute(lines[i], vars_))
            i += 1
    return out


def main():
    tpl_lines = [
        "Report for [[user]]",
        "[% if admin %]",
        "  (admin view)",
        "[% else %]",
        "  (read-only view)",
        "[% endif %]",
        "[% for team in teams %]",
        "Team [[loop.index]]: [[team]]",
        "[% for who in members %]",
        "  - [[who]] ([[team]])",
        "[% endfor %]",
        "[% endfor %]",
        "[% if not extras %]",
        "No extras.",
        "[% endif %]",
        "Missing: <[[nothing]]>",
    ]
    lists = {"teams": ["core", "tools"], "members": ["ann", "bo"], "extras": []}
    for admin in ["true", ""]:
        print(f"== admin={admin}")
        for line in render(tpl_lines, {"user": "kim", "admin": admin}, lists):
            print(line)


main()
