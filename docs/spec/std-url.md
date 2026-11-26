# Standard library: `std.url`

Chapter key: `std`

> **Status:** Drafted. Written in AIL (`std/url/url.ail`, ADR 1214).

URLs and URI references by [RFC 3986](https://www.rfc-editor.org/rfc/rfc3986): split one into its parts,
write it back, resolve a link against the page it is on, and read and write percent-encoded text and
queries. Every function is pure (`eff.infer.1`). Internationalised host names (IDNA) and the browser rules
of the WHATWG URL standard are not in this module (`std.url.11`).

## The module

**std.url.1** — `use std.url` provides `parse`, `resolve`, `join_path`, `encode`, `decode`, `encode_form`,
`decode_form`, `parse_query`, `query_map` and `build_query`, the method `Url.resolve`, and the types `Url`,
`UrlError` and `UrlKind`. The module is in scope as `url` without the `use` line (`std.use.4`); a local
named `url` hides it, so name a URL text `link`, `addr` or `raw`. *(ADR 1214)*

**std.url.2** — `parse(s str) Url ! UrlError` splits `s` into the fields of a `Url`: `scheme`, `user` (the
userinfo before `@`, `name:password` included), `host`, `port` (an `int?`), `path`, `query` (without the
`?`) and `fragment` (without the `#`). A part that is absent is `""`, and `port` is `none`; an empty port
(`http://h:/`) is `none` too. A string with no scheme is a relative reference: `parse("../a?b")` has the path
`../a` and the query `b`. The host of `http://[::1]:80/` is `::1`, without the brackets. A scheme is a letter
followed by letters, digits, `+`, `-` and `.`, so `localhost:8080` has the scheme `localhost`; write
`//localhost:8080` or `http://localhost:8080` for a host and port. *(ADR 1214)*

**std.url.3** — `parse` writes `scheme` and `host` in lower case and keeps every other part as written: the
parts stay percent-encoded (`/a%20b` is the path `/a%20b`), and `url.decode` reads one. *(ADR 1214)*

**std.url.4** — `u.to_str() str` is the text of a `Url` by RFC 3986 section 5.3, and is its text form, so
`print(u)` and `"{u}"` show it (`trait.std.3`). An authority (`//`) is written when `host`, `user` or `port`
is set, or the scheme is `file`; a host with a `:` is written in brackets; a path that does not start with
`/` after an authority gets one; a relative path whose first segment holds a `:` is written `./a:b`. An empty
query or fragment is not written, so `to_str` of `parse("a?#")` is `a`. *(ADR 1214)*

```ail rules=std.url.1,std.url.2,std.url.3,std.url.4
fn main() !
    u := url.parse("HTTPS://ann@Example.COM:8080/a%20b?q=1#top")?
    print(u.scheme)                 # https
    print(u.user)                   # ann
    print(u.host)                   # example.com
    print(u.port else 443)          # 8080
    print(u.path)                   # /a%20b
    print(u.query)                  # q=1
    print(u.fragment)               # top
    print(u)                        # https://ann@example.com:8080/a%20b?q=1#top
    v := url.parse("http://[::1]/x")?
    print(v.host)                   # ::1
    print(v.port else 80)           # 80
    print(url.Url(scheme: "https", host: "h", path: "/p"))   # https://h/p
```

## Resolving and joining

**std.url.5** — `resolve(base str, r str) str ! UrlError` is the text of the reference `r` resolved against
the URL `base` by RFC 3986 section 5.2, strictly: what a link `r` on the page `base` points to.
`u.resolve(r Url) Url` does the same on parsed values and cannot fail. `.` and `..` segments are applied (section 5.2.4);
`..` above the root stays at the root. A reference with a scheme replaces the base, even the base's own
scheme (`http:g` is `http:g`). *(ADR 1214)*

**std.url.6** — `join_path(base str, parts List[str]) str ! UrlError` is `base` with the path segments of
`parts` added after its path, one `/` between each. Each `/`-separated piece of a part is a segment; empty
pieces are dropped; a character a path segment cannot hold (a space, `?`, `#`, a `%` that starts no escape)
is percent-encoded; `.` and `..` are applied; and a last part that ends in `/` keeps the `/`. The query and
fragment of `base` are kept. *(ADR 1214)*

```ail rules=std.url.5,std.url.6
fn main() !
    page := "http://a/b/c/d;p?q"
    print(url.resolve(page, "g")?)          # http://a/b/c/g
    print(url.resolve(page, "../g")?)       # http://a/b/g
    print(url.resolve(page, "/g")?)         # http://a/g
    print(url.resolve(page, "//g")?)        # http://g
    print(url.resolve(page, "?y")?)         # http://a/b/c/d;p?y
    print(url.resolve(page, "#s")?)         # http://a/b/c/d;p?q#s
    print(url.resolve(page, "../../../g")?) # http://a/g
    base := url.parse(page)?
    print(base.resolve(url.parse("g;x?y#s")?))   # http://a/b/c/g;x?y#s
    print(url.join_path("https://h/v1", ["users", "42"])?)    # https://h/v1/users/42
    print(url.join_path("https://h/v1/", ["a b/", "../c/"])?) # https://h/v1/c/
```

## Percent-encoding, forms and queries

**std.url.7** — `encode(s str) str` writes every UTF-8 byte of `s` but the unreserved characters (`A-Z`, `a-z`,
`0-9`, `-`, `.`, `_`, `~`) as `%XX` with upper-case hexadecimal digits, for one part of a URL: a path segment,
a query key or a query value. `decode(s str) str?` reads each `%XX` back, and is `none` when a `%` is not
followed by two hexadecimal digits or the bytes are not UTF-8; `+` stays `+`. *(ADR 1214)*

**std.url.8** — `encode_form(s str) str` and `decode_form(s str) str?` read and write
`application/x-www-form-urlencoded` text, the query of an HTML form: `encode_form` is `encode` with a space
written `+`, and `decode_form` is `decode` with `+` read as a space. *(ADR 1214)*

**std.url.9** — `parse_query(s str) List[(str, str)]` is the `key=value` pairs of a query, in order, each
side decoded by `decode_form`: a leading `?` is skipped, empty pieces between `&` are dropped, a piece with no
`=` has the value `""`, and a side that does not decode is kept as written. `query_map(s str) Map[str, str]`
is those pairs as a map; a key given twice keeps its first value.
`build_query(pairs List[(str, str)]) str` encodes each key and value with `encode_form` and joins the pairs
with `&`. *(ADR 1214)*

```ail rules=std.url.7,std.url.8,std.url.9
fn main() !
    print(url.encode("a b/é"))              # a%20b%2F%C3%A9
    print(url.decode("a%20b%2F") else "?")  # a b/
    print(url.decode("100%") else "bad")    # bad
    print(url.encode_form("a b&c"))         # a+b%26c
    print(url.decode_form("a+b%26c") else "?")   # a b&c
    for kv in url.parse_query("?tag=x+y&tag=z&flag")
        print("{kv.0}={kv.1}")              # tag=x y, then tag=z, then flag=
    q := url.query_map("a=1&b=2&a=3")
    print(q.get("a") else "")               # 1
    print(url.build_query([("q", "a b"), ("n", "1&2")]))   # q=a+b&n=1%262
```

## Errors and scope

**std.url.10** — A failure of `parse`, `resolve` or `join_path` is the error struct `url.UrlError`, with fields
`kind url.UrlKind`, `why str` (one sentence on the fix), `text str` (the text given) and `at int` (the 0-based
character offset of the problem). Its message is `bad URL "TEXT": WHY (at N)`. The kinds are: `BadChar`, a
space or a control character (U+0000 to U+0020, U+007F); `BadEscape`, a `%` not followed by two hexadecimal
digits; `BadScheme`, a `:` before the first `/`, `?` or `#` with no valid scheme in front of it (`1a:b`);
`BadHost`, a `[` host with no `]`, one that is not hexadecimal digits, `:` and `.`, or text other than `:PORT`
after the `]`; `BadPort`, a port that is not digits or is above 65535. Other characters, non-ASCII ones
included, are kept as written. *(ADR 1214)*

**std.url.11** — `std.url` follows RFC 3986, not the WHATWG URL standard: it does not convert host names to
punycode (IDNA), add or drop default ports, read `\` as `/`, or percent-encode what `parse` was given. An
empty query (`a?`) and an empty fragment (`a#`) are read as absent, and an empty authority (`//`) is kept only
for `file:`. *(ADR 1214)*

```ail rules=std.url.10,std.url.11
fn main() !
    bad := url.parse("http://h:99999") else err
        print(err.kind == url.UrlKind.BadPort)   # true
        print(err)      # bad URL "http://h:99999": a port is at most 65535 (at 9)
        url.Url()
    print(bad.host == "")                        # true: the fallback
    if u := url.parse("http://a b")
        print(u)
    else err
        print(err.kind)                          # BadChar
    p := url.parse("http://h:80/x")?
    print(p.port else 0)                         # 80: a default port is kept
```
