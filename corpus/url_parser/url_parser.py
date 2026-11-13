def split_once(s, sep):
    i = s.find(sep)
    if i < 0:
        return (s, "")
    return (s[:i], s[i + len(sep):])


def default_port(scheme):
    return {"http": 80, "https": 443, "ftp": 21}.get(scheme, 0)


def to_int(s):
    if s.isdigit() and s.isascii():
        return int(s)
    return None


def parse_url(text):
    scheme, rest = split_once(text, "://")
    if rest == "":
        return None
    before_frag, frag = split_once(rest, "#")
    before_query, query = split_once(before_frag, "?")
    slash = before_query.find("/")
    auth = before_query
    path = "/"
    if slash >= 0:
        auth = before_query[:slash]
        path = before_query[slash:]
    host, port_text = split_once(auth, ":")
    port = to_int(port_text)
    if port is None:
        port = default_port(scheme)
    if host == "":
        return None
    return {"scheme": scheme, "host": host, "port": port, "path": path,
            "query": query, "fragment": frag}


def pct_decode(s):
    out = ""
    i = 0
    while i < len(s):
        c = s[i]
        if c == "%" and i + 2 < len(s):
            h = s[i + 1:i + 3]
            if all(x in "0123456789abcdefABCDEF" for x in h):
                out += chr(int(h, 16))
                i += 3
                continue
        if c == "+":
            out += " "
        else:
            out += c
        i += 1
    return out


def parse_query(q):
    params = {}
    if q == "":
        return params
    for pair in q.split("&"):
        k, v = split_once(pair, "=")
        params[pct_decode(k)] = pct_decode(v)
    return params


def show(text):
    print(text)
    u = parse_url(text)
    if u is None:
        print("  invalid")
        return
    print("  scheme=%s host=%s port=%d" % (u["scheme"], u["host"], u["port"]))
    print("  path=%s fragment=%s" % (u["path"], u["fragment"]))
    params = parse_query(u["query"])
    for k in sorted(params):
        print("  param %s = %s" % (k, params[k]))


def main():
    show("https://example.org/a/b?q=hello+world&lang=en#top")
    show("http://localhost:8080/api/v1/items?id=42&tag=a%26b&tag2=%7Ex")
    show("ftp://files.example.net")
    show("https://example.org:99/?")
    show("not a url")
    show("http:///nohost")


main()
