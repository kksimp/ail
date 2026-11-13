import os, tempfile


def clean(text):
    return "".join(ch if ch.isalpha() else " " for ch in text).lower()


def count_words(text):
    counts = {}
    for w in clean(text).split():
        counts[w] = counts.get(w, 0) + 1
    return counts


def lookup(idx, w):
    if w not in idx:
        return "-"
    return " ".join(idx[w])


def main():
    d = tempfile.mkdtemp(prefix="corpus_parallel_word_index")
    names = ["a.txt", "b.txt", "c.txt", "d.txt"]
    texts = [
        "The quick brown fox jumps over the lazy dog.",
        "A river runs past the old mill; the river is cold.",
        "The fox met the dog by the river.",
        "Quick quick: the mill is closed.",
    ]
    for n, t in zip(names, texts):
        with open(os.path.join(d, n), "w", newline="") as f:
            f.write(t)
    parts = []
    for n in names:
        with open(os.path.join(d, n), newline="") as f:
            parts.append((n, count_words(f.read())))
    idx = {}
    for doc, counts in sorted(parts, key=lambda p: p[0]):
        for w, c in counts.items():
            idx.setdefault(w, []).append(f"{doc}:{c}")
    postings = sum(len(v) for v in idx.values())
    print(f"{len(idx)} words, {postings} postings")
    shared = [w for w in sorted(idx) if len(idx[w]) >= 3]
    print(f"in 3+ docs: {' '.join(shared)}")
    for q in ["fox", "river", "the", "quick", "zebra"]:
        print(f"{q:<6}{lookup(idx, q)}")
    for n in names:
        os.remove(os.path.join(d, n))
    os.rmdir(d)
    print(f"cleaned up: {str(not os.path.exists(d)).lower()}")


main()
