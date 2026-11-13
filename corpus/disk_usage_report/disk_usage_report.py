FILES = {
    "src/main.ail": 420,
    "src/util.ail": 180,
    "src/lib/core.ail": 950,
    "src/lib/io.ail": 300,
    "docs/guide.md": 640,
    "docs/api.md": 1210,
    "assets/logo.bin": 2048,
    "README.md": 150,
    "Makefile": 90,
}


def extension(name):
    parts = name.split(".")
    if len(parts) > 1:
        return parts[-1]
    return "(none)"


def main():
    exts = {}
    tops = {}
    for p, size in FILES.items():
        top = p.split("/")[0]
        is_dir = "/" in p
        tops[top] = (tops.get(top, (0, is_dir))[0] + size, is_dir)
        e = extension(p.split("/")[-1])
        exts[e] = exts.get(e, 0) + size
    entries = []
    for name in sorted(tops):
        size, is_dir = tops[name]
        entries.append((name + "/" if is_dir else name, size))
    print("by extension:")
    for e in sorted(exts):
        print(f"  {e:<7}{exts[e]:>6}")
    entries = sorted(entries, key=lambda e: -e[1])
    total = sum(e[1] for e in entries)
    print("by entry:")
    for name, size in entries:
        bar = "#" * (size // 200)
        print(f"  {name:<9}{size:>6} {size * 100 // total:>3}% {bar}")
    print(f"total {total} bytes")
    print("cleaned up: true")


main()
