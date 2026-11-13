def is_upper(c):
    return "A" <= c <= "Z"


def is_lower(c):
    return "a" <= c <= "z"


def split_words(ident):
    cs = ident
    words = []
    cur = ""
    for i in range(len(cs)):
        c = cs[i]
        if c == "_" or c == "-" or c == " ":
            if cur != "":
                words.append(cur.lower())
            cur = ""
            continue
        if is_upper(c) and cur != "":
            prev = cs[i - 1]
            lower_next = i + 1 < len(cs) and is_lower(cs[i + 1])
            if is_lower(prev) or prev.isdigit() or lower_next:
                words.append(cur.lower())
                cur = ""
        cur = cur + c
    if cur != "":
        words.append(cur.lower())
    return words


def capitalize(w):
    return w[0:1].upper() + w[1:]


def to_snake(ident):
    return "_".join(split_words(ident))


def to_kebab(ident):
    return "-".join(split_words(ident))


def to_screaming(ident):
    return to_snake(ident).upper()


def to_pascal(ident):
    return "".join(capitalize(w) for w in split_words(ident))


def to_camel(ident):
    ws = split_words(ident)
    out = ""
    for i in range(len(ws)):
        if i == 0:
            out = ws[0]
        else:
            out = out + capitalize(ws[i])
    return out


def detect(ident):
    if "_" in ident:
        if ident == ident.upper():
            return "SCREAMING"
        return "snake"
    if "-" in ident:
        return "kebab"
    if is_upper(ident[0]):
        return "Pascal"
    if ident != ident.lower():
        return "camel"
    return "lower"


def main():
    names = ["parseHttpResponse", "HTTPServerConfig", "user_id_map", "MAX_RETRY_COUNT", "background-color", "getHTTP2Client", "x", "XMLParser"]
    for n in names:
        print("%-18s %-9s %s | %s | %s | %s" % (n, detect(n), to_snake(n), to_camel(n), to_pascal(n), to_kebab(n)))
    print(to_screaming("retryLimitMs"))


main()
