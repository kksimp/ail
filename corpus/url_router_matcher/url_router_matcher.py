def segs(path):
    return [s for s in path.split("/") if s != ""]


def match_path(pattern, path):
    ps = segs(pattern)
    xs = segs(path)
    params = {}
    for i in range(len(ps)):
        p = ps[i]
        if p.startswith("*"):
            params[p[1:]] = "/".join(xs[i:])
            return params
        if i >= len(xs):
            return None
        if p.startswith(":"):
            params[p[1:]] = xs[i]
        elif p != xs[i]:
            return None
    if len(xs) != len(ps):
        return None
    return params


ROUTES = [
    ("GET", "/", "home"),
    ("GET", "/users/:id", "show_user"),
    ("DELETE", "/users/:id", "delete_user"),
    ("GET", "/users/:id/posts/:post", "show_post"),
    ("GET", "/users/me", "me"),
    ("GET", "/static/*path", "static_file"),
]


def dispatch(table, method, path):
    allowed = []
    for m, pattern, name in table:
        found = match_path(pattern, path)
        if found is None:
            continue
        if m != method:
            allowed.append(m)
            continue
        shown = [f"{k}={found[k]}" for k in sorted(found)]
        return f"{name} [{', '.join(shown)}]"
    if not allowed:
        return "404 not found"
    return "405 allowed: " + ",".join(allowed)


requests = [
    ("GET", "/"),
    ("GET", "/users/42"),
    ("GET", "/users/me"),
    ("DELETE", "/users/7/"),
    ("POST", "/users/7"),
    ("GET", "/users/7/posts/99"),
    ("GET", "/static/css/site/main.css"),
    ("GET", "/static"),
    ("GET", "/users/7/posts"),
    ("PUT", "/nothing/here"),
]
for m, p in requests:
    print(f"{m:<6} {p:<28} -> {dispatch(ROUTES, m, p)}")
