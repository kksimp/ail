def compile_re(pat):
    atoms = []
    at_start = False
    at_end = False
    for i, c in enumerate(pat):
        if i == 0 and c == "^":
            at_start = True
            continue
        if i == len(pat) - 1 and c == "$":
            at_end = True
            continue
        if c in "*+?" and atoms:
            atoms[-1][1] = c
            continue
        atoms.append([c, " "])
    return atoms, at_start, at_end


def hit(a, c):
    return a[0] == "." or a[0] == c


def here(re, ai, text, ti):
    atoms, at_start, at_end = re
    if ai == len(atoms):
        return (not at_end) or ti == len(text)
    a = atoms[ai]
    if a[1] == " ":
        if ti < len(text) and hit(a, text[ti]):
            return here(re, ai + 1, text, ti + 1)
        return False
    lo = 1 if a[1] == "+" else 0
    hi = 1 if a[1] == "?" else len(text)
    k = 0
    while ti + k < len(text) and k < hi and hit(a, text[ti + k]):
        k += 1
    j = k
    while j >= lo:
        if here(re, ai + 1, text, ti + j):
            return True
        j -= 1
    return False


def search(re, s):
    text = list(s)
    if re[1]:
        return 0 if here(re, 0, text, 0) else -1
    for start in range(len(text) + 1):
        if here(re, 0, text, start):
            return start
    return -1


def verdict(pat, s):
    at = search(compile_re(pat), s)
    if at < 0:
        return "no match"
    return "match at %d" % at


cases = [
    ("abc", "xxabcxx"),
    ("^abc", "xxabcxx"),
    ("abc$", "xxabc"),
    ("a.c", "a-c"),
    ("ab*c", "ac"),
    ("ab*c", "abbbc"),
    ("ab+c", "ac"),
    ("colou?r", "my color"),
    ("^a.*z$", "abc xyz"),
    ("^a.*z$", "abc xyz!"),
    ("x*", "aaa"),
    ("^$", ""),
    ("a.+b", "ab"),
    ("9+7", "id 99997"),
]
for pat, text in cases:
    print(f"{pat:<10} {text:<10} {verdict(pat, text)}")
