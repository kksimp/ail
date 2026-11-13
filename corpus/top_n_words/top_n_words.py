def passage():
    lines = [
        "Data is only useful when the data is clean and the reports are clear.",
        "A report that hides its data is a story, and a story is not a report.",
        "Clean data, clear reports and honest tables make a useful team.",
        "Honest reports show the data, the gaps and the doubts.",
    ]
    return " ".join(lines)


def stopwords():
    return {"the", "is", "and", "a", "that", "its", "are", "not", "only", "when", "show"}


def tokens(text):
    out = []
    cur = ""
    for c in text.lower():
        if c.isascii() and c.isalpha():
            cur = f"{cur}{c}"
        elif cur != "":
            out.append(cur)
            cur = ""
    if cur != "":
        out.append(cur)
    return out


def keep(words, skip, min_len):
    return [w for w in words if len(w) >= min_len and w not in skip]


def top(words, n):
    counts = {}
    for w in words:
        counts[w] = counts.get(w, 0) + 1
    ordered = sorted(sorted(counts), key=lambda w: -counts[w])
    return [f"{w} {counts[w]}" for w in ordered[:n]]


def main():
    all_words = tokens(passage())
    kept = keep(all_words, stopwords(), 3)
    print(f"tokens: {len(all_words)}, kept: {len(kept)}")
    for entry in top(kept, 5):
        parts = entry.split(" ")
        n = int(parts[1])
        pct = n * 100 // len(kept)
        word = parts[0]
        print(f"{word:<9}{n:>3} {pct:>3}%")
    longest = sorted(sorted(kept), key=lambda w: -len(w))[0]
    print(f"longest: {longest}")


main()
