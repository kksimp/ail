def sample():
    lines = [
        "The quick brown fox jumps over the lazy dog.",
        "The dog barks; the fox runs away, and the dog follows the fox.",
        "A quick fox is a happy fox, and a lazy dog is a sleepy dog.",
    ]
    return " ".join(lines)


def split_words(text):
    out = []
    cur = ""
    for c in text.lower():
        if ("a" <= c <= "z"):
            cur += c
        elif cur != "":
            out.append(cur)
            cur = ""
    if cur != "":
        out.append(cur)
    return out


def count_words(words):
    counts = {}
    for w in words:
        counts[w] = counts.get(w, 0) + 1
    return counts


def rank(counts):
    return sorted(sorted(counts.keys()), key=lambda w: -counts[w])


def main():
    words = split_words(sample())
    counts = count_words(words)
    print(f"words: {len(words)}, unique: {len(counts)}")
    ranked = rank(counts)
    for w in ranked[:6]:
        print(f"{w:<8}{counts[w]:>3}")
    longest = sorted(words, key=lambda w: -len(w))[0]
    print(f"longest: {longest} ({len(longest)})")
    once = 0
    for w in counts:
        if counts[w] == 1:
            once += 1
    print(f"seen once: {once}")


main()
