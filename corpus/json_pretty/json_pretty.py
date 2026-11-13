def pad(depth):
    return "  " * depth


def pretty(src):
    cs = list(src)
    out = ""
    depth = 0
    in_str = False
    esc = False
    i = 0
    while i < len(cs):
        c = cs[i]
        i += 1
        if in_str:
            out += c
            if esc:
                esc = False
            elif c == "\\":
                esc = True
            elif c == '"':
                in_str = False
        elif c == '"':
            in_str = True
            out += c
        elif c == "{" or c == "[":
            if i < len(cs) and (cs[i] == "}" or cs[i] == "]"):
                out += c + cs[i]
                i += 1
            else:
                depth += 1
                out += c + "\n" + pad(depth)
        elif c == "}" or c == "]":
            depth -= 1
            out += "\n" + pad(depth) + c
        elif c == ",":
            out += ",\n" + pad(depth)
        elif c == ":":
            out += ": "
        elif not c.isspace():
            out += c
    return out


def minify(src):
    out = ""
    in_str = False
    esc = False
    for c in src:
        if in_str:
            out += c
            if esc:
                esc = False
            elif c == "\\":
                esc = True
            elif c == '"':
                in_str = False
        elif c == '"':
            in_str = True
            out += c
        elif not c.isspace():
            out += c
    return out


def max_depth(src):
    depth = 0
    best = 0
    in_str = False
    esc = False
    for c in src:
        if in_str:
            if esc:
                esc = False
            elif c == "\\":
                esc = True
            elif c == '"':
                in_str = False
        elif c == '"':
            in_str = True
        elif c == "{" or c == "[":
            depth += 1
            best = max(best, depth)
        elif c == "}" or c == "]":
            depth -= 1
    return best


def main():
    parts = [
        '{"name":"ail","tags":["fast","safe"],',
        '"nested":{"a":1,"b":[1,2,{"c":null}],"empty":{},"list":[]},',
        '"quote":"say \\"hi\\", ok"}',
    ]
    doc = "".join(parts)
    text = pretty(doc)
    print(text)
    print("lines: %d" % len(text.split("\n")))
    print("depth: %d" % max_depth(doc))
    print("minified again: %s" % ("true" if minify(text) == doc else "false"))
    print(pretty("[]"))
    print(pretty("[[],{}]"))


main()
