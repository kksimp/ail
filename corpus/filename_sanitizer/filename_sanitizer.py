MAX_LEN = 24
RESERVED = ["CON", "PRN", "AUX", "NUL", "COM1", "LPT1"]


def safe_char(c):
    return c.isalpha() or c.isdigit() or c == "-" or c == "."


def clean(s):
    out = ""
    last_us = False
    for c in s:
        if safe_char(c):
            out = f"{out}{c}"
            last_us = False
        elif not last_us:
            out = f"{out}_"
            last_us = True
    return out


def trim_edges(s):
    cs = list(s)
    while cs and (cs[0] == "_" or cs[0] == "." or cs[0] == "-"):
        cs = cs[1:]
    while cs and (cs[-1] == "_" or cs[-1] == "."):
        cs = cs[:-1]
    return "".join(cs)


def split_ext(name):
    dot = -1
    for i in range(len(name)):
        if name[i] == ".":
            dot = i
    if dot <= 0:
        return (name, "")
    return (name[:dot], name[dot + 1:])


def sanitize(name):
    raw_stem, raw_ext = split_ext(name.strip())
    stem = trim_edges(clean(raw_stem))
    ext = clean(raw_ext).replace("-", "").replace(".", "").replace("_", "").lower()
    if len(ext) > 5:
        ext = ext[:5]
    if stem == "":
        stem = "unnamed"
    if stem.upper() in RESERVED:
        stem = f"_{stem}"
    room = MAX_LEN if ext == "" else MAX_LEN - len(ext) - 1
    if len(stem) > room:
        stem = trim_edges(stem[:room])
    return (stem, ext)


def join_name(stem, ext):
    if ext == "":
        return stem
    return f"{stem}.{ext}"


def sanitize_all(names):
    taken = set()
    out = []
    for n in names:
        stem, ext = sanitize(n)
        cand = join_name(stem, ext)
        k = 2
        while cand.lower() in taken:
            cand = join_name(f"{stem}-{k}", ext)
            k += 1
        taken.add(cand.lower())
        out.append(cand)
    return out


def main():
    names = [
        "My Report (final).DOCX",
        "  holiday photo #3.jpg ",
        "a/b\\c:d*e?.txt",
        "CON.txt",
        "..hidden..",
        "notes.TXT",
        "Notes.txt",
        "this is a very long file name that keeps going and going.markdown",
        "???.md",
        "archive.tar.gz",
    ]
    cleaned = sanitize_all(names)
    for i in range(len(names)):
        print(f"{names[i]:<30} -> {cleaned[i]}")


main()
