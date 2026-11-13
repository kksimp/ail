def sample():
    return [
        "# Guide",
        "Intro text.",
        "## Install",
        "```",
        "# not a heading",
        "```",
        "## Usage",
        "### Flags",
        "####### too deep",
        "#nospace",
        "## Usage",
        "# Done",
    ]


def parse_heading(line):
    n = 0
    for c in line:
        if c == "#":
            n += 1
        else:
            break
    rest = line[n:]
    if n < 1 or n > 6 or not rest.startswith(" "):
        return None
    return (n, rest.strip())


def headings(lines):
    out = []
    fenced = False
    for line in lines:
        if line.startswith("```"):
            fenced = not fenced
            continue
        if fenced:
            continue
        h = parse_heading(line)
        if h is None:
            continue
        out.append(h)
    return out


def slug(title):
    out = ""
    for c in title.lower():
        if ("a" <= c <= "z") or ("0" <= c <= "9"):
            out += c
        elif c == " " or c == "-":
            out += "-"
    return out


def main():
    hs = headings(sample())
    print(f"{len(hs)} headings")
    seen = {}
    for level, title in hs:
        base = slug(title)
        k = seen.get(base, 0)
        seen[base] = k + 1
        anchor = base if k == 0 else f"{base}-{k}"
        pad = " " * ((level - 1) * 2)
        print(f"{pad}- {title} (#{anchor})")


main()
