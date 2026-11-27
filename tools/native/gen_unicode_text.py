#!/usr/bin/env python3
"""Writes the Unicode tables of `std.unicode` (std/unicode/unicode.ail, ADR 1256)
and its conformance test (tests/cases/std_unicode_conformance.ail) from the
Unicode Character Database:

    tools/native/unicode-text/run.sh          # fetches the pinned UCD files, then runs this

or, with the files already in DIR:

    python3 tools/native/gen_unicode_text.py DIR

The tables replace the lines between `## BEGIN GENERATED TABLES` and
`## END GENERATED TABLES` in std/unicode/unicode.ail; nothing else in that
file is touched. Before writing, the script decodes its own tables and runs
the conformance data (NormalizationTest.txt, GraphemeBreakTest.txt) through a
Python copy of the module's algorithms, and stops if a case fails.

Each table is a string of numbers. A number is its base-45 digits, most
significant first: every digit but the last is one of the first 45 characters
of ALPHABET, the last one of the second 45. ALPHABET is printable ASCII
without `"`, `\\`, `{` and `}`, so a table is a plain AIL string literal.
Sorted tables store differences from the previous entry (see each table's
comment in the AIL file)."""

import os
import sys

VERSION = "16.0.0"
ALPHABET = "".join(chr(c) for c in range(33, 127) if chr(c) not in '"\\{}')
assert len(ALPHABET) == 90
MORE, LAST = ALPHABET[:45], ALPHABET[45:]

HANGUL_S, HANGUL_L, HANGUL_V, HANGUL_T = 0xAC00, 0x1100, 0x1161, 0x11A7
HANGUL_LCOUNT, HANGUL_VCOUNT, HANGUL_TCOUNT = 19, 21, 28
HANGUL_NCOUNT = HANGUL_VCOUNT * HANGUL_TCOUNT
HANGUL_SCOUNT = HANGUL_LCOUNT * HANGUL_NCOUNT

# Grapheme classes (the AIL file's GC_* constants).
GCB = ["Other", "CR", "LF", "Control", "Extend", "ZWJ", "Regional_Indicator",
       "Prepend", "SpacingMark", "L", "V", "T", "LV", "LVT", "ExtPict"]
INCB = ["None", "Linker", "Consonant", "Extend"]


# ---------------------------------------------------------------- reading


def data_lines(path):
    with open(path, encoding="utf-8") as f:
        for line in f:
            line = line.split("#", 1)[0].strip()
            if line:
                yield [p.strip() for p in line.split(";")]


def code_range(field):
    if ".." in field:
        a, b = field.split("..")
        return int(a, 16), int(b, 16)
    return int(field, 16), int(field, 16)


def read_unicode_data(ucd):
    ccc, decomp, compat, mn = {}, {}, {}, set()
    first = None
    for f in data_lines(os.path.join(ucd, "UnicodeData.txt")):
        cp = int(f[0], 16)
        if f[1].endswith(", First>"):
            first = cp
            continue
        span = range(first, cp + 1) if f[1].endswith(", Last>") else [cp]
        first = None
        for c in span:
            if int(f[3]) != 0:
                ccc[c] = int(f[3])
            if f[2] == "Mn":
                mn.add(c)
        if f[5]:
            parts = f[5].split()
            if parts[0].startswith("<"):
                compat[cp] = [int(p, 16) for p in parts[1:]]
            else:
                decomp[cp] = [int(p, 16) for p in parts]
    return ccc, decomp, compat, mn


def read_exclusions(ucd):
    out = set()
    for f in data_lines(os.path.join(ucd, "DerivedNormalizationProps.txt")):
        if f[1] == "Full_Composition_Exclusion":
            lo, hi = code_range(f[0])
            out.update(range(lo, hi + 1))
    return out


def read_folding(ucd):
    out = {}
    for f in data_lines(os.path.join(ucd, "CaseFolding.txt")):
        if f[1] in ("C", "F"):
            out[int(f[0], 16)] = [int(p, 16) for p in f[2].split()]
    return out


def read_property(path, prop_field, names, value=None):
    out = {}
    for f in data_lines(path):
        if value is not None:
            if len(f) < 3 or f[1] != prop_field:
                continue
            name = f[2]
        else:
            name = f[1]
        if name not in names:
            continue
        lo, hi = code_range(f[0])
        for c in range(lo, hi + 1):
            out[c] = names.index(name)
    return out


def read_gcb(ucd):
    gcb = read_property(os.path.join(ucd, "GraphemeBreakProperty.txt"), None, GCB)
    pict = 0
    for f in data_lines(os.path.join(ucd, "emoji-data.txt")):
        if f[1] != "Extended_Pictographic":
            continue
        lo, hi = code_range(f[0])
        for c in range(lo, hi + 1):
            if c in gcb:
                sys.exit(f"U+{c:04X} is Extended_Pictographic and {GCB[gcb[c]]}")
            gcb[c] = GCB.index("ExtPict")
            pict += 1
    # Hangul syllables are classed by the module's arithmetic, not the table.
    for c in range(HANGUL_S, HANGUL_S + HANGUL_SCOUNT):
        want = "LV" if (c - HANGUL_S) % HANGUL_TCOUNT == 0 else "LVT"
        assert GCB[gcb[c]] == want, hex(c)
        del gcb[c]
    return gcb


def read_incb(ucd):
    return read_property(os.path.join(ucd, "DerivedCoreProperties.txt"), "InCB", INCB, value=True)


# ---------------------------------------------------------------- tables


def full_decomp(cp, decomp, compat, use_compat):
    if HANGUL_S <= cp < HANGUL_S + HANGUL_SCOUNT:
        return hangul_decomp(cp)
    m = (compat.get(cp) if use_compat else None) or decomp.get(cp)
    if m is None:
        return [cp]
    out = []
    for c in m:
        out.extend(full_decomp(c, decomp, compat, use_compat))
    return out


def hangul_decomp(cp):
    s = cp - HANGUL_S
    out = [HANGUL_L + s // HANGUL_NCOUNT, HANGUL_V + s % HANGUL_NCOUNT // HANGUL_TCOUNT]
    if s % HANGUL_TCOUNT:
        out.append(HANGUL_T + s % HANGUL_TCOUNT)
    return out


def ranges(prop):
    """Sorted (lo, hi, value) runs of a code -> value map."""
    out = []
    for c in sorted(prop):
        v = prop[c]
        if out and out[-1][1] == c - 1 and out[-1][2] == v:
            out[-1][1] = c
        else:
            out.append([c, c, v])
    return out


def encode(nums):
    parts = []
    for n in nums:
        assert n >= 0, n
        digits = []
        while True:
            digits.append(n % 45)
            n //= 45
            if n == 0:
                break
        digits.reverse()
        parts.append("".join(MORE[d] for d in digits[:-1]) + LAST[digits[-1]])
    return "".join(parts)


def decode(s):
    out, acc = [], 0
    for ch in s:
        i = ALPHABET.index(ch)
        if i < 45:
            acc = acc * 45 + i
        else:
            out.append(acc * 45 + i - 45)
            acc = 0
    return out


def range_nums(rs, with_value=True):
    """(lo - previous hi, hi - lo[, value]) for each run."""
    out, prev = [], 0
    for lo, hi, v in rs:
        out += [lo - prev, hi - lo] + ([v] if with_value else [])
        prev = hi
    return out


def map_nums(m):
    """Key difference, length, then the values, for each sorted key."""
    out, prev = [], 0
    for k in sorted(m):
        out += [k - prev, len(m[k])] + m[k]
        prev = k
    return out


def pair_nums(pairs):
    """(first difference, second, composite) for each pair sorted by (first, second);
    the first difference is from the previous pair's first."""
    out, prev = [], 0
    for (a, b), c in sorted(pairs.items()):
        out += [a - prev, b, c]
        prev = a
    return out


def build(ucd):
    ccc, decomp, compat, mn = read_unicode_data(ucd)
    excl = read_exclusions(ucd)
    canon = {}
    kcanon = {}
    for cp in set(decomp) | set(compat):
        d = full_decomp(cp, decomp, compat, False)
        k = full_decomp(cp, decomp, compat, True)
        if d != [cp]:
            canon[cp] = d
        if k != d:
            kcanon[cp] = k
    pairs = {}
    for cp, m in decomp.items():
        if len(m) == 2 and cp not in excl:
            pairs[(m[0], m[1])] = cp
    fold = read_folding(ucd)
    gcb = read_gcb(ucd)
    incb = read_incb(ucd)
    # The module skips the table lookups below these code points.
    assert min(ccc) == 0x300 and min(incb) >= 0x300 and min(b for _, b in pairs) >= 0x300
    assert min(canon) >= 0xC0 and min(kcanon) >= 0xA0 and min(mn) >= 0x300
    assert all(gcb.get(c, 0) == (3 if c < 32 else 0) for c in range(0x7F) if c not in (10, 13))
    assert all(c in fold or not (0x41 <= c <= 0x5A) for c in range(0x80)) and all(0x41 <= c <= 0x5A for c in fold if c < 0x80)
    tables = {
        "CCC": range_nums(ranges(ccc)),
        "DECOMP": map_nums(canon),
        "KDECOMP": map_nums(kcanon),
        "COMPOSE": pair_nums(pairs),
        "FOLD": map_nums(fold),
        "GRAPHEME": range_nums(ranges(gcb)),
        "INCB": range_nums(ranges(incb)),
        "MARK": range_nums(ranges({c: 0 for c in mn}), with_value=False),
    }
    return {k: encode(v) for k, v in tables.items()}


# ---------------------------------------------------------------- the algorithms (a copy of the AIL module's)


class Tables:
    def __init__(self, enc):
        t = {k: decode(v) for k, v in enc.items()}
        self.ccc = self.runs(t["CCC"])
        self.decomp = self.maps(t["DECOMP"])
        self.kdecomp = self.maps(t["KDECOMP"])
        self.fold = self.maps(t["FOLD"])
        self.compose = {}
        prev, xs = 0, t["COMPOSE"]
        for i in range(0, len(xs), 3):
            prev += xs[i]
            self.compose[(prev, xs[i + 1])] = xs[i + 2]
        self.gcb = self.runs(t["GRAPHEME"])
        self.incb = self.runs(t["INCB"])
        self.mark = self.runs(t["MARK"], 2)

    @staticmethod
    def runs(xs, step=3):
        out, prev = {}, 0
        for i in range(0, len(xs), step):
            lo = prev + xs[i]
            hi = lo + xs[i + 1]
            for c in range(lo, hi + 1):
                out[c] = xs[i + 2] if step == 3 else 1
            prev = hi
        return out

    @staticmethod
    def maps(xs):
        out, prev, i = {}, 0, 0
        while i < len(xs):
            prev += xs[i]
            n = xs[i + 1]
            out[prev] = xs[i + 2:i + 2 + n]
            i += 2 + n
        return out


def decompose(t, cps, compat):
    out, ccs = [], []
    for cp in cps:
        if HANGUL_S <= cp < HANGUL_S + HANGUL_SCOUNT:
            parts = hangul_decomp(cp)
        else:
            parts = (t.kdecomp.get(cp) if compat else None) or t.decomp.get(cp) or [cp]
        for c in parts:
            cc = t.ccc.get(c, 0)
            out.append(c)
            ccs.append(cc)
            i = len(out) - 1
            while cc != 0 and i > 0 and ccs[i - 1] > cc:
                out[i], out[i - 1] = out[i - 1], out[i]
                ccs[i], ccs[i - 1] = ccs[i - 1], ccs[i]
                i -= 1
    return out


def compose_pair(t, a, b):
    if HANGUL_L <= a < HANGUL_L + HANGUL_LCOUNT and HANGUL_V <= b < HANGUL_V + HANGUL_VCOUNT:
        return HANGUL_S + ((a - HANGUL_L) * HANGUL_VCOUNT + b - HANGUL_V) * HANGUL_TCOUNT
    s = a - HANGUL_S
    if 0 <= s < HANGUL_SCOUNT and s % HANGUL_TCOUNT == 0 and HANGUL_T < b < HANGUL_T + HANGUL_TCOUNT:
        return a + b - HANGUL_T
    return t.compose.get((a, b))


def compose(t, cps):
    out, starter, last_cc = [], -1, 0
    for c in cps:
        cc = t.ccc.get(c, 0)
        if starter >= 0 and (len(out) - 1 == starter or (last_cc != 0 and last_cc < cc)):
            m = compose_pair(t, out[starter], c)
            if m is not None:
                out[starter] = m
                continue
        if cc == 0:
            starter = len(out)
        last_cc = cc
        out.append(c)
    return out


def gclass(t, c):
    s = c - HANGUL_S
    if 0 <= s < HANGUL_SCOUNT:
        return GCB.index("LV") if s % HANGUL_TCOUNT == 0 else GCB.index("LVT")
    return t.gcb.get(c, 0)


def graphemes(t, cps):
    G = {n: i for i, n in enumerate(GCB)}
    out = []
    prev = -1
    ri = 0          # regional indicators in a row before this character
    pict = 0        # 1: ExtPict Extend*, 2: then ZWJ
    conj = 0        # 1: InCB Consonant [Extend Linker]*, 2: with a Linker in it
    for c in cps:
        k = gclass(t, c)
        ib = t.incb.get(c, 0)
        brk = True
        if prev < 0:
            brk = True
        elif prev == G["CR"] and k == G["LF"]:
            brk = False
        elif prev in (G["Control"], G["CR"], G["LF"]) or k in (G["Control"], G["CR"], G["LF"]):
            brk = True
        elif prev == G["L"] and k in (G["L"], G["V"], G["LV"], G["LVT"]):
            brk = False
        elif prev in (G["LV"], G["V"]) and k in (G["V"], G["T"]):
            brk = False
        elif prev in (G["LVT"], G["T"]) and k == G["T"]:
            brk = False
        elif k in (G["Extend"], G["ZWJ"], G["SpacingMark"]) or prev == G["Prepend"]:
            brk = False
        elif ib == INCB.index("Consonant") and conj == 2:
            brk = False
        elif prev == G["ZWJ"] and k == G["ExtPict"] and pict == 2:
            brk = False
        elif prev == G["Regional_Indicator"] and k == G["Regional_Indicator"] and ri % 2 == 1:
            brk = False
        if brk:
            out.append([c])
        else:
            out[-1].append(c)
        # states after this character
        ri = ri + 1 if k == G["Regional_Indicator"] else 0
        if k == G["ExtPict"]:
            pict = 1
        elif k == G["Extend"] and pict == 1:
            pict = 1
        elif k == G["ZWJ"] and pict == 1:
            pict = 2
        else:
            pict = 0
        if ib == INCB.index("Consonant"):
            conj = 1
        elif ib == INCB.index("Linker") and conj > 0:
            conj = 2
        elif ib == INCB.index("Extend") and conj > 0:
            pass
        else:
            conj = 0
        prev = k
    return out


# ---------------------------------------------------------------- conformance


def norm_cases(ucd):
    """The part name and the five columns of each line of NormalizationTest.txt."""
    part = ""
    with open(os.path.join(ucd, "NormalizationTest.txt"), encoding="utf-8") as f:
        for line in f:
            if line.startswith("@Part"):
                part = line.split()[0]
                continue
            line = line.split("#", 1)[0].strip()
            if not line:
                continue
            cols = [[int(x, 16) for x in c.split()] for c in line.split(";")[:5]]
            yield part, cols


def grapheme_cases(ucd):
    with open(os.path.join(ucd, "GraphemeBreakTest.txt"), encoding="utf-8") as f:
        for line in f:
            line = line.split("#", 1)[0].strip()
            if not line:
                continue
            clusters, cur = [], []
            for tok in line.split():
                if tok == "÷":
                    if cur:
                        clusters.append(cur)
                    cur = []
                elif tok == "×":
                    pass
                else:
                    cur.append(int(tok, 16))
            yield clusters


def check(t, ucd):
    bad = 0
    for part, (c1, c2, c3, c4, c5) in norm_cases(ucd):
        nfd = lambda x: decompose(t, x, False)
        nfkd = lambda x: decompose(t, x, True)
        nfc = lambda x: compose(t, nfd(x))
        nfkc = lambda x: compose(t, nfkd(x))
        for got, want in [(nfc(c1), c2), (nfc(c2), c2), (nfc(c3), c2), (nfc(c4), c4), (nfc(c5), c4),
                          (nfd(c1), c3), (nfd(c3), c3), (nfd(c5), c5), (nfkc(c1), c4), (nfkc(c5), c4),
                          (nfkd(c1), c5), (nfkd(c5), c5)]:
            if got != want:
                bad += 1
                if bad < 10:
                    print(f"normalization {part} {c1}: got {got}, want {want}", file=sys.stderr)
    for want in grapheme_cases(ucd):
        flat = [c for g in want for c in g]
        got = graphemes(t, flat)
        if got != want:
            bad += 1
            if bad < 20:
                print(f"graphemes {flat}: got {got}, want {want}", file=sys.stderr)
    if bad:
        sys.exit(f"{bad} conformance cases fail")


# ---------------------------------------------------------------- writing


def hexes(cps):
    return " ".join(f"{c:X}" for c in cps)


def write_module(path, enc):
    with open(path, encoding="utf-8") as f:
        text = f.read()
    begin, end = "## BEGIN GENERATED TABLES\n", "## END GENERATED TABLES\n"
    a, b = text.index(begin) + len(begin), text.index(end)
    body = [f"## Written by tools/native/gen_unicode_text.py from the Unicode {VERSION} character",
            "## database. Do not edit; run the generator.",
            f'UNICODE_VERSION := "{VERSION}"']
    for name, s in enc.items():
        body.append(f'T_{name} := "{s}"')
    with open(path, "w", encoding="utf-8") as f:
        f.write(text[:a] + "\n".join(body) + "\n" + text[b:])


# Every Nth line of a part of NormalizationTest.txt goes into the AIL test.
SAMPLE = {"@Part1": 32, "@Part2": 4, "@Part4": 4}


def write_test(path, ucd):
    norm, n = [], 0
    for part, cols in norm_cases(ucd):
        n += 1
        if n % SAMPLE.get(part, 1):
            continue
        norm.append(";".join(hexes(c) for c in cols))
    graph = ["|".join(hexes(g) for g in cl) for cl in grapheme_cases(ucd)]
    lines = [
        "## std.unicode.2-8 against the Unicode " + VERSION + " conformance data: NormalizationTest.txt",
        "## (parts 0, 3 and 5 whole, every 4th line of parts 2 and 4, every 32nd of part 1) and",
        "## GraphemeBreakTest.txt (every line).",
        "## Written by tools/native/gen_unicode_text.py. Do not edit; run the generator.",
        "",
        "## The text of the code points written in hex, separated by spaces.",
        "fn text(hex str) str",
        "    mut cs := List[char]()",
        "    for w in hex.words()",
        "        n := w.to_int(16) else 0",
        "        cs.push(char.from_code(n) else ' ')",
        "    cs.join(\"\")",
        "",
        "fn codes(s str) str",
        "    [c.code().to_hex() for c in s.chars()].join(\" \")",
        "",
        "## 1 when `got` is not `want`, printing the first five such cases.",
        "fn same(bad int, form str, src str, got str, want str) int",
        "    if got == want",
        "        return 0",
        "    if bad < 5",
        "        print(\"{form} {codes(src)}: got {codes(got)}, want {codes(want)}\")",
        "    1",
        "",
        "fn normalization() int",
        "    mut bad := 0",
        "    for line in NORM.lines()",
        "        cols := [text(c) for c in line.split(\";\")]",
        "        c1 := cols[0]",
        "        c2 := cols[1]",
        "        c3 := cols[2]",
        "        c4 := cols[3]",
        "        c5 := cols[4]",
        "        bad += same(bad, \"nfc\", c1, unicode.nfc(c1), c2)",
        "        bad += same(bad, \"nfc\", c2, unicode.nfc(c2), c2)",
        "        bad += same(bad, \"nfc\", c3, unicode.nfc(c3), c2)",
        "        bad += same(bad, \"nfc\", c4, unicode.nfc(c4), c4)",
        "        bad += same(bad, \"nfc\", c5, unicode.nfc(c5), c4)",
        "        bad += same(bad, \"nfd\", c1, unicode.nfd(c1), c3)",
        "        bad += same(bad, \"nfd\", c3, unicode.nfd(c3), c3)",
        "        bad += same(bad, \"nfd\", c5, unicode.nfd(c5), c5)",
        "        bad += same(bad, \"nfkc\", c1, unicode.nfkc(c1), c4)",
        "        bad += same(bad, \"nfkc\", c5, unicode.nfkc(c5), c4)",
        "        bad += same(bad, \"nfkd\", c1, unicode.nfkd(c1), c5)",
        "        bad += same(bad, \"nfkd\", c5, unicode.nfkd(c5), c5)",
        "    bad",
        "",
        "fn graphemes() int",
        "    mut bad := 0",
        "    for line in GRAPH.lines()",
        "        want := [text(g) for g in line.split(\"|\")]",
        "        whole := want.join(\"\")",
        "        got := unicode.graphemes(whole)",
        "        if got != want or unicode.grapheme_count(whole) != want.len()",
        "            if bad < 5",
        "                shown := [codes(g) for g in got]",
        "                print(\"graphemes {line}: got {shown.join(\"|\")}\")",
        "            bad += 1",
        "    bad",
        "",
        "fn main()",
        "    print(\"normalization: {NORM.lines().len()} lines, {normalization()} failing\")",
        "    print(\"graphemes: {GRAPH.lines().len()} lines, {graphemes()} failing\")",
        "",
        "NORM := \"\"\"",
    ]
    lines += ["    " + x for x in norm]
    lines += ["    \"\"\"", "", "GRAPH := \"\"\""]
    lines += ["    " + x for x in graph]
    lines += ["    \"\"\""]
    with open(path, "w", encoding="utf-8") as f:
        f.write("\n".join(lines) + "\n")
    with open(path[:-4] + ".out", "w", encoding="utf-8") as f:
        f.write(f"normalization: {len(norm)} lines, 0 failing\ngraphemes: {len(graph)} lines, 0 failing\n")


def main():
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    ucd = sys.argv[1]
    enc = build(ucd)
    check(Tables(enc), ucd)
    root = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
    write_module(os.path.join(root, "std", "unicode", "unicode.ail"), enc)
    write_test(os.path.join(root, "tests", "cases", "std_unicode_conformance.ail"), ucd)
    sizes = ", ".join(f"{k} {len(v)}" for k, v in enc.items())
    print(f"Unicode {VERSION}: {sizes} characters")


if __name__ == "__main__":
    main()
