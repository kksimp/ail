class Tally:
    def __init__(self, name, hits):
        self.name = name
        self.hits = hits


def extract_words(text):
    out = []
    cur = ""
    for c in text.lower():
        if c.isalpha():
            cur += c
        elif cur != "":
            out.append(cur)
            cur = ""
    if cur != "":
        out.append(cur)
    return out


def count_sentences(text):
    n = 0
    for c in text:
        if c in ".!?":
            n += 1
    return n


def top_words(words, limit):
    counts = {}
    for w in words:
        counts[w] = counts.get(w, 0) + 1
    items = [Tally(w, counts[w]) for w in sorted(counts)]
    ranked = sorted(items, key=lambda t: -t.hits)
    return ranked[:limit]


def top_letters(words, limit):
    freq = [0] * 26
    for w in words:
        for c in w:
            freq[ord(c) - 97] += 1
    items = []
    for i in range(26):
        if freq[i] > 0:
            items.append(Tally(chr(97 + i), freq[i]))
    ranked = sorted(items, key=lambda t: -t.hits)
    return ranked[:limit]


def bar(n):
    return "#" * n


def main():
    text = "\n".join([
        "The quick brown fox jumps over the lazy dog. The dog sleeps!",
        "Is the fox quick? Yes, the fox is quick and the dog is lazy.",
        "",
        "A third paragraph ends here, quickly.",
    ])
    words = extract_words(text)
    letters = 0
    longest = ""
    for w in words:
        letters += len(w)
        if len(w) > len(longest):
            longest = w
    avg = letters * 100 // len(words)
    print(f"characters: {len(text)}")
    print(f"lines: {len(text.split(chr(10)))}")
    print(f"sentences: {count_sentences(text)}")
    print(f"words: {len(words)}")
    print(f"average word length: {avg // 100}.{avg % 100:02}")
    print(f"longest word: {longest}")
    print("top words:")
    for t in top_words(words, 4):
        print(f"  {t.name:<8}{t.hits}")
    print("top letters:")
    for t in top_letters(words, 3):
        print(f"  {t.name} {t.hits}")
    print("word lengths:")
    by_len = {}
    for w in words:
        by_len[len(w)] = by_len.get(len(w), 0) + 1
    for n in range(1, len(longest) + 1):
        hits = by_len.get(n, 0)
        print(f"  {n:>2} {bar(hits)} {hits}")


main()
