import subprocess
from pathlib import Path


def scan(root):
    out = []
    for file in sorted(p for p in root.rglob("*") if p.is_file()):
        try:
            text = file.read_text()
        except UnicodeDecodeError:
            continue
        r = subprocess.run(["git", "hash-object", "--stdin"], input=text, capture_output=True, text=True)
        out.append((file.relative_to(root).as_posix(), len(text.splitlines()), r.stdout.strip()[:8]))
    return out


def main():
    found = scan(Path(__file__).resolve().parents[5] / "examples/data")
    by_ext = {}
    for name, lines, _ in found:
        ext = name.rsplit(".", 1)[-1]
        by_ext[ext] = by_ext.get(ext, 0) + lines
    for ext in sorted(by_ext):
        print(f"{ext:<5}{by_ext[ext]:>5} lines")
    shown = set()
    for _, _, h in found:
        same = [n for n, _, g in found if g == h]
        if len(same) > 1 and h not in shown:
            shown.add(h)
            print(f"duplicate {h}: {', '.join(same)}")
    print(f"{len(found)} files, {sum(f[1] for f in found)} lines")


if __name__ == "__main__":
    main()
