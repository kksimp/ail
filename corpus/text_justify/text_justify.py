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


def full_line(line, width):
    ws = line.split()
    if len(ws) < 2:
        return line
    letters = sum(len(w) for w in ws)
    gaps = len(ws) - 1
    spare = width - letters
    out = ws[0]
    for i in range(1, len(ws)):
        n = spare // gaps
        if i <= spare % gaps:
            n += 1
        out = out + spaces(n) + ws[i]
    return out


def center(s, width):
    # extra space goes to the right side
    total = width - len(s)
    left = total // 2
    return " " * left + s + " " * (total - left)


def layout(text, width, mode):
    lines = wrap(text, width)
    out = []
    for i in range(len(lines)):
        line = lines[i]
        if mode == "left":
            padded = line.ljust(width)
        elif mode == "right":
            padded = line.rjust(width)
        elif mode == "center":
            padded = center(line, width)
        else:
            padded = full_line(line, width) if i + 1 < len(lines) else line.ljust(width)
        out.append(padded)
    return out


def main():
    text = "Plain text tools should be boring and predictable. A justified paragraph spreads the spare space across the gaps, and the last line stays left aligned."
    for mode in ["left", "right", "center", "full"]:
        print(mode + ":")
        print("+" + "-".ljust(26, "-") + "+")
        for line in layout(text, 26, mode):
            print("|" + line + "|")


main()
