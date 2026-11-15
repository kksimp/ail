#!/usr/bin/env python3
"""Reference oracle for the stock tasks (ADR 0916): `ref.py ARGS... < events` -> stdout, stderr lines, exit code.
VARIANT is "base" (spec.md), "loc" (stock-locations: bins, `move`, `where`) or "dec" (stock-decimal: quantities in
thousandths). Each task's ref.py is this file with VARIANT set; keep them identical otherwise."""
import re, sys
VARIANT = "loc"
SKU = r"[A-Z0-9-]+"
LOC = r"@[a-z0-9-]+"
INT = r"[0-9]{1,5}"
QTY = r"[0-9]{1,5}(\.[0-9]{1,3})?" if VARIANT == "dec" else INT

def qty_value(s):  # thousandths in the dec variant, whole units otherwise
    if VARIANT != "dec": return int(s)
    whole, _, frac = s.partition(".")
    return int(whole) * 1000 + int((frac + "000")[:3])

def money(c):
    return "%s%d.%02d" % ("-" if c < 0 else "", abs(c) // 100, abs(c) % 100)

def show_qty(q):
    return "%d.%03d" % (q // 1000, q % 1000) if VARIANT == "dec" else str(q)

USAGE = "usage: stock report|low N|total" + ("|where SKU" if VARIANT == "loc" else "")

def parse(line):
    """An event (kind, sku, qty, price or source location, location), or None for a bad line."""
    at = "(?: (%s))?" % LOC if VARIANT == "loc" else "()"
    m = re.fullmatch(r"receive (%s) (%s) (%s)%s" % (SKU, QTY, INT, at), line)
    if m:
        g = [x for x in m.groups()]
        sku, q, price, loc = g[0], g[1], g[-2] if VARIANT == "dec" else g[2], g[-1]
        q = qty_value(q)
        return ("receive", sku, q, int(price), loc or "@main") if q > 0 else None
    m = re.fullmatch(r"(ship|count) (%s) (%s)%s" % (SKU, QTY, at), line)
    if m:
        q = qty_value(m[3]); loc = m.groups()[-1] or "@main"
        return None if (m[1] == "ship" and q == 0) else (m[1], m[2], q, None, loc)
    if VARIANT == "loc":
        m = re.fullmatch(r"move (%s) (%s) (%s) (%s)" % (SKU, QTY, LOC, LOC), line)
        if m and int(m[2]) > 0: return ("move", m[1], int(m[2]), m[3], m[4])
    return None

def main(argv):
    n = None
    if len(argv) == 1 and argv[0] in ("report", "total"): pass
    elif len(argv) == 2 and argv[0] == "low" and re.fullmatch(QTY, argv[1]): n = qty_value(argv[1])
    elif VARIANT == "loc" and len(argv) == 2 and argv[0] == "where" and re.fullmatch(SKU, argv[1]): pass
    else:
        sys.stderr.write(USAGE + "\n"); return 1
    items, bad = {}, 0  # sku -> {"cost": cents, "at": {loc: qty}}
    lines = sys.stdin.buffer.read().decode("utf-8", "replace").split("\n")
    if lines and lines[-1] == "": lines.pop()
    for k, line in enumerate(lines, 1):
        if line == "" or line.startswith("#"): continue
        e = parse(line); why = None
        if e is None: why = "bad line"
        else:
            kind, sku = e[0], e[1]; it = items.get(sku)
            if kind != "receive" and it is None: why = "unknown " + sku
            else:
                if it is None: it = items[sku] = {"cost": 0, "at": {}}
                total = sum(it["at"].values())
                if kind == "receive":
                    _, _, q, price, loc = e
                    it["cost"] += q * price // 1000 if VARIANT == "dec" else q * price
                    it["at"][loc] = it["at"].get(loc, 0) + q
                elif kind == "ship":
                    _, _, q, _, loc = e
                    if it["at"].get(loc, 0) < q: why = "short " + sku
                    else:
                        it["cost"] -= it["cost"] * q // total
                        it["at"][loc] -= q
                elif kind == "count":
                    _, _, q, _, loc = e
                    new = total - it["at"].get(loc, 0) + q
                    if total > 0: it["cost"] = it["cost"] * new // total
                    it["at"][loc] = q
                else:
                    _, _, q, src, dst = e
                    if it["at"].get(src, 0) < q: why = "short " + sku
                    else:
                        it["at"][src] -= q; it["at"][dst] = it["at"].get(dst, 0) + q
        if why:
            sys.stderr.write("error: line %d: %s\n" % (k, why)); bad += 1
    key = lambda s: s.encode()
    out = []
    if argv[0] == "report":
        out = ["%s\t%s\t%s" % (s, show_qty(sum(items[s]["at"].values())), money(items[s]["cost"])) for s in sorted(items, key=key)]
    elif argv[0] == "total":
        out = [money(sum(it["cost"] for it in items.values()))]
    elif argv[0] == "low":
        out = [s for s in sorted(items, key=key) if sum(items[s]["at"].values()) < n]
    else:
        it = items.get(argv[1])
        if it is None:
            sys.stderr.write("error: unknown %s\n" % argv[1]); return 4
        out = ["%s\t%s" % (l, show_qty(q)) for l, q in sorted(it["at"].items(), key=lambda p: p[0].encode()) if q > 0]
    sys.stdout.write("".join(l + "\n" for l in out))
    return 2 if bad else 0

if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
