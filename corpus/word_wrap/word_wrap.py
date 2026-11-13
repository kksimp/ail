def spaces(n):
    return " " * n


def wrap(text, width):
    lines = []
    cur = ""
    for w in text.split():
        if cur == "":
            cur = w
        elif len(cur) + 1 + len(w) <= width:
            cur = cur + " " + w
        else:
            lines.append(cur)
            cur = w
    if cur != "":
        lines.append(cur)
    return lines


def justify_line(line, width):
    ws = line.split()
    if len(ws) < 2:
        return line
    letters = sum(len(w) for w in ws)
    gaps = len(ws) - 1
    total = width - letters
    base = total // gaps
    extra = total % gaps
    out = ws[0]
    for i in range(1, len(ws)):
        n = base
        if i <= extra:
            n += 1
        out = out + spaces(n) + ws[i]
    return out


def justify(lines, width):
    out = []
    for i in range(len(lines)):
        if i + 1 < len(lines):
            out.append(justify_line(lines[i], width))
        else:
            out.append(lines[i])
    return out


def boxed(lines, width):
    for line in lines:
        print("|" + line.ljust(width) + "|")


def main():
    text = "AIL is a garbage collected, memory safe systems language designed so that humans and AI agents can both read, write and check programs with the same confidence."
    print("width 30, left aligned:")
    boxed(wrap(text, 30), 30)
    print("width 30, justified:")
    boxed(justify(wrap(text, 30), 30), 30)
    print("width 12:")
    boxed(wrap(text, 12), 12)
    print("a long word does not break:")
    boxed(wrap("tiny supercalifragilistic word", 10), 20)


main()
