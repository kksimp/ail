TABLE = "A:.- B:-... C:-.-. D:-.. E:. F:..-. G:--. H:.... I:.. J:.--- K:-.- L:.-.. M:-- N:-. O:--- P:.--. Q:--.- R:.-. S:... T:- U:..- V:...- W:.-- X:-..- Y:-.-- Z:--.. 0:----- 1:.---- 2:..--- 3:...-- 4:....- 5:..... 6:-.... 7:--... 8:---.. 9:----."


def forward():
    m = {}
    for entry in TABLE.split(" "):
        parts = entry.split(":")
        m[parts[0]] = parts[1]
    return m


def backward():
    m = {}
    for entry in TABLE.split(" "):
        parts = entry.split(":")
        m[parts[1]] = parts[0]
    return m


def encode(text):
    table = forward()
    words = []
    for w in text.upper().split():
        letters = []
        for c in w:
            letters.append(table.get(c, "?"))
        words.append(" ".join(letters))
    return " / ".join(words)


def decode(morse):
    table = backward()
    words = []
    for w in morse.split(" / "):
        out = ""
        for code in w.split():
            out += table.get(code, "?")
        words.append(out)
    return " ".join(words)


def letter_units(code):
    n = 0
    for c in code:
        if c == ".":
            n += 1
        else:
            n += 3
    return n + len(code) - 1


def units(morse):
    total = 0
    words = morse.split(" / ")
    for wi in range(len(words)):
        letters = words[wi].split()
        for li in range(len(letters)):
            total += letter_units(letters[li])
            if li + 1 < len(letters):
                total += 3
        if wi + 1 < len(words):
            total += 7
    return total


def main():
    for phrase in ["SOS", "hello world", "AIL 2026", "Meet at 9", "a-b"]:
        m = encode(phrase)
        print("%-12s %s" % (phrase, m))
        print("             back: %s (%d units)" % (decode(m), units(m)))


main()
