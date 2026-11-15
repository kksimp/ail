#!/usr/bin/env python3
"""Reference oracle for the till tasks (ADR 0916): `ref.py ARGS... < journal` -> stdout, stderr lines, exit code.
VARIANT is "base" (spec.md), "ret" (till-returns: the `return` entry) or "ren" (till-rename: customers are accounts).
Each task's ref.py is this file with VARIANT set; keep them identical otherwise."""
import re, sys
VARIANT = "base"
CUST = "account" if VARIANT == "ren" else "customer"
SKU = r"[A-Z][A-Z0-9-]{0,11}"
CAT = r"[a-z]{1,12}"
ID = r"[a-z][a-z0-9]{0,11}"
ORDER = r"o[0-9]{1,6}"
PO = r"p[0-9]{1,6}"
CARD = r"g[0-9]{1,6}"
CODE = r"[A-Z][A-Z0-9]{2,9}"
TIER = r"basic|silver|gold"
QTY = r"[0-9]{1,5}"
CENTS = r"[0-9]{1,7}"
PCT = r"[0-9]{1,2}"
USES = r"[0-9]{1,3}"
TIER_PCT = {"basic": 0, "silver": 5, "gold": 10}
USAGE = "usage: till orders|invoice ORDER|stock|low N|%ss|history ID|sales|margin|purchases|gifts|promos" % CUST

def percent(a, p): return (a * p + 50) // 100

def money(c): return "%s%d.%02d" % ("-" if c < 0 else "", abs(c) // 100, abs(c) % 100)

def parse(line):
    """A tuple whose first element is the entry kind, or None for a bad line."""
    f = lambda pat: re.fullmatch(pat, line)
    m = f(r"item (%s) (%s) (%s)" % (SKU, CAT, CENTS))
    if m: return ("item", m[1], m[2], int(m[3]))
    m = f(r"tax (%s) (%s)" % (CAT, PCT))
    if m: return ("tax", m[1], int(m[2]))
    m = f(r"(restock|adjust) (%s) (%s)" % (SKU, QTY))
    if m and (m[1] == "adjust" or int(m[3]) >= 1): return (m[1], m[2], int(m[3]))
    m = f(r"%s (%s) (%s)" % (CUST, ID, TIER))
    if m: return ("customer", m[1], m[2])
    m = f(r"promo (%s) (%s) (%s) (%s)" % (CODE, CAT, PCT, USES))
    if m and int(m[4]) >= 1: return ("promo", m[1], m[2], int(m[3]), int(m[4]))
    m = f(r"order (%s) (%s) (\S+)" % (ORDER, ID))
    if m:
        items = []
        for part in m[3].split(","):
            p = re.fullmatch(r"(%s):(%s)" % (SKU, QTY), part)
            if not p or int(p[2]) < 1 or p[1] in [s for s, _ in items]: return None
            items.append((p[1], int(p[2])))
        return ("order", m[1], m[2], items)
    m = f(r"buy (%s) (\S+)" % PO)
    if m:
        buys = []
        for part in m[2].split(","):
            p = re.fullmatch(r"(%s):(%s)@(%s)" % (SKU, QTY, CENTS), part)
            if not p or int(p[2]) < 1 or p[1] in [b[0] for b in buys]: return None
            buys.append((p[1], int(p[2]), int(p[3])))
        return ("buy", m[1], buys)
    m = f(r"receive (%s)" % PO)
    if m: return ("receive", m[1])
    m = f(r"gift (%s) (%s)" % (CARD, CENTS))
    if m: return ("gift", m[1], int(m[2])) if int(m[2]) >= 1 else None
    m = f(r"apply (%s) (%s)" % (ORDER, CODE))
    if m: return ("apply", m[1], m[2])
    m = f(r"pay (%s) (%s|credit|%s)" % (ORDER, CENTS, CARD))
    if m:
        if m[2] == "credit": return ("pay", m[1], None)
        if m[2].startswith("g"): return ("pay", m[1], m[2])
        return ("pay", m[1], int(m[2])) if int(m[2]) >= 1 else None
    m = f(r"(ship|cancel) (%s)" % ORDER)
    if m: return (m[1], m[2])
    if VARIANT == "ret":
        m = f(r"return (%s) (%s) (%s)" % (ORDER, SKU, QTY))
        if m and int(m[3]) >= 1: return ("return", m[1], m[2], int(m[3]))
    return None

def price_line(l, promo):
    gross = l["price"] * l["qty"]
    tier = percent(gross, l["tier"])
    pro = percent(gross - tier, promo["pct"]) if promo and promo["cat"] == l["cat"] else 0
    net = gross - tier - pro
    tax = percent(net, l["rate"])
    l.update(gross=gross, discount=tier + pro, net=net, tax=tax, total=net + tax)

def reprice(o, promos):
    promo = promos[o["promo"]] if o["promo"] else None
    for l in o["lines"]: price_line(l, promo)
    o["total"] = sum(l["total"] for l in o["lines"])

def main(argv):
    ok = (len(argv) == 1 and argv[0] in ("orders", "stock", CUST + "s", "sales", "margin", "purchases", "gifts", "promos")) or (len(argv) == 2 and (
        (argv[0] == "invoice" and re.fullmatch(ORDER, argv[1])) or (argv[0] == "low" and re.fullmatch(QTY, argv[1]))
        or (argv[0] == "history" and re.fullmatch(ID, argv[1]))))
    if not ok:
        sys.stderr.write(USAGE + "\n"); return 1
    items, rates, custs, promos, orders, pos, gifts = {}, {}, {}, {}, {}, {}, {}
    bad = 0
    lines = sys.stdin.buffer.read().decode("utf-8", "replace").split("\n")
    if lines and lines[-1] == "": lines.pop()
    for k, line in enumerate(lines, 1):
        if line == "" or line.startswith("#"): continue
        e = parse(line); why = None
        kind = e[0] if e else None
        if e is None: why = "bad line"
        elif kind == "item":
            _, sku, cat, cents = e
            if sku in items and items[sku]["cat"] != cat: why = "category changed " + sku
            elif sku in items: items[sku]["price"] = cents
            else: items[sku] = {"cat": cat, "price": cents, "onhand": 0, "reserved": 0, "value": 0}
        elif kind == "tax": rates[e[1]] = e[2]
        elif kind == "restock":
            if e[1] not in items: why = "unknown item " + e[1]
            else: items[e[1]]["onhand"] += e[2]
        elif kind == "adjust":
            it = items.get(e[1])
            if it is None: why = "unknown item " + e[1]
            elif e[2] < it["reserved"]: why = "below reserved " + e[1]
            else:
                if it["onhand"] > 0: it["value"] = it["value"] * e[2] // it["onhand"]
                it["onhand"] = e[2]
        elif kind == "buy":
            if e[1] in pos: why = "duplicate purchase " + e[1]
            else: why = next(("unknown item " + b[0] for b in e[2] if b[0] not in items), None)
            if not why: pos[e[1]] = {"status": "ordered", "buys": e[2]}
        elif kind == "receive":
            po = pos.get(e[1])
            if po is None: why = "unknown purchase " + e[1]
            elif po["status"] != "ordered": why = "purchase %s is %s" % (e[1], po["status"])
            else:
                for sku, q, c in po["buys"]: items[sku]["onhand"] += q; items[sku]["value"] += q * c
                po["status"] = "received"
        elif kind == "gift":
            if e[1] in gifts: why = "duplicate gift " + e[1]
            else: gifts[e[1]] = {"sold": e[2], "balance": e[2]}
        elif kind == "customer":
            if e[1] in custs: custs[e[1]]["tier"] = e[2]
            else: custs[e[1]] = {"tier": e[2], "credit": 0}
        elif kind == "promo":
            _, code, cat, pct, uses = e
            if code in promos: why = "duplicate promo " + code
            else: promos[code] = {"cat": cat, "pct": pct, "uses": uses, "used": 0}
        elif kind == "order":
            _, oid, cid, its = e
            if oid in orders: why = "duplicate order " + oid
            elif cid not in custs: why = "unknown %s %s" % (CUST, cid)
            else:
                why = next(("unknown item " + s for s, _ in its if s not in items), None)
                why = why or next(("short " + s for s, q in its if items[s]["onhand"] - items[s]["reserved"] < q), None)
            if not why:
                ls = [{"sku": s, "qty": q, "price": items[s]["price"], "cat": items[s]["cat"], "rate": rates.get(items[s]["cat"], 0),
                       "tier": TIER_PCT[custs[cid]["tier"]], "returned": 0, "rnet": 0, "rtax": 0, "cost": 0, "rcost": 0} for s, q in its]
                for s, q in its: items[s]["reserved"] += q
                o = orders[oid] = {"id": oid, "cust": cid, "status": "open", "lines": ls, "promo": None, "paid": 0}
                reprice(o, promos)
        else:
            o = orders.get(e[1])
            if o is None: why = "unknown order " + e[1]
            elif kind == "apply":
                code = e[2]; p = promos.get(code)
                if o["status"] != "open": why = "order %s is %s" % (o["id"], o["status"])
                elif p is None: why = "unknown promo " + code
                elif o["promo"]: why = "promo already on " + o["id"]
                elif p["used"] >= p["uses"]: why = "promo %s used up" % code
                elif not any(l["cat"] == p["cat"] for l in o["lines"]): why = "promo %s does not apply" % code
                else:
                    o["promo"] = code; p["used"] += 1; reprice(o, promos)
                    if o["paid"] >= o["total"]:
                        custs[o["cust"]]["credit"] += o["paid"] - o["total"]; o["paid"] = o["total"]; o["status"] = "paid"
            elif kind == "pay":
                c = custs[o["cust"]]
                if o["status"] != "open": why = "order %s is %s" % (o["id"], o["status"])
                elif e[2] is None and c["credit"] == 0: why = "no credit " + o["cust"]
                elif isinstance(e[2], str) and e[2] not in gifts: why = "unknown gift " + e[2]
                elif isinstance(e[2], str) and gifts[e[2]]["balance"] == 0: why = "gift %s is empty" % e[2]
                else:
                    due = o["total"] - o["paid"]
                    if e[2] is None:
                        amount = min(c["credit"], due); c["credit"] -= amount
                    elif isinstance(e[2], str):
                        amount = min(gifts[e[2]]["balance"], due); gifts[e[2]]["balance"] -= amount
                    else: amount = e[2]
                    applied = min(amount, due); o["paid"] += applied; c["credit"] += amount - applied
                    if o["paid"] == o["total"]: o["status"] = "paid"
            elif kind == "ship":
                if o["status"] != "paid": why = "order %s is %s" % (o["id"], o["status"])
                else:
                    for l in o["lines"]:
                        it = items[l["sku"]]
                        l["cost"] = it["value"] * l["qty"] // it["onhand"]; it["value"] -= l["cost"]
                        it["onhand"] -= l["qty"]; it["reserved"] -= l["qty"]
                    o["status"] = "shipped"
            elif kind == "cancel":
                if o["status"] not in ("open", "paid"): why = "order %s is %s" % (o["id"], o["status"])
                else:
                    for l in o["lines"]: items[l["sku"]]["reserved"] -= l["qty"]
                    custs[o["cust"]]["credit"] += o["paid"]; o["paid"] = 0
                    if o["promo"]: promos[o["promo"]]["used"] -= 1
                    o["status"] = "cancelled"
            elif kind == "return":
                _, _, sku, q = e
                l = next((l for l in o["lines"] if l["sku"] == sku), None)
                if o["status"] != "shipped": why = "order %s is %s" % (o["id"], o["status"])
                elif l is None: why = "%s not in %s" % (sku, o["id"])
                elif q > l["qty"] - l["returned"]: why = "too many " + sku
                else:
                    r0, r1 = l["returned"], l["returned"] + q
                    dn = l["net"] * r1 // l["qty"] - l["net"] * r0 // l["qty"]
                    dt = l["tax"] * r1 // l["qty"] - l["tax"] * r0 // l["qty"]
                    dc = l["cost"] * r1 // l["qty"] - l["cost"] * r0 // l["qty"]
                    l["returned"] = r1; l["rnet"] += dn; l["rtax"] += dt; l["rcost"] += dc; items[sku]["value"] += dc
                    custs[o["cust"]]["credit"] += dn + dt; items[sku]["onhand"] += q
                    if all(x["returned"] == x["qty"] for x in o["lines"]): o["status"] = "returned"
        if why:
            sys.stderr.write("error: line %d: %s\n" % (k, why)); bad += 1
    key = lambda s: s.encode()
    asc = lambda d: sorted(d, key=key)
    out = []
    cmd = argv[0]
    sold = ("shipped", "returned")
    refund = lambda o: sum(l["rnet"] + l["rtax"] for l in o["lines"])
    if cmd == "orders":
        out = ["\t".join((o["id"], o["cust"], o["status"], money(o["total"]), money(o["paid"]))) for o in orders.values()]
    elif cmd == "invoice":
        o = orders.get(argv[1])
        if o is None:
            sys.stderr.write("error: no order %s\n" % argv[1]); return 3
        out = ["order %s %s %s" % (o["id"], o["cust"], o["status"])]
        for l in o["lines"]:
            out.append("\t".join([l["sku"], str(l["qty"])] + [money(l[x]) for x in ("price", "gross", "discount", "tax", "total")]))
        if o["promo"]: out.append("promo\t" + o["promo"])
        s = lambda x: sum(l[x] for l in o["lines"])
        out += ["subtotal\t" + money(s("gross")), "discount\t" + money(s("discount")), "tax\t" + money(s("tax")), "total\t" + money(o["total"])]
        if refund(o) > 0: out.append("returned\t" + money(refund(o)))
        out += ["paid\t" + money(o["paid"]), "due\t" + money(o["total"] - o["paid"] if o["status"] == "open" else 0)]
    elif cmd == "stock":
        out = ["\t".join((s, items[s]["cat"], money(items[s]["price"]), str(items[s]["onhand"]), str(items[s]["reserved"]),
                          str(items[s]["onhand"] - items[s]["reserved"]), money(items[s]["value"]))) for s in asc(items)]
    elif cmd == "low":
        n = int(argv[1])
        out = ["%s\t%d" % (s, items[s]["onhand"] - items[s]["reserved"]) for s in asc(items) if items[s]["onhand"] - items[s]["reserved"] < n]
    elif cmd == CUST + "s":
        for cid in asc(custs):
            mine = [o for o in orders.values() if o["cust"] == cid]
            n = sum(o["status"] != "cancelled" for o in mine)
            spent = sum(o["total"] - refund(o) for o in mine if o["status"] in sold)
            out.append("\t".join((cid, custs[cid]["tier"], str(n), money(spent), money(custs[cid]["credit"]))))
    elif cmd == "history":
        c = custs.get(argv[1])
        if c is None:
            sys.stderr.write("error: no %s %s\n" % (CUST, argv[1])); return 3
        out = ["\t".join((o["id"], o["status"], money(o["total"]))) for o in orders.values() if o["cust"] == argv[1]]
        out.append("credit\t" + money(c["credit"]))
    elif cmd == "sales":
        cats = {}
        for o in orders.values():
            if o["status"] not in sold: continue
            for l in o["lines"]:
                c = cats.setdefault(l["cat"], [0, 0, 0])
                c[0] += l["qty"] - l["returned"]; c[1] += l["net"] - l["rnet"]; c[2] += l["tax"] - l["rtax"]
        t = [sum(c[i] for c in cats.values()) for i in range(3)]
        out = ["%s\t%d\t%s\t%s" % (c, cats[c][0], money(cats[c][1]), money(cats[c][2])) for c in asc(cats)]
        out.append("total\t%d\t%s\t%s" % (t[0], money(t[1]), money(t[2])))
    elif cmd == "margin":
        cats = {}
        for o in orders.values():
            if o["status"] not in sold: continue
            for l in o["lines"]:
                c = cats.setdefault(l["cat"], [0, 0])
                c[0] += l["net"] - l["rnet"]; c[1] += l["cost"] - l["rcost"]
        rows = [(c, cats[c]) for c in asc(cats)] + [("total", [sum(v[0] for v in cats.values()), sum(v[1] for v in cats.values())])]
        out = ["%s\t%s\t%s\t%s" % (c, money(v[0]), money(v[1]), money(v[0] - v[1])) for c, v in rows]
    elif cmd == "purchases":
        out = ["%s\t%s\t%d\t%s" % (p, po["status"], sum(b[1] for b in po["buys"]), money(sum(b[1] * b[2] for b in po["buys"]))) for p, po in pos.items()]
    elif cmd == "gifts":
        out = ["%s\t%s\t%s" % (g, money(gifts[g]["sold"]), money(gifts[g]["balance"])) for g in asc(gifts)]
    elif cmd == "promos":
        out = ["%s\t%s\t%d%%\t%d/%d" % (c, promos[c]["cat"], promos[c]["pct"], promos[c]["used"], promos[c]["uses"]) for c in asc(promos)]
    sys.stdout.write("".join(l + "\n" for l in out))
    return 2 if bad else 0

if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
