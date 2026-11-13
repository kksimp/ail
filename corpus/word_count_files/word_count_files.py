import os, tempfile, shutil


def count_text(text):
    return (text.count("\n"), len(text.split()), len(text.encode()))


def row(name, c):
    return f"{c[0]:>6}{c[1]:>7}{c[2]:>7} {name}"


def main():
    d = tempfile.mkdtemp(prefix="corpus_word_count_files")
    names = ["notes.txt", "poem.txt", "empty.txt"]
    texts = [
        "buy milk\ncall the bank\nfix the sink tap\n",
        "roses are red\n  violets are blue\nsugar is sweet\nand so are you\n",
        "",
    ]
    for n, t in zip(names, texts):
        with open(os.path.join(d, n), "w", newline="") as f:
            f.write(t)
    print(f"{'lines':>6}{'words':>7}{'bytes':>7} file")
    total = [0, 0, 0]
    for n in names:
        with open(os.path.join(d, n), newline="") as f:
            c = count_text(f.read())
        for i in range(3):
            total[i] += c[i]
        print(row(n, c))
    print(row("total", tuple(total)))
    try:
        open(os.path.join(d, "missing.txt")).read()
        print("unexpected")
    except OSError:
        print("missing.txt: cannot read")
    shutil.rmtree(d)
    print(f"cleaned up: {str(not os.path.exists(d)).lower()}")


main()
