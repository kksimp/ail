#!/usr/bin/env python3
"""std.validate reference: checks the module's rules against real libraries and
writes the test programs and goldens in tests/cases/std_validate_*.{ail,out}.

Libraries (pip install idna email-validator python-stdnum phonenumbers; the
ipaddress module and the punycode codec are in Python):
  Punycode  the `punycode` codec          IDNA      idna.encode / idna.decode
  email     email_validator (strict, no DNS)   IP, CIDR  ipaddress
  Luhn      stdnum.luhn                   phone     phonenumbers (E.164)
Host names and card brands have no library: those lines come from the rules
in docs/spec/std-validate.md.

`mirror` below is a line-for-line model of std/validate/validate.ail (same
checks in the same order, same kind names). Every case where a library has an
opinion must agree with the mirror, or this script stops. Run it from the
repository root: python3 tools/validate/reference.py [--check]
"""
import ipaddress
import os
import re
import sys

import email_validator as ev
import idna
import phonenumbers as pn
import stdnum.luhn as stdluhn


class VErr(Exception):
    def __init__(self, kind):
        self.kind = kind


# ---------------------------------------------------------------- mirror
ATEXT = set("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!#$%&'*+-/=?^_`{|}~")
DIGITS = "abcdefghijklmnopqrstuvwxyz0123456789"


def adapt(delta, n, first):
    delta = delta // 700 if first else delta // 2
    delta += delta // n
    k = 0
    while delta > 455:
        delta //= 35
        k += 36
    return k + 36 * delta // (delta + 38)


def thresh(k, bias):
    t = k - bias
    return 1 if t < 1 else (26 if t > 26 else t)


def digit_val(k):
    if 48 <= k <= 57:
        return k - 22
    if 97 <= k <= 122:
        return k - 97
    if 65 <= k <= 90:
        return k - 65
    return -1


def puny_encode(s):
    cps = [ord(c) for c in s]
    out = [c for c in s if ord(c) < 128]
    b = len(out)
    h = b
    if b > 0:
        out.append("-")
    n, delta, bias = 128, 0, 72
    while h < len(cps):
        m = 1114112
        for k in cps:
            if n <= k < m:
                m = k
        delta += (m - n) * (h + 1)
        n = m
        for k in cps:
            if k < n:
                delta += 1
            if k == n:
                q, j = delta, 36
                while True:
                    t = thresh(j, bias)
                    if q < t:
                        break
                    d = t + (q - t) % (36 - t)
                    out.append(DIGITS[d])
                    q = (q - t) // (36 - t)
                    j += 36
                out.append(DIGITS[q])
                bias = adapt(delta, h + 1, h == b)
                delta = 0
                h += 1
        delta += 1
        n += 1
    return "".join(out)


def puny_decode(s):
    cut = s.rfind("-")
    out, pos = [], 0
    if cut > 0:
        for c in s[:cut]:
            if ord(c) >= 128:
                raise VErr("BadIdna")
            out.append(ord(c))
        pos = cut + 1
    n, at, bias = 128, 0, 72
    while pos < len(s):
        oldi, w, j = at, 1, 36
        while True:
            if pos >= len(s):
                raise VErr("BadIdna")
            d = digit_val(ord(s[pos]))
            if d < 0 or w > 1000000000:
                raise VErr("BadIdna")
            pos += 1
            at += d * w
            t = thresh(j, bias)
            if d < t:
                break
            w *= 36 - t
            j += 36
        ln = len(out) + 1
        bias = adapt(at - oldi, ln, oldi == 0)
        n += at // ln
        at %= ln
        if n > 1114111:
            raise VErr("BadIdna")
        out.insert(at, n)
        at += 1
    if any(0xD800 <= k <= 0xDFFF for k in out):
        raise VErr("BadIdna")
    return "".join(chr(k) for k in out)


def label_cp_ok(k):
    if k < 192 or k == 215 or k == 247:
        return False
    if 8192 <= k <= 11263 or 55296 <= k <= 63743 or 65024 <= k <= 65039:
        return False
    if 12288 <= k <= 12292 or 12296 <= k <= 12320 or 12336 <= k <= 12351:
        return False
    if k >= 65280 or 126976 <= k <= 131071 or k >= 917504:
        return False
    return True


def ldh(c):
    k = ord(c)
    return 48 <= k <= 57 or 97 <= k <= 122 or k == 45


def label_to_ascii(label):
    if label == "":
        raise VErr("BadLabel")
    if label[0] == "-" or label[-1] == "-":
        raise VErr("BadLabel")
    if all(ord(c) < 128 for c in label):
        for c in label:
            if not ldh(c):
                raise VErr("BadChar")
        if label.startswith("xn--"):
            rest = label[4:]
            u = puny_decode(rest)
            if puny_encode(u) != rest:
                raise VErr("BadIdna")
            plain = True
            for c in u:
                if ord(c) < 128:
                    continue
                plain = False
                if not label_cp_ok(ord(c)):
                    raise VErr("BadIdna")
            if plain:
                raise VErr("BadIdna")
        elif len(label) >= 4 and label[2:4] == "--":
            raise VErr("BadLabel")
        return label
    for c in label:
        if ord(c) < 128:
            if not ldh(c):
                raise VErr("BadChar")
        elif c.isupper() or not label_cp_ok(ord(c)):
            raise VErr("BadIdna")
    if len(label) >= 4 and label[2:4] == "--":
        raise VErr("BadLabel")
    return "xn--" + puny_encode(label)


def to_ascii(domain):
    if domain == "":
        raise VErr("Empty")
    dot = domain.endswith(".")
    body = domain[:-1] if dot else domain
    labels = []
    for l in body.lower().split("."):
        a = label_to_ascii(l)
        if len(a) > 63:
            raise VErr("TooLong")
        labels.append(a)
    joined = ".".join(labels)
    if len(joined) > 253:
        raise VErr("TooLong")
    return joined + "." if dot else joined


def to_unicode(domain):
    a = to_ascii(domain)
    return ".".join(puny_decode(l[4:]) if l.startswith("xn--") else l for l in a.split("."))


def local_cp_ok(k):
    if k < 161 or k in (5760, 6158, 12288, 65279):
        return False
    if 8192 <= k <= 8207 or 8232 <= k <= 8239 or 8288 <= k <= 8303:
        return False
    if 55296 <= k <= 63743 or 65520 <= k <= 65535:
        return False
    if k >= 917504 or k % 65536 >= 65534:
        return False
    return True


def email(s):
    for c in s:
        if ord(c) <= 32 or ord(c) == 127:
            raise VErr("BadChar")
    at = s.find("@")
    if at < 0:
        raise VErr("BadShape")
    local, dom = s[:at], s[at + 1:]
    if local == "" or dom == "":
        raise VErr("Empty")
    if "@" in dom:
        raise VErr("BadChar")
    utf8 = False
    for c in local:
        k = ord(c)
        if k < 128:
            if c != "." and c not in ATEXT:
                raise VErr("BadChar")
        else:
            utf8 = True
            if not local_cp_ok(k):
                raise VErr("BadChar")
    if local.startswith(".") or local.endswith(".") or ".." in local:
        raise VErr("BadShape")
    if len(local.encode()) > 64:
        raise VErr("TooLong")
    if dom.startswith(".") or dom.endswith(".") or ".." in dom:
        raise VErr("BadShape")
    ascii_ = to_ascii(dom)
    if len((local + "@" + ascii_).encode()) > 254:
        raise VErr("TooLong")
    labels = ascii_.split(".")
    if len(labels) < 2:
        raise VErr("BadDomain")
    if not ("a" <= labels[-1][-1] <= "z"):
        raise VErr("BadDomain")
    for sp in ("arpa", "invalid", "local", "localhost", "onion", "test"):
        if ascii_ == sp or ascii_.endswith("." + sp):
            raise VErr("BadDomain")
    return local, ascii_, to_unicode(ascii_), utf8


def hexval(c):
    return int(c, 16) if c in "0123456789abcdefABCDEF" else -1


def ipv4_octets(s):
    parts = s.split(".")
    if len(parts) != 4:
        raise VErr("BadAddress")
    out = []
    for p in parts:
        if p == "" or len(p) > 3 or not all("0" <= c <= "9" for c in p):
            raise VErr("BadAddress")
        if len(p) > 1 and p[0] == "0":
            raise VErr("BadAddress")
        if int(p) > 255:
            raise VErr("BadAddress")
        out.append(int(p))
    return out


def groups(s):
    return s.split(":") if s != "" else []


def group_octets(gs, tail):
    out = []
    for g in gs:
        if "." in g:
            out += tail
        else:
            v = int(g, 16)
            out += [v // 256, v % 256]
    return out


def ipv6_octets(s):
    if s.count("::") > 1 or ":::" in s:
        raise VErr("BadAddress")
    gap = "::" in s
    if gap:
        i = s.index("::")
        lg, rg = groups(s[:i]), groups(s[i + 2:])
    else:
        lg, rg = groups(s), []
    allg = lg + rg
    tail, has_tail = [], False
    for i, g in enumerate(allg):
        if "." in g:
            if i != len(allg) - 1 or (gap and not rg):
                raise VErr("BadAddress")
            tail, has_tail = ipv4_octets(g), True
            continue
        if g == "" or len(g) > 4 or any(hexval(c) < 0 for c in g):
            raise VErr("BadAddress")
    n = len(allg) + (1 if has_tail else 0)
    if gap and n > 7:
        raise VErr("BadAddress")
    if not gap and n != 8:
        raise VErr("BadAddress")
    lb = group_octets(lg, tail)
    if not gap:
        return lb
    rb = group_octets(rg, tail)
    return lb + [0] * (16 - len(lb) - len(rb)) + rb


def ip(s):
    return ipv6_octets(s) if ":" in s else ipv4_octets(s)


def addr_str(o):
    if len(o) == 4:
        return ".".join(map(str, o))
    w = [o[2 * i] * 256 + o[2 * i + 1] for i in range(8)]
    best, bl, i = -1, 0, 0
    while i < 8:
        if w[i] != 0:
            i += 1
            continue
        j = i
        while j < 8 and w[j] == 0:
            j += 1
        if j - i > bl:
            best, bl = i, j - i
        i = j
    hx = lambda ws: ":".join("%x" % x for x in ws)
    if bl < 2:
        return hx(w)
    return hx(w[:best]) + "::" + hx(w[best + bl:])


def keep_mask(prefix, i):
    keep = max(0, min(8, prefix - 8 * i))
    return (255 << (8 - keep)) & 255


def cidr(s, strict=True):
    head, pfx, has = s, "", False
    if "/" in s:
        head, pfx, has = s.split("/", 1)[0], s.split("/", 1)[1], True
    a = ip(head)
    bits = 8 * len(a)
    prefix = bits
    if has:
        if pfx == "" or len(pfx) > 3 or not all("0" <= c <= "9" for c in pfx):
            raise VErr("BadPrefix")
        prefix = int(pfx)
        if prefix > bits:
            raise VErr("BadPrefix")
    net, host = [], False
    for i in range(len(a)):
        m = keep_mask(prefix, i)
        net.append(a[i] & m)
        if a[i] & (255 - m):
            host = True
    if host and strict:
        raise VErr("BadPrefix")
    return net, prefix


def net_contains(n, a):
    if (len(a) == 16) != (len(n[0]) == 16):
        return False
    return all((a[i] & keep_mask(n[1], i)) == n[0][i] for i in range(len(a)))


def net_last(n):
    return [n[0][i] | (255 - keep_mask(n[1], i)) for i in range(len(n[0]))]


def hostname(s):
    body = s[:-1] if s.endswith(".") else s
    if body == "":
        raise VErr("Empty")
    if len(body) > 253:
        raise VErr("TooLong")
    labels = body.split(".")
    for l in labels:
        if l == "" or len(l) > 63:
            raise VErr("BadLabel")
        if l[0] == "-" or l[-1] == "-":
            raise VErr("BadLabel")
        for c in l:
            if not (c.isascii() and (c.isalnum() or c == "-")):
                raise VErr("BadChar")
    if all("0" <= c <= "9" for c in labels[-1]):
        raise VErr("BadLabel")
    return body.lower()


def luhn_sum(s, double_first):
    total, dbl = 0, double_first
    for c in reversed(s):
        d = ord(c) - 48
        if dbl:
            d *= 2
            if d > 9:
                d -= 9
        total += d
        dbl = not dbl
    return total


def luhn(s):
    return len(s) >= 2 and all("0" <= c <= "9" for c in s) and luhn_sum(s, False) % 10 == 0


def luhn_digit(p):
    if p == "" or not all("0" <= c <= "9" for c in p):
        raise VErr("BadNumber")
    return (10 - luhn_sum(p, True) % 10) % 10


def brand_of(n):
    ln = len(n)
    p2, p3, p4 = int(n[:2]), int(n[:3]), int(n[:4])
    if n[0] == "4" and ln in (13, 16, 19):
        return "Visa"
    if (51 <= p2 <= 55 or 2221 <= p4 <= 2720) and ln == 16:
        return "Mastercard"
    if p2 in (34, 37) and ln == 15:
        return "Amex"
    if (p4 == 6011 or p2 == 65 or 644 <= p3 <= 649) and 16 <= ln <= 19:
        return "Discover"
    if (300 <= p3 <= 305 or p3 == 309 or p2 in (36, 38, 39)) and 14 <= ln <= 19:
        return "Diners"
    if 3528 <= p4 <= 3589 and 16 <= ln <= 19:
        return "Jcb"
    if p2 == 62 and 16 <= ln <= 19:
        return "UnionPay"
    return "Unknown"


def card(s):
    digits = []
    for c in s:
        if c in " -":
            continue
        if not "0" <= c <= "9":
            raise VErr("BadChar")
        digits.append(c)
    n = "".join(digits)
    if len(n) < 12 or len(n) > 19:
        raise VErr("BadNumber")
    if not luhn(n):
        raise VErr("BadNumber")
    return brand_of(n), n, n[-4:]


CODES = {str(c) for c in pn.COUNTRY_CODE_TO_REGION_CODE}


def phone(s, default_cc=""):
    t = s.strip()
    intl = t.startswith("+")
    body = t[1:] if intl else t
    digits = []
    for c in body:
        if "0" <= c <= "9":
            digits.append(c)
        elif c not in " -.()":
            raise VErr("BadPhone")
    ds = "".join(digits)
    if not intl and ds.startswith("00"):
        intl, ds = True, ds[2:]
    if not intl:
        if default_cc == "":
            raise VErr("BadPhone")
        if default_cc == "1":
            if len(ds) == 11 and ds.startswith("1"):
                ds = ds[1:]
        elif ds.startswith("0"):
            ds = ds[1:]
        ds = default_cc + ds
    if ds == "" or ds.startswith("0") or len(ds) > 15:
        raise VErr("BadPhone")
    code = ""
    for k in (1, 2, 3):
        if len(ds) >= k and ds[:k] in CODES:
            code = ds[:k]
            break
    if code == "":
        raise VErr("BadPhone")
    rest = ds[len(code):]
    if len(rest) < 4:
        raise VErr("BadPhone")
    if code == "1" and len(rest) != 10:
        raise VErr("BadPhone")
    return code, rest


# ---------------------------------------------------------------- libraries
def lib_email(s):
    try:
        r = ev.validate_email(s, check_deliverability=False, strict=True)
        return (r.local_part, r.ascii_domain, r.domain, r.smtputf8)
    except (ev.EmailNotValidError, ValueError):
        return None


def lib_to_ascii(d):
    try:
        return idna.encode(d, uts46=True).decode()
    except (idna.IDNAError, UnicodeError):
        return None


def lib_to_unicode(d):
    try:
        return idna.decode(d)
    except (idna.IDNAError, UnicodeError):
        return None


def lib_ip(s):
    try:
        a = ipaddress.ip_address(s)
    except ValueError:
        return None
    if getattr(a, "scope_id", None):
        return None
    return a


# ---------------------------------------------------------------- cases
EMAILS = [
    "a@example.com", "User.Name+tag@Example.COM", "user@bücher.de", "jörg@müller.de", "日本語@日本語.jp",
    "a@xn--bcher-kva.de", "a@xn--bcher-kva.DE", "A@X.COM", "a+b@x.io", "a@123.com", "a@x.c0m", "a@x.example",
    "😀@x.com", "tést@exämple.org", "!#$%&'*+-/=?^_`{|}~@x.com", "a@sub.domain.example.org",
    "a@b", "a@localhost", "a@example.test", "a@x.invalid", "a@x.local", "a@x.onion", "a@x.arpa", "a@sub.test",
    "a..b@x.com", ".a@x.com", "a.@x.com", "a b@x.com", "\"a b\"@x.com", "a@[1.2.3.4]", "a@1.2.3.4",
    "a@-x.com", "a@x-.com", "a@x..com", "Name <a@x.com>", "a@b.co.", "@x.com", "a@", "a@ex_ample.com",
    "a@x.123", "x@y@z.com", "a@x.co m", "a@😀.com", "no-at-sign", "",
    "a" * 64 + "@x.com", "a" * 65 + "@x.com", "a@" + "a" * 63 + ".com", "a@" + "a" * 64 + ".com",
    "a@x.com.", "a@.com", "a@xn--p1ai", "ab@ÄÖ.com",
]
TO_ASCII = [
    "bücher.de", "Example.COM", "日本語.jp", "münchen.de", "straße.de", "фото.рф", "xn--mller-kva.de", "a.b.c.d.example",
    "BÜCHER.de", "a_b.com", "-a.com", "a-.com", "ab--cd.com", "xn--x.de", "xn--a.com", "😀.com", "a..b", "ex ample.com",
    "ΕΛΛΗΝΙΚΑ.gr", "a.b.", "", "a" * 64 + ".com", "bü_x.de",
]
PUNY = ["bücher", "münchen", "日本語", "ü", "abc", "a-b", "-", "ЯЯЯ", "한국어", "mañana", "αβγ", "a😀b"]
PUNY_BAD = ["!", "bcher-kv", "9999999999999", "-x", "ü"]
IPS = [
    "1.2.3.4", "0.0.0.0", "255.255.255.255", "256.1.1.1", "1.2.3", "1.2.3.4.5", "01.2.3.4", "1..3.4", "a.b.c.d", "",
    "::", "::1", "1::", "1:2:3:4:5:6:7:8", "1:2:3:4:5:6:7::", "::2:3:4:5:6:7:8", "1:2:3:4:5:6:7:8:9", "1::2::3",
    "1:::2", "2001:db8::1", "2001:DB8:0:0:1::1", "2001:db8:0:0:0:0:0:1", "1:0:0:2:0:0:0:3", "1:0:0:2:0:0:3:4",
    "0:0:0:0:0:0:0:0", "::1.2.3.4", "1:2:3:4:5:6:1.2.3.4", "1:2:3:4:5:6:7:1.2.3.4", "1.2.3.4::", "12345::1",
    "g::1", "1:2:3:4:5:6:7", ":1:2:3:4:5:6:7", "1::1.2.3.4", "1.2.3.4:1", "fe80::1%eth0", "1: 2::",
]
CIDRS = [
    "10.0.0.0/8", "10.0.0.1/8", "192.168.1.0/24", "1.2.3.4", "1.2.3.4/32", "1.2.3.4/33", "1.2.3.4/", "::/0",
    "2001:db8::/32", "2001:db8::1/32", "2001:db8::/129", "fe80::/10", "0.0.0.0/0", "10.0.0.0/-1", "10.0.0.0/8/8",
    "172.16.5.4/12", "::1/128", "2001:db8:abcd::/48",
]
CONTAINS = [("10.0.0.0/8", "10.9.8.7"), ("10.0.0.0/8", "11.0.0.1"), ("192.168.1.0/24", "192.168.1.255"),
            ("192.168.1.0/24", "192.168.2.0"), ("2001:db8::/32", "2001:db8:ffff::1"), ("2001:db8::/32", "2001:db9::1"),
            ("10.0.0.0/8", "::1"), ("0.0.0.0/0", "8.8.8.8")]
PHONES = [
    ("+1 415-555-2671", ""), ("+14155552671", ""), ("+44 20 7946 0958", ""), ("+81 3-1234-5678", ""), ("+49 30 123456", ""),
    ("+33 1 23 45 67 89", ""), ("+61 2 1234 5678", ""), ("+86 10 1234 5678", ""), ("+91 98765 43210", ""),
    ("+55 11 91234-5678", ""), ("+7 495 123-45-67", ""), ("+353 1 234 5678", ""), ("+44 20-7946.0958", ""),
    ("+1 (415) 555-2671 ", ""), ("(415) 555-2671", "1"), ("1-415-555-2671", "1"), ("020 7946 0958", "44"),
    ("030 123456", "49"), ("+999 123456", ""), ("+1 415 555", ""), ("+1234567890123456", ""), ("+0 1234567", ""),
    ("415 555 2671", ""), ("", ""), ("+", ""), ("+44", ""),
]
PHONES_OURS = [("0044 20 7946 0958", ""), ("00 1 415 555 2671", ""), ("+44 20 7946 095a", ""), ("+1 415-555-2671 ext 5", "")]
IS_E164 = ["+14155552671", "+1 415 555 2671", "14155552671", "+44", "+4420794609580"]
HOSTS = ["localhost", "example.com", "Example.COM", "a-b.example.org", "a.b.", "x", "my_host.com", "-a.com", "a-.com",
         "a..b", "", ".", "a.123", "123.456.789.012", "1.2.3.4", "a" * 63 + ".com", "a" * 64 + ".com", "bücher.de",
         "a b.com", "a.b-.c", ("a" * 62 + ".") * 4 + "com"]
LUHNS = ["79927398713", "4111111111111111", "4111111111111112", "0", "00", "", "12a4", "49927398716", "5500000000000004",
         "378282246310005", "6011111111111117", "3530111333300000", "1234567812345670", " 4111"]
LUHN_PAYLOADS = ["7992739871", "411111111111111", "550000000000000", "37828224631000", "0", "", "12x"]
CARDS = ["4111111111111111", "4111 1111 1111 1111", "4111-1111-1111-1111", "5555555555554444", "2223003122003222",
         "378282246310005", "6011111111111117", "3056930009020004", "3530111333300000", "6200000000000005",
         "4111111111111112", "4111", "41111111111111111111", "4111 1111 1111 111a", "", "4222222222222", "9999999999999995"]


def esc(s):
    return s.replace("\\", "\\\\").replace('"', '\\"').replace("{", "\\{")


def q(s):
    return '"' + esc(s) + '"'


def b(v):
    return "true" if v else "false"


def lines_for_email():
    helper = '''fn try_email(s str)
    e := validate.email(s) else err
        print("[{s}] {err.kind.name()}")
        return
    print("[{s}] {e.local} {e.domain} {e.unicode_domain} {e.smtputf8} {e}")

'''
    calls, out = [], []
    for s in EMAILS:
        calls.append(f"    try_email({q(s)})")
        try:
            m = email(s)
        except VErr as e:
            m, kind = None, e.kind
        lib = lib_email(s)
        if (m is None) != (lib is None) or (m is not None and m != lib):
            sys.exit(f"email mirror and library differ on {s!r}: {m} vs {lib}")
        out.append(f"[{s}] {kind}" if m is None else f"[{s}] {m[0]} {m[1]} {m[2]} {b(m[3])} {m[0]}@{m[1]}")
    calls.append('    print(validate.is_email("a@example.com"))')
    out.append("true")
    calls.append('    print(validate.is_email("a@b"))')
    out.append("false")
    return "std.validate.4 to std.validate.8: email addresses", helper, calls, out


def lines_for_idna():
    helper = '''fn try_ascii(s str)
    a := validate.to_ascii(s) else err
        print("[{s}] {err.kind.name()}")
        return
    u := validate.to_unicode(a) else err
        print("[{s}] {a} unicode: {err.kind.name()}")
        return
    print("[{s}] {a} {u}")

fn try_decode(s str)
    d := validate.punycode_decode(s) else err
        print("[{s}] {err.kind.name()}")
        return
    print("[{s}] {d}")

'''
    calls, out = [], []
    for s in TO_ASCII:
        calls.append(f"    try_ascii({q(s)})")
        try:
            m = to_ascii(s)
            mu = to_unicode(m)
            kind = None
        except VErr as e:
            m, kind = None, e.kind
        lib = lib_to_ascii(s)
        if s == "ǆ.com":
            continue
        if (m is None) != (lib is None) or (m is not None and m != lib):
            sys.exit(f"to_ascii mirror and library differ on {s!r}: {m} vs {lib}")
        if m is None:
            out.append(f"[{s}] {kind}")
        else:
            lu = lib_to_unicode(m)
            if lu != mu and not (m.endswith(".") and lu is None):
                sys.exit(f"to_unicode mirror and library differ on {m!r}: {mu} vs {lu}")
            out.append(f"[{s}] {m} {mu}")
    for s in PUNY:
        calls.append(f"    print(validate.punycode_encode({q(s)}))")
        e = s.encode("punycode").decode()
        assert puny_encode(s) == e, (s, puny_encode(s), e)
        assert puny_decode(e) == s
        out.append(e)
        calls.append(f"    try_decode({q(e)})")
        out.append(f"[{e}] {s}")
    for s in PUNY_BAD:
        try:
            r = puny_decode(s)
            sys.exit(f"expected {s!r} to be bad Punycode, got {r!r}")
        except VErr as e:
            calls.append(f"    try_decode({q(s)})")
            out.append(f"[{s}] {e.kind}")
        try:
            s.encode("ascii").decode("punycode")
            if s != "9999999999999":
                sys.exit(f"python decodes {s!r}")
        except (UnicodeError, ValueError):
            pass
    calls.append('    print(validate.is_domain("bücher.de"))')
    out.append("true")
    calls.append('    print(validate.is_domain("a..b"))')
    out.append("false")
    return "std.validate.2, std.validate.3: Punycode and IDNA", helper, calls, out


def lines_for_ip():
    helper = '''fn try_ip(s str)
    a := validate.ip(s) else err
        print("[{s}] {err.kind.name()}")
        return
    print("[{s}] {a} v6={a.v6} {a.octets.len()}")

fn try_cidr(s str)
    n := validate.cidr(s) else err
        print("[{s}] strict {err.kind.name()}")
        l := validate.cidr(s, strict: false) else err2
            print("[{s}] loose {err2.kind.name()}")
            return
        print("[{s}] loose {l} last {l.last()}")
        return
    print("[{s}] strict {n} last {n.last()}")

fn try_in(block str, host str)
    n := validate.cidr(block) else return
    a := validate.ip(host) else return
    print("{block} has {host}: {n.contains(a)}")

fn try_v6(s str)
    a := validate.ipv6(s) else err
        print("[{s}] v6 {err.kind.name()}")
        return
    print("[{s}] v6 {a}")

fn try_v4(s str)
    a := validate.ipv4(s) else err
        print("[{s}] v4 {err.kind.name()}")
        return
    print("[{s}] v4 {a}")

'''
    calls, out = [], []
    for s in IPS:
        calls.append(f"    try_ip({q(s)})")
        try:
            o = ip(s)
            m = addr_str(o)
            kind = None
        except VErr as e:
            m, kind = None, e.kind
        lib = lib_ip(s)
        ref = None if lib is None else (lib.compressed if lib.version == 6 else str(lib))
        if lib is not None and lib.version == 6 and lib.ipv4_mapped is not None:
            ref = m  # Python 3.13 writes ::ffff:a.b.c.d; this module writes hex groups
        if m != ref:
            sys.exit(f"ip mirror and library differ on {s!r}: {m} vs {ref}")
        out.append(f"[{s}] {kind}" if m is None else f"[{s}] {m} v6={b(len(o) == 16)} {len(o)}")
    for s in CIDRS:
        calls.append(f"    try_cidr({q(s)})")
        res = {}
        for strict in (True, False):
            try:
                n = cidr(s, strict)
                res[strict] = (f"{addr_str(n[0])}/{n[1]}", addr_str(net_last(n)))
            except VErr as e:
                res[strict] = e.kind
            try:
                lib = ipaddress.ip_network(s, strict=strict)
                ref = (f"{lib.network_address.compressed}/{lib.prefixlen}", lib.broadcast_address.compressed
                       if lib.version == 4 else lib[-1].compressed)
            except ValueError:
                ref = None
            if (ref is None) != isinstance(res[strict], str) or (ref is not None and res[strict] != ref):
                sys.exit(f"cidr mirror and library differ on {s!r} strict={strict}: {res[strict]} vs {ref}")
        if isinstance(res[True], str):
            first = f"[{s}] strict {res[True]}"
            out.append(first)
            if isinstance(res[False], str):
                out.append(f"[{s}] loose {res[False]}")
            else:
                out.append(f"[{s}] loose {res[False][0]} last {res[False][1]}")
        else:
            out.append(f"[{s}] strict {res[True][0]} last {res[True][1]}")
    for blk, h in CONTAINS:
        calls.append(f"    try_in({q(blk)}, {q(h)})")
        n, a = cidr(blk), ip(h)
        lib = ipaddress.ip_address(h) in ipaddress.ip_network(blk)
        if net_contains(n, a) != lib and (len(a) == len(n[0])):
            sys.exit(f"contains differs {blk} {h}")
        out.append(f"{blk} has {h}: {b(net_contains(n, a))}")
    calls.append('    print(validate.is_ip("::1"))')
    out.append("true")
    calls.append('    print(validate.is_ip("1.2.3"))')
    out.append("false")
    calls.append('    try_v4("::1")')
    out.append("[::1] v4 BadAddress")
    calls.append('    try_v4("1.2.3.4")')
    out.append("[1.2.3.4] v4 1.2.3.4")
    for v in ("2001:db8::1", "1.2.3.4"):
        calls.append(f'    try_v6("{v}")')
        try:
            out.append(f"[{v}] v6 {addr_str(ipv6_octets(v))}")
        except VErr as e:
            out.append(f"[{v}] v6 {e.kind}")
    return "std.validate.9 to std.validate.11: IP addresses and CIDR blocks", helper, calls, out


def lines_for_phone():
    helper = '''fn try_phone(s str, cc str)
    p := validate.phone(s, default_cc: cc) else err
        print("[{s}] {err.kind.name()}")
        return
    print("[{s}] {p} cc={p.country_code} national={p.national}")

'''
    calls, out = [], []
    region = {"1": "US", "44": "GB", "49": "DE"}
    for s, cc in PHONES + PHONES_OURS:
        calls.append(f"    try_phone({q(s)}, {q(cc)})")
        try:
            c, n = phone(s, cc)
            m, kind = "+" + c + n, None
        except VErr as e:
            m, kind = None, e.kind
        if (s, cc) not in PHONES_OURS:
            try:
                p = pn.parse(s, region.get(cc))
                lib = pn.format_number(p, pn.PhoneNumberFormat.E164) if pn.is_possible_number(p) else None
            except pn.NumberParseException:
                lib = None
            if m != lib:
                sys.exit(f"phone mirror and library differ on {s!r} {cc!r}: {m} vs {lib}")
        out.append(f"[{s}] {kind}" if m is None else f"[{s}] {m} cc={c} national={n}")
    for s in IS_E164:
        calls.append(f"    print(validate.is_e164({q(s)}))")
        try:
            phone(s)
            ok = s.startswith("+") and s[1:].isdigit()
        except VErr:
            ok = False
        out.append(b(ok))
    return "std.validate.15, std.validate.16: E.164 phone numbers", helper, calls, out


def lines_for_misc():
    helper = '''fn try_host(s str)
    h := validate.hostname(s) else err
        print("[{s}] {err.kind.name()}")
        return
    print("[{s}] {h}")

fn try_card(s str)
    c := validate.card(s) else err
        print("[{s}] {err.kind.name()}")
        return
    print("[{s}] {c.brand.name()} {c.number} {c.last4}")

fn try_digit(s str)
    d := validate.luhn_digit(s) else err
        print("[{s}] {err.kind.name()}")
        return
    print("[{s}] {d}")

'''
    calls, out = [], []
    hostre = re.compile(r"^(?:[A-Za-z0-9](?:[A-Za-z0-9-]{0,61}[A-Za-z0-9])?\.)*[A-Za-z0-9](?:[A-Za-z0-9-]{0,61}[A-Za-z0-9])?\.?$")
    for s in HOSTS:
        calls.append(f"    try_host({q(s)})")
        try:
            m, kind = hostname(s), None
        except VErr as e:
            m, kind = None, e.kind
        body = s[:-1] if s.endswith(".") else s
        ref = bool(hostre.match(s)) and len(body) <= 253 and not body.rsplit(".", 1)[-1].isdigit()
        if ref != (m is not None):
            sys.exit(f"hostname mirror and regex differ on {s!r}")
        out.append(f"[{s}] {kind}" if m is None else f"[{s}] {m}")
    for s in ("localhost", "my_host.com", "a.b.", "1.2.3.4"):
        calls.append(f"    print(validate.is_hostname({q(s)}))")
        out.append(b(s in ("localhost", "a.b.")))
    for s in LUHNS:
        calls.append(f"    print(validate.luhn({q(s)}))")
        m = luhn(s)
        ref = len(s) >= 2 and s.isascii() and s.isdigit() and stdluhn.is_valid(s)
        if m != ref:
            sys.exit(f"luhn differs on {s!r}")
        out.append(b(m))
    for s in LUHN_PAYLOADS:
        calls.append(f"    try_digit({q(s)})")
        try:
            d = luhn_digit(s)
            if d != int(stdluhn.calc_check_digit(s)):
                sys.exit(f"luhn_digit differs on {s!r}")
            out.append(f"[{s}] {d}")
        except VErr as e:
            out.append(f"[{s}] {e.kind}")
    for s in CARDS:
        calls.append(f"    try_card({q(s)})")
        try:
            br, n, l4 = card(s)
            out.append(f"[{s}] {br} {n} {l4}")
        except VErr as e:
            out.append(f"[{s}] {e.kind}")
    return "std.validate.12 to std.validate.14: host names, Luhn and card numbers", helper, calls, out


FILES = {
    "std_validate_email": lines_for_email,
    "std_validate_idna": lines_for_idna,
    "std_validate_ip": lines_for_ip,
    "std_validate_phone": lines_for_phone,
    "std_validate_misc": lines_for_misc,
}


def main():
    check = "--check" in sys.argv
    root = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "tests", "cases")
    bad = 0
    for name, fn in FILES.items():
        title, helper, calls, out = fn()
        prog = f"## {title}.\nuse std.validate\n\n{helper}fn main()\n" + "\n".join(calls) + "\n"
        outtxt = "\n".join(out) + "\n"
        for ext, text in (("ail", prog), ("out", outtxt)):
            path = os.path.join(root, f"{name}.{ext}")
            if check:
                if not os.path.exists(path) or open(path, encoding="utf-8").read() != text:
                    print("differs:", path)
                    bad = 1
            else:
                with open(path, "w", encoding="utf-8") as f:
                    f.write(text)
    sys.exit(bad)


if __name__ == "__main__":
    main()
