import os, tempfile

SAMPLE = [
    "The quick brown fox",
    "jumps over the lazy dog.",
    "THE END of the first act",
    "",
    "A dog barks; the fox runs away",
    "Nothing to see here",
    "the last line",
]


def hit(line, pat, ignore_case, invert):
    found = pat.lower() in line.lower() if ignore_case else pat in line
    return (not found) if invert else found


def grep(lines, pat, ignore_case, invert, numbers):
    out = []
    for i, line in enumerate(lines):
        if hit(line, pat, ignore_case, invert):
            out.append(f"{i + 1}:{line}" if numbers else line)
    return out


def show(title, lines, pat, ic, inv, nums, count_only):
    found = grep(lines, pat, ic, inv, nums)
    print(f"$ {title}")
    if count_only:
        print(len(found))
        return
    for l in found:
        print(l)
    print(f"-- {len(found)} of {len(lines)} lines")


def main():
    d = tempfile.mkdtemp(prefix="corpus_grep_lite")
    p = os.path.join(d, "story.txt")
    with open(p, "w", newline="") as f:
        f.write("\n".join(SAMPLE) + "\n")
    with open(p, newline="") as f:
        lines = f.read().splitlines()
    show("grep the story.txt", lines, "the", False, False, False, False)
    show("grep -i the story.txt", lines, "the", True, False, False, False)
    show("grep -n fox story.txt", lines, "fox", False, False, True, False)
    show("grep -v -n o story.txt", lines, "o", False, True, True, False)
    show("grep -ic dog story.txt", lines, "dog", True, False, False, True)
    os.remove(p)
    os.rmdir(d)


main()
