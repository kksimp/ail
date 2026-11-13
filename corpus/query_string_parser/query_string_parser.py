def split_once(s, sep):
    i = s.find(sep)
    if i < 0:
        return (s, "")
    return (s[:i], s[i + len(sep):])


HEX = "0123456789abcdefABCDEF"


def decode(s):
    cs = list(s)
    out = ""
    i = 0
    while i < len(cs):
        c = cs[i]
        if c == "%" and i + 2 < len(cs):
            if cs[i + 1] in HEX and cs[i + 2] in HEX:
                out += chr(int(cs[i + 1] + cs[i + 2], 16))
                i += 3
                continue
        if c == "+":
            out += " "
        else:
            out += c
        i += 1
    return out


def encode(s):
    digits = "0123456789ABCDEF"
    out = ""
    for c in s:
        if c.isalpha() or c.isdigit() or c in "-_.~":
            out += c
        elif c == " ":
            out += "+"
        else:
            n = ord(c)
            out += "%" + digits[n // 16] + digits[n % 16]
    return out


def parse_query(raw):
    params = {}
    q = raw[1:] if raw.startswith("?") else raw
    for pair in q.split("&"):
        if pair == "":
            continue
        k, v = split_once(pair, "=")
        key = decode(k)
        if key.endswith("[]"):
            key = key[:-2]
        params.setdefault(key, []).append(decode(v))
    return params


def canonical(params):
    parts = []
    for k in sorted(params):
        for v in params[k]:
            parts.append(encode(k) + "=" + encode(v))
    return "&".join(parts)


def first_of(params, key):
    vals = params.get(key)
    if vals is None or not vals:
        return None
    return vals[0]


def int_of(params, key, fallback):
    v = first_of(params, key)
    if v is None:
        return fallback
    try:
        return int(v)
    except ValueError:
        return fallback


def show(raw):
    params = parse_query(raw)
    print(f"query '{raw}'")
    for k in sorted(params):
        print(f"  {k} = {' | '.join(params[k])}")
    print(f"  canonical: '{canonical(params)}'")


show("?q=hello+world&page=3&tag=a&tag=b&tag=c")
show("name=Ann%20Lee&city=S%26o&flag&empty=&x[]=1&x[]=2")
show("a=1&&b=%zz&c=100%&d=%4")
show("")
params = parse_query("q=ail&page=3&per=many&tag=a&tag=b")
print(f"page: {int_of(params, 'page', 1)}")
print(f"per: {int_of(params, 'per', 20)}")
print(f"limit: {int_of(params, 'limit', 50)}")
t = first_of(params, "tag")
print(f"first tag: {t if t is not None else '-'}")
t = first_of(params, "sort")
print(f"sort: {t if t is not None else '-'}")
