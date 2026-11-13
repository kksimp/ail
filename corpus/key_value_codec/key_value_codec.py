def escape(s):
    out = ""
    for c in s:
        if c == "\\" or c == ";" or c == "=":
            out += "\\" + c
        else:
            out += c
    return out


def encode(pairs):
    return ";".join(f"{escape(k)}={escape(v)}" for k, v in pairs)


def decode(text):
    out = []
    key = ""
    val = ""
    in_key = True
    esc = False
    for c in text:
        if esc:
            if in_key:
                key += c
            else:
                val += c
            esc = False
        elif c == "\\":
            esc = True
        elif c == "=" and in_key:
            in_key = False
        elif c == ";":
            if not (in_key and key == ""):
                out.append((key, val))
            key = ""
            val = ""
            in_key = True
        elif in_key:
            key += c
        else:
            val += c
    if not (in_key and key == ""):
        out.append((key, val))
    return out


def samples():
    return [
        [("name", "Ann Lee"), ("role", "admin")],
        [("expr", "a=b;c"), ("path", "C:\\temp\\x")],
        [("empty", ""), ("k;ey", "v=1")],
        [],
    ]


for pairs in samples():
    text = encode(pairs)
    back = decode(text)
    print(f"{len(pairs)} pairs -> '{text}' ({len(text)} chars)")
    for k, v in back:
        print(f"  [{k}] = [{v}]")
    print(f"  round trip: {str(back == pairs).lower()}")
loose = decode("a=1;;b;c=x=y;")
print(f"loose: {len(loose)} pairs")
for k, v in loose:
    print(f"  [{k}] = [{v}]")
