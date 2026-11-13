def words_of(title):
    words = []
    cur = ""
    for c in title.lower():
        if c.isalnum():
            cur += c
        elif cur != "":
            words.append(cur)
            cur = ""
    if cur != "":
        words.append(cur)
    return words


def slugify(title, max_len):
    out = ""
    for w in words_of(title):
        cand = w if out == "" else out + "-" + w
        if len(cand) > max_len:
            if out == "":
                out = w[:max_len]
            break
        out = cand
    return out


def unique_slugs(titles, max_len):
    seen = {}
    out = []
    for t in titles:
        base = slugify(t, max_len)
        seen[base] = seen.get(base, 0) + 1
        n = seen[base]
        if n == 1:
            out.append(base)
        else:
            out.append("%s-%d" % (base, n))
    return out


def main():
    titles = [
        "Hello, World!",
        "  Parsing JSON: A Practical Guide  ",
        "C++ vs. Rust vs. AIL (2026 edition)",
        "Hello, World!",
        "100% Memory-Safe",
        "!!!",
        "Hello World",
        "An extraordinarily long title that keeps going and going",
    ]
    for t in titles:
        print("%-58s -> '%s'" % (t, slugify(t, 30)))
    print("unique:")
    for s in unique_slugs(titles, 30):
        print("  " + s)


main()
