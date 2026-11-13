import os, tempfile, shutil


def count_in_file(p):
    with open(p, newline="") as f:
        return f.read().count("colour")


def replace_in_file(p, old, new):
    with open(p, newline="") as f:
        text = f.read()
    n = text.count(old)
    if n > 0:
        with open(p, "w", newline="") as f:
            f.write(text.replace(old, new))
    return n


def report(changes):
    files = 0
    total = 0
    for name, count in changes:
        if count > 0:
            files += 1
            total += count
            print(f"  {name:<10}{count:>3} replaced")
    print(f"  {files} files changed, {total} replacements")
    return total


def main():
    d = tempfile.mkdtemp(prefix="corpus_find_replace_files")
    names = ["a.txt", "b.md", "c.cfg", "d.txt"]
    texts = [
        "The colour red and the colour blue.\nNo match here.\n",
        "# colour guide\nPick a colour.\n",
        "theme=dark\nname=plain\n",
        "colourful colours: colour, colour, colour\n",
    ]
    for n, t in zip(names, texts):
        with open(os.path.join(d, n), "w", newline="") as f:
            f.write(t)
    print("dry run:")
    before = 0
    for n in names:
        c = count_in_file(os.path.join(d, n))
        before += c
        print(f"  {n:<10}{c:>3} matches")
    print("apply:")
    changes = [(n, replace_in_file(os.path.join(d, n), "colour", "color")) for n in names]
    done = report(changes)
    print(f"expected {before}, replaced {done}")
    print("verify:")
    left = sum(count_in_file(os.path.join(d, n)) for n in names)
    print(f"  matches left: {left}")
    with open(os.path.join(d, "b.md"), newline="") as f:
        print("b.md now: " + " | ".join(f.read().splitlines()))
    shutil.rmtree(d)
    print(f"cleaned up: {str(not os.path.exists(d)).lower()}")


main()
