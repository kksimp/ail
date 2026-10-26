#!/usr/bin/env python3
"""Reference oracle for the ledger task: `ref.py ARGS... < csv` -> stdout, exit code (same contract as spec.md)."""
import re, sys

def money(c):
    s = "-" if c < 0 else ""
    c = abs(c)
    return "%s%d.%02d" % (s, c // 100, c % 100)

def main(argv):
    ok = (len(argv) == 1 and argv[0] in ("balance", "monthly")) or \
         (len(argv) == 2 and argv[0] == "top" and re.fullmatch(r"[0-9]+", argv[1]))
    if not ok:
        sys.stderr.write("usage: ledger balance|top N|monthly\n")
        return 1
    lines = sys.stdin.buffer.read().decode("utf-8", "replace").split("\n")
    if lines and lines[-1] == "":
        lines.pop()
    if not lines or lines[0] != "date,account,amount_cents,memo":
        sys.stderr.write("error: line 1: bad header\n")
        return 2
    rows, bad = [], 0
    for k, ln in enumerate(lines[1:], 2):
        f = ln.split(",")
        why = None
        if len(f) != 4:
            why = "expected 4 fields"
        elif not re.fullmatch(r"[0-9]{4}-(0[1-9]|1[0-2])-(0[1-9]|[12][0-9]|3[01])", f[0]):
            why = "bad date"
        elif f[1] == "":
            why = "empty account"
        elif not re.fullmatch(r"-?[0-9]+", f[2]):
            why = "bad amount"
        if why:
            sys.stderr.write("error: line %d: %s\n" % (k, why))
            bad += 1
        else:
            rows.append((f[0], f[1], int(f[2]), f[3]))
    out = []
    if argv[0] == "balance" or argv[0] == "monthly":
        tot = {}
        for d, a, c, _ in rows:
            key = a if argv[0] == "balance" else d[:7]
            tot[key] = tot.get(key, 0) + c
        out = ["%s\t%s" % (k, money(tot[k])) for k in sorted(tot, key=lambda s: s.encode())]
    else:
        top = sorted(rows, key=lambda r: -abs(r[2]))[: int(argv[1])]
        out = ["%s\t%s\t%s\t%s" % (d, a, money(c), m) for d, a, c, m in top]
    sys.stdout.write("".join(l + "\n" for l in out))
    return 2 if bad else 0

if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
