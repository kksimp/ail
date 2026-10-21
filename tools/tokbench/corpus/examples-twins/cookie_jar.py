# Python twin of examples/cookie_jar.ail: the same session, the same output.
from urllib.parse import urlsplit

jar = []  # (name, value, host, domain, path, secure, httponly, host_only, order)


def default_path(p):
    return p[: p.rfind("/") + 1] or "/"


def set_cookie(url, line):
    u = urlsplit(url)
    host = u.hostname
    parts = [x.strip() for x in line.split(";")]
    name, value = parts[0].split("=", 1)
    attrs = {}
    flags = set()
    for a in parts[1:]:
        k, _, v = a.partition("=")
        if k.lower() in ("secure", "httponly"):
            flags.add(k.lower())
        else:
            attrs[k.lower()] = v
    domain = attrs.get("domain")
    if domain is not None:
        d = domain.lstrip(".")
        if host != d and not host.endswith("." + d):
            return False
        host_only = False
    else:
        d = host
        host_only = True
    path = attrs.get("path", default_path(u.path or "/"))
    jar[:] = [c for c in jar if not (c[0] == name and c[3] == d and c[4] == path)]
    jar.append((name, value, host, d, path, "secure" in flags, "httponly" in flags, host_only, len(jar)))
    return True


def matching(url):
    u = urlsplit(url)
    hits = []
    for c in jar:
        name, value, host, d, path, secure, httponly, host_only, order = c
        if host_only:
            ok = u.hostname == d
        else:
            ok = u.hostname == d or u.hostname.endswith("." + d)
        ok = ok and (u.path or "/").startswith(path) and (not secure or u.scheme == "https")
        if ok:
            hits.append(c)
    hits.sort(key=lambda c: (-len(c[4]), c[8]))
    return hits


def header(url):
    return "; ".join(f"{c[0]}={c[1]}" for c in matching(url))


def value(url, name):
    for c in matching(url):
        if c[0] == name:
            return c[1]
    return None


def for_script(url):
    return {c[0]: c[1] for c in matching(url) if not c[6]}


site = "https://api.shop.test/"
set_cookie(site, "sid=abc123; Path=/; Secure; HttpOnly")
set_cookie(site, "theme=dark; Path=/prefs")
set_cookie(site, "cart=3")
print(header("https://api.shop.test/prefs/list"))
print(header("https://api.shop.test/orders"))
print(f"over http: [{header('http://api.shop.test/orders')}]")
print(f"sid: {value(site, 'sid') or 'none'}")
print(f"scripts see {len(for_script(site))} of {len(jar)} cookies")
print(f"foreign domain refused: {str(set_cookie(site, 'x=1; Domain=other.test')).lower()}")
set_cookie(site, "cart=5")
print(f"cart: {value(site, 'cart') or 'none'}, cookies: {len(jar)}")
jar.clear()
print(f"after clear: [{header(site)}]")
