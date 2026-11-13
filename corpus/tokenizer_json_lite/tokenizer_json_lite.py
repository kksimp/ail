class JsonError(Exception):
    pass


def punct(c):
    return {"{": "lbrace", "}": "rbrace", "[": "lbracket", "]": "rbracket", ":": "colon", ",": "comma"}.get(c)


def lex_string(cs, start):
    i = start + 1
    body = ""
    closed = False
    while i < len(cs) and not closed:
        d = cs[i]
        if d == '"':
            closed = True
            i += 1
        elif d == "\\" and i + 1 < len(cs):
            body += d + cs[i + 1]
            i += 2
        else:
            body += d
            i += 1
    if not closed:
        raise JsonError("unterminated string at %d" % start)
    return ("str", body, start)


def lex_number(cs, start):
    i = start
    body = ""
    while i < len(cs) and (cs[i] in "0123456789" or cs[i] in "-+.eE"):
        body += cs[i]
        i += 1
    return ("num", body, start)


def lex_word(cs, start):
    i = start
    body = ""
    while i < len(cs) and cs[i].isalpha():
        body += cs[i]
        i += 1
    if body not in ("true", "false", "null"):
        raise JsonError("bad literal %s at %d" % (body, start))
    return ("lit", body, start)


def tokenize(src):
    cs = list(src)
    out = []
    i = 0
    while i < len(cs):
        c = cs[i]
        if c.isspace():
            i += 1
            continue
        k = punct(c)
        if k is not None:
            out.append((k, c, i))
            i += 1
            continue
        if c == '"':
            tok = lex_string(cs, i)
        elif c == "-" or c.isdigit():
            tok = lex_number(cs, i)
        elif c.isalpha():
            tok = lex_word(cs, i)
        else:
            raise JsonError("bad character %s at %d" % (c, i))
        out.append(tok)
        i += len(tok[1])
        if tok[0] == "str":
            i += 2
    return out


def max_depth(toks):
    depth = 0
    deepest = 0
    for t in toks:
        if t[0] in ("lbrace", "lbracket"):
            depth += 1
            deepest = max(deepest, depth)
        elif t[0] in ("rbrace", "rbracket"):
            depth -= 1
    return deepest


def show(src):
    print(src)
    try:
        toks = tokenize(src)
    except JsonError as e:
        print(f"  error: {e}")
        return
    counts = {}
    for t in toks:
        counts[t[0]] = counts.get(t[0], 0) + 1
    print(f"  {len(toks)} tokens, depth {max_depth(toks)}")
    for name in sorted(counts):
        print(f"  {name:<9} {counts[name]}")
    shown = [t[1] for t in toks if t[0] in ("str", "lit")]
    print("  words: " + " ".join(shown))


show('{"name": "ail", "tags": ["a", "b"], "n": -12}')
show('[1, 2.5e3, true, false, null, {"k": []}]')
show('["x\\"y"]')
show('{"a": tru}')
show('[1, "open]')
show('{"a": @}')
