def closer_of(c):
    return {"(": ")", "[": "]"}.get(c, "}")


def check(s):
    closers = []
    opens = []
    for i, c in enumerate(s):
        if c in "([{":
            closers.append(closer_of(c))
            opens.append(i)
        elif c in ")]}":
            if not closers:
                return f"unexpected '{c}' at {i}"
            want = closers.pop()
            opener = opens.pop()
            if c != want:
                return f"mismatch at {i}: found '{c}', expected '{want}' for opener at {opener}"
    if not closers:
        return "ok"
    last = len(closers) - 1
    return f"unclosed, expected '{closers[last]}' for opener at {opens[last]}"


def main():
    inputs = [
        "(a + b) * [c - d]",
        "f(x[1], {k: v})",
        "((]",
        "no brackets at all",
        ")(",
        "[({})",
        "",
    ]
    for s in inputs:
        print(f"{s:<20} -> {check(s)}")


main()
