BAR_MAX = 20


def parse_diff(lines):
    out = []
    path = ""
    adds = dels = hunks = 0
    for line in lines:
        if line.startswith("+++ "):
            if path != "":
                out.append((path, adds, dels, hunks))
            rest = line[4:]
            path = rest[2:] if rest.startswith("b/") else rest
            adds = dels = hunks = 0
        elif line.startswith("@@"):
            hunks += 1
        elif line.startswith("---") or line.startswith("diff "):
            continue
        elif line.startswith("+"):
            adds += 1
        elif line.startswith("-"):
            dels += 1
    if path != "":
        out.append((path, adds, dels, hunks))
    return out


def scaled(n, biggest):
    if n == 0:
        return 0
    if biggest <= BAR_MAX:
        return n
    return max(n * BAR_MAX // biggest, 1)


def plural(n, word):
    if n == 1:
        return f"{n} {word}"
    return f"{n} {word}s"


def report(stats):
    out = []
    width = 0
    biggest = 0
    total_add = 0
    total_del = 0
    for path, adds, dels, hunks in stats:
        width = max(width, len(path))
        biggest = max(biggest, adds + dels)
        total_add += adds
        total_del += dels
    for path, adds, dels, hunks in stats:
        total = adds + dels
        marks = "+" * scaled(adds, biggest) + "-" * scaled(dels, biggest)
        out.append(f" {path.ljust(width)} | {total:>3} {marks}")
    out.append(f" {plural(len(stats), 'file')} changed, {plural(total_add, 'insertion')}(+), {plural(total_del, 'deletion')}(-)")
    return out


def sample():
    lines = [
        "diff --git a/src/main.ail b/src/main.ail",
        "--- a/src/main.ail",
        "+++ b/src/main.ail",
        "@@ -1,4 +1,5 @@",
        " fn main()",
        "-    print(1)",
        "+    print(2)",
        "+    print(3)",
        "@@ -20,3 +21,3 @@",
        "-    old()",
        "+    new()",
        "diff --git a/README.md b/README.md",
        "--- a/README.md",
        "+++ b/README.md",
        "@@ -1 +1 @@",
        "-hello",
        "+hello, world",
        "diff --git a/lib/util.ail b/lib/util.ail",
        "--- /dev/null",
        "+++ b/lib/util.ail",
        "@@ -0,0 +1,30 @@",
    ]
    for i in range(1, 31):
        lines.append(f"+line {i}")
    return lines


def main():
    stats = parse_diff(sample())
    for path, adds, dels, hunks in stats:
        print(f"{path}: {plural(hunks, 'hunk')}")
    for row in report(stats):
        print(row)


main()
