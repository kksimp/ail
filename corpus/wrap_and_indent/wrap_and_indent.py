def wrap_indent(text, width, lead, hang):
    lines = []
    cur = lead
    fresh = True
    for w in text.split():
        if fresh:
            cur = f"{cur}{w}"
            fresh = False
        elif len(cur) + 1 + len(w) <= width:
            cur = f"{cur} {w}"
        else:
            lines.append(cur)
            cur = f"{hang}{w}"
    if not fresh:
        lines.append(cur)
    return lines


def indent_of(line):
    return len(line) - len(line.lstrip())


def dedent(lines):
    least = -1
    for l in lines:
        if l.strip() == "":
            continue
        n = indent_of(l)
        if least < 0 or n < least:
            least = n
    out = []
    for l in lines:
        if l.strip() == "":
            out.append("")
        else:
            out.append(l[least:])
    return out


def indent_block(lines, prefix):
    out = []
    for l in lines:
        if l == "":
            out.append("")
        else:
            out.append(f"{prefix}{l}")
    return out


def show(title, lines):
    print(f"{title}:")
    for l in lines:
        print(f"[{l}]")


def main():
    text = "Wrapping text is mostly bookkeeping: count the characters on the line, break before a word that does not fit, and carry the hanging indent onto every continuation line."
    show("plain 30", wrap_indent(text, 30, "", ""))
    show("bullet 36", wrap_indent(text, 36, "- ", "  "))
    show("quote 28", wrap_indent("short words only here and a verylongunbreakableword too", 28, "> ", "> "))
    code = [
        "        if ready:",
        "            go()",
        "",
        "        stop()",
    ]
    show("dedent", dedent(code))
    show("reindent", indent_block(dedent(code), "    "))


main()
