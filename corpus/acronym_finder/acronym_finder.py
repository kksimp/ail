def is_word_char(c):
    return c.isalpha() or c.isdigit()


def clean(word):
    lo = 0
    hi = len(word)
    while lo < hi and not is_word_char(word[lo]):
        lo += 1
    while hi > lo and not is_word_char(word[hi - 1]):
        hi -= 1
    return word[lo:hi]


def is_acronym(s):
    if len(s) < 2:
        return False
    for c in s:
        if not (c.isalpha() and c == c.upper()):
            return False
    return True


def initials(words):
    out = ""
    for w in words:
        if w == "":
            continue
        out += w[:1].upper()
    return out


def definition(words, i):
    raw = words[i]
    name = clean(raw)
    if not raw.startswith("(") or not is_acronym(name):
        return None
    n = len(name)
    if i < n:
        return None
    start = i - n
    before = [clean(w) for w in words[start:i]]
    if initials(before) == name:
        return " ".join(before)
    return None


def main():
    text = " ".join([
        "The Portable Network Graphics (PNG) format replaced GIF in many tools.",
        "A Central Processing Unit (CPU) runs code; the CPU and the GPU share RAM.",
        "HTTP and TLS secure the web, and Transport Layer Security (TLS) is the",
        "successor of SSL. Sending a PNG over HTTP needs no CPU tricks. The",
        "Domain Name System (DNS) maps names, but (OK) is not an expansion.",
    ])
    words = text.split()
    counts = {}
    defined = {}
    for i in range(len(words)):
        word = clean(words[i])
        if not is_acronym(word):
            continue
        counts[word] = counts.get(word, 0) + 1
        d = definition(words, i)
        if d is None:
            continue
        defined[word] = d
    print("defined:")
    for key in sorted(defined):
        print(f"  {key:<5} {defined[key]} (used {counts[key]}x)")
    rest = []
    for key2 in sorted(counts):
        if key2 not in defined:
            rest.append((key2, counts[key2]))
    print("undefined:")
    for f in sorted(rest, key=lambda x: -x[1]):
        print(f"  {f[0]:<5} {f[1]}")


main()
