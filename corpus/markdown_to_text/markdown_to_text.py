def rule(ch, n):
    return ch * n


def index_from(cs, target, start):
    i = start
    while i < len(cs):
        if cs[i] == target:
            return i
        i += 1
    return -1


def strip_inline(s):
    cs = s
    out = ""
    i = 0
    while i < len(cs):
        c = cs[i]
        if c == "*" or c == "`":
            i += 1
            continue
        if c == "[":
            shut = index_from(cs, "]", i + 1)
            if shut > 0 and shut + 1 < len(cs) and cs[shut + 1] == "(":
                stop = index_from(cs, ")", shut + 2)
                if stop > 0:
                    label = cs[i + 1:shut]
                    url = cs[shut + 2:stop]
                    out = f"{out}{label} ({url})"
                    i = stop + 1
                    continue
        out = f"{out}{c}"
        i += 1
    return out


def heading_level(line):
    n = 0
    for c in line:
        if c != "#":
            break
        n += 1
    return n


def render(text):
    out = []
    in_code = False
    for raw in text.split("\n"):
        line = raw.rstrip()
        if line.startswith("```"):
            in_code = not in_code
            continue
        if in_code:
            out.append(f"    {raw}")
            continue
        level = heading_level(line)
        rest = line[level:]
        if level > 0 and level <= 6 and rest.startswith(" "):
            title = strip_inline(rest.strip())
            if level == 1:
                out.append(title.upper())
                out.append(rule("=", len(title)))
            elif level == 2:
                out.append(title)
                out.append(rule("-", len(title)))
            else:
                out.append(f"{title}:")
        elif line == "---":
            out.append(rule("-", 20))
        elif line.startswith("> "):
            out.append(f"| {strip_inline(line[2:])}")
        elif line.startswith("- ") or line.startswith("* "):
            out.append(f"  * {strip_inline(line[2:])}")
        else:
            out.append(strip_inline(line))
    return out


def main():
    doc = [
        "# Release notes",
        "Version **2.1** is out; see the [changelog](https://example.org/log) or `ail --help`.",
        "",
        "## Highlights",
        "- *Faster* builds",
        "* Better [docs](docs/index.md)",
        "> Note: upgrade **before** friday",
        "---",
        "### Install",
        "```",
        "ail install --all",
        "```",
        "Done [unclosed link](oops",
    ]
    for line in render("\n".join(doc)):
        print(line)


main()
