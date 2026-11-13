class Stats:
    def __init__(self, word, vowels, consonants, longest, pct):
        self.word = word
        self.vowels = vowels
        self.consonants = consonants
        self.longest = longest
        self.pct = pct


def is_vowel(c):
    return c in "aeiou"


def analyze(word):
    vowels = 0
    consonants = 0
    run = 0
    longest = 0
    for c in word.lower():
        if is_vowel(c):
            vowels += 1
            run += 1
            longest = max(longest, run)
        else:
            run = 0
            if c.isalpha():
                consonants += 1
    letters = vowels + consonants
    pct = 0 if letters == 0 else vowels * 100 // letters
    return Stats(word, vowels, consonants, longest, pct)


def clean_words(text):
    out = []
    for w in text.split():
        word = ""
        for c in w:
            if c.isalpha():
                word += c
        if word != "":
            out.append(word)
    return out


def bar(n):
    return "#" * n


def main():
    text = "Rhythm and beautiful queue: the sky, a quiet idea, strengths of AIL."
    rows = [analyze(w) for w in clean_words(text)]
    print("word        v  c run  pct")
    for s in sorted(rows, key=lambda x: -x.pct):
        print("%-10s %2d %2d %3d %3d%%" % (s.word, s.vowels, s.consonants, s.longest, s.pct))
    total = 0
    letters = 0
    for s in rows:
        total += s.vowels
        letters += s.vowels + s.consonants
    print("total vowels %d of %d letters, %d%%" % (total, letters, total * 100 // letters))
    lower = text.lower()
    for v in "aeiou":
        n = len([x for x in lower if x == v])
        print("%s %2d %s" % (v, n, bar(n)))


main()
