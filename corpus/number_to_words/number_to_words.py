SMALL = [
    "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine",
    "ten", "eleven", "twelve", "thirteen", "fourteen", "fifteen", "sixteen",
    "seventeen", "eighteen", "nineteen",
]
TENS = ["", "", "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety"]
SCALES = ["", "thousand", "million", "billion"]


def below_thousand(n):
    parts = []
    rest = n
    if rest >= 100:
        parts.append("%s hundred" % SMALL[rest // 100])
        rest = rest % 100
    if rest >= 20:
        t = TENS[rest // 10]
        if rest % 10 > 0:
            parts.append("%s-%s" % (t, SMALL[rest % 10]))
        else:
            parts.append(t)
    elif rest > 0:
        parts.append(SMALL[rest])
    return " ".join(parts)


def to_words(n):
    if n == 0:
        return "zero"
    if n < 0:
        return "minus " + to_words(0 - n)
    parts = []
    rest = n
    level = 0
    while rest > 0:
        chunk = rest % 1000
        if chunk > 0:
            name = SCALES[level]
            text = below_thousand(chunk)
            if name == "":
                parts.append(text)
            else:
                parts.append("%s %s" % (text, name))
        rest = rest // 1000
        level += 1
    return " ".join(parts[::-1])


def word_values():
    m = {}
    for i in range(len(SMALL)):
        m[SMALL[i]] = i
    for i in range(2, len(TENS)):
        m[TENS[i]] = i * 10
    return m


def from_words(text):
    values = word_values()
    negative = False
    total = 0
    chunk = 0
    for w in text.replace("-", " ").split():
        if w == "minus":
            negative = True
        elif w == "hundred":
            chunk = chunk * 100
        elif w in ("thousand", "million", "billion"):
            scale = 1000
            if w == "million":
                scale = 1000000
            if w == "billion":
                scale = 1000000000
            total += chunk * scale
            chunk = 0
        elif w in values:
            chunk += values[w]
        else:
            return None
    total += chunk
    if negative:
        return 0 - total
    return total


def main():
    for n in [0, 7, 13, 20, 42, 100, 101, 999, 1000, 1234, 20000, 100001, 1000000, 7654321, 2000000000, 1234567891, -45]:
        print("%11d  %s" % (n, to_words(n)))
    for p in ["forty-two", "one thousand two hundred thirty-four", "minus nine", "seven bazillion"]:
        v = from_words(p)
        if v is None:
            print("%s -> invalid" % p)
            continue
        print("%s -> %d" % (p, v))
    bad = 0
    for n in range(5000):
        back = from_words(to_words(n))
        if back is None:
            back = -1
        if back != n:
            bad += 1
    print("round trip failures in 0..4999: %d" % bad)


main()
