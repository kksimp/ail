def normalize(text):
    out = ""
    for c in text.lower():
        if c.isalpha() or c == " ":
            out += c
        else:
            out += " "
    return " ".join(out.split())


def char_ngrams(text, n):
    counts = {}
    for i in range(len(text)):
        if i + n <= len(text):
            gram = text[i:i + n]
            if " " not in gram:
                counts[gram] = counts.get(gram, 0) + 1
    return counts


def word_ngrams(text, n):
    ws = text.split()
    counts = {}
    for i in range(len(ws)):
        if i + n <= len(ws):
            gram = " ".join(ws[i:i + n])
            counts[gram] = counts.get(gram, 0) + 1
    return counts


def top(counts, limit):
    keys = sorted(counts.keys())
    ranked = sorted(keys, key=lambda k: -counts[k])
    return ["%s=%d" % (k, counts[k]) for k in ranked[:limit]]


def main():
    raw = "The cat sat on the mat. The cat saw the other cat, and the other cat ran; the mat stayed."
    text = normalize(raw)
    print(text)
    print("char trigrams: " + " ".join(top(char_ngrams(text, 3), 6)))
    print("char bigrams:  " + " ".join(top(char_ngrams(text, 2), 6)))
    print("word bigrams:  " + " | ".join(top(word_ngrams(text, 2), 5)))
    print("word unigrams: " + " ".join(top(word_ngrams(text, 1), 5)))
    print("distinct word trigrams: %d" % len(word_ngrams(text, 3)))


main()
