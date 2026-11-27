#!/usr/bin/env python3
"""Writes the SQLite fixtures of the std.sqlite tests and their expected output.

    python3 tests/sqlite/gen.py            # from the repository root

Everything the .out files hold is read back from the finished files by SQLite
itself (the `sqlite3` module links the real engine), never typed by hand. The
script also runs a Python port of the reader over every fixture and fails when
it disagrees with SQLite, so the .out files are checked against two readers.
The fixtures are committed; run the script again only to change them.
"""
import binascii
import os
import sqlite3
import struct
import sys
import zlib

HERE = os.path.dirname(os.path.abspath(__file__))
CASES = os.path.join(HERE, "..", "cases")


def fresh(name, page_size):
    path = os.path.join(HERE, name)
    for suffix in ("", "-journal", "-wal", "-shm"):
        if os.path.exists(path + suffix):
            os.remove(path + suffix)
    con = sqlite3.connect(path)
    con.execute("PRAGMA page_size=%d" % page_size)
    return con, path


def lcg_bytes(seed, n):
    out = bytearray()
    x = seed
    for _ in range(n):
        x = (x * 1103515245 + 12345) & 0x7FFFFFFF
        out.append((x >> 16) & 255)
    return bytes(out)


def make_basic():
    con, path = fresh("basic.db", 1024)
    con.executescript(
        """
        CREATE TABLE people(id INTEGER PRIMARY KEY, name TEXT NOT NULL, age INTEGER, score REAL, bio BLOB);
        CREATE TABLE ints(n INTEGER);
        CREATE TABLE reals(x REAL, y NUMERIC, z);
        CREATE TABLE texts(t TEXT);
        CREATE TABLE "order items"(
            "item id" INTEGER PRIMARY KEY AUTOINCREMENT,
            "unit price" REAL DEFAULT 1,
            qty INT,
            [weird name] TEXT,
            note VARCHAR(20) DEFAULT 'a,b(c)',
            UNIQUE(qty),
            CHECK(qty >= 0),
            FOREIGN KEY(qty) REFERENCES people(id)
        );
        CREATE INDEX idx_people_name ON people(name);
        CREATE INDEX idx_ints ON ints(n);
        CREATE INDEX idx_texts ON texts(t);
        CREATE VIEW v AS SELECT name FROM people;
        CREATE TRIGGER trg AFTER INSERT ON ints BEGIN SELECT 1; END;
        """
    )
    people = [
        (1, "Ann", 30, 91.5, b"\x00\xff"),
        (2, "Bob", None, None, None),
        (3, "héllo wörld ☃", 0, 1.0, b""),
        (4, "", 1, -0.5, b"\xde\xad\xbe\xef"),
        (5, "gone", 5, 5.5, None),
        (7, "It's", -1, 100.25, None),
    ]
    con.executemany("INSERT INTO people VALUES(?,?,?,?,?)", people)
    con.execute("DELETE FROM people WHERE id=5")
    ints = [
        0, 1, -1, 127, -128, 128, -129, 32767, -32768, 32768, 8388607, 8388608,
        -8388608, -8388609, 2147483647, -2147483648, 2147483648,
        140737488355327, 140737488355328, -140737488355328, -140737488355329,
        281474976710656, 9223372036854775807, -9223372036854775808,
    ]
    con.executemany("INSERT INTO ints VALUES(?)", [(n,) for n in ints])
    con.execute("DELETE FROM ints WHERE n=127")
    con.execute("INSERT INTO ints VALUES(NULL)")
    con.executemany(
        "INSERT INTO reals VALUES(?,?,?)",
        [(1.0, "12", "a"), (2.5, 3.5, None), (-3.0, 3.0, "7"), (0.1, "x", "y"), (1000, 100, 1)],
    )
    con.executemany(
        "INSERT INTO texts VALUES(?)",
        [("plain",), ("with 'quote'",), ("two\nlines",), ("tab\there",), ("\U0001F600",), ("",), (None,), ("plain",)],
    )
    con.executemany(
        'INSERT INTO "order items"(qty, "unit price", [weird name], note) VALUES(?,?,?,?)',
        [(1, 2.5, "w1", "n1"), (2, None, "w2", None), (3, 4.0, None, "n3")],
    )
    con.commit()
    con.close()
    return path


def make_big():
    con, path = fresh("big.db", 512)
    con.executescript(
        """
        CREATE TABLE nums(id INTEGER PRIMARY KEY, sq INTEGER, label TEXT, pad TEXT);
        CREATE TABLE loose(a, b);
        CREATE INDEX idx_label ON nums(label);
        """
    )
    con.executemany(
        "INSERT INTO nums VALUES(?,?,?,?)",
        [(i, i * i, "label_%04d" % (i % 100), "x" * 230) for i in range(1, 121)],
    )
    con.execute("DELETE FROM nums WHERE id % 7 = 0")
    con.executemany(
        "INSERT INTO loose VALUES(?,?)", [(i, "v%d" % i) for i in range(1, 301)]
    )
    con.execute("DELETE FROM loose WHERE a % 3 = 0")
    con.commit()
    con.close()
    return path


def make_overflow():
    con, path = fresh("overflow.db", 512)
    con.executescript(
        """
        CREATE TABLE docs(id INTEGER PRIMARY KEY, body TEXT, data BLOB);
        CREATE INDEX idx_body ON docs(body);
        """
    )
    rows = []
    for n in range(1, 21):
        size = 470 + n
        body = "".join(chr(97 + (i * 7 + n) % 26) for i in range(size))
        rows.append((n, body, None if n % 5 == 0 else lcg_bytes(n, n * 3)))
    for n in range(21, 37):
        size = 90 + n
        body = "".join(chr(97 + (i * 5 + n) % 26) for i in range(size))
        rows.append((n, body, lcg_bytes(n, n)))
    for k, n in enumerate([1000, 3000]):
        body = "".join(chr(97 + (i * 11 + k) % 26) for i in range(n))
        rows.append((37 + k, body, lcg_bytes(900 + k, n * 2)))
    con.executemany("INSERT INTO docs VALUES(?,?,?)", rows)
    con.commit()
    con.close()
    return path


# ---------------------------------------------------------------- expected output


def show(v):
    if v is None:
        return "NULL"
    if isinstance(v, int):
        return str(v)
    if isinstance(v, float):
        return repr(v)
    if isinstance(v, str):
        return "'" + v.replace("'", "''") + "'"
    return "X'" + binascii.hexlify(v).decode().upper() + "'"


def qi(name):
    return '"' + name.replace('"', '""') + '"'


def columns(con, table):
    return [r[1] for r in con.execute("PRAGMA table_info(%s)" % qi(table))]


def index_entries(con, name, table):
    cols = [r[2] for r in con.execute("PRAGMA index_info(%s)" % qi(name))]
    cl = ",".join(qi(c) for c in cols)
    return [
        (list(r[:-1]), r[-1])
        for r in con.execute(
            "SELECT %s, rowid FROM %s ORDER BY %s, rowid" % (cl, qi(table), cl)
        )
    ]


def scan(con, table):
    return list(con.execute("SELECT rowid, * FROM %s ORDER BY rowid" % qi(table)))


def basic_out(path):
    con = sqlite3.connect(path)
    out = []
    pages = con.execute("PRAGMA page_count").fetchone()[0]
    size = con.execute("PRAGMA page_size").fetchone()[0]
    out.append("pages %d size %d" % (pages, size))
    sch = list(con.execute("SELECT type, name, tbl_name, rootpage FROM sqlite_master ORDER BY rowid"))
    for k, n, t, r in sch:
        out.append("%s %s %s %d" % (k, n, t, r))
    for k, n, t, r in sch:
        if k == "table":
            out.append("table %s: %s" % (n, ",".join(columns(con, n))))
            rows = scan(con, n)
            out.append("count %d rows %d" % (len(rows), len(rows)))
            for row in rows:
                out.append("%d|%s" % (row[0], "|".join(show(v) for v in row[1:])))
    for k, n, t, r in sch:
        if k == "index":
            out.append("index %s" % n)
            for vals, rid in index_entries(con, n, t):
                out.append("%s -> %d" % ("|".join(show(v) for v in vals), rid))
    con.close()
    return out


def big_out(path):
    con = sqlite3.connect(path)
    out = []
    size = con.execute("PRAGMA page_size").fetchone()[0]
    out.append("page size %d" % size)
    rows = scan(con, "nums")
    out.append("count %d scan %d" % (len(rows), len(rows)))
    out.append("sum %d" % sum(r[2] for r in rows))
    for r in rows[:3] + rows[-2:]:
        out.append("%d|%s" % (r[0], "|".join(show(v) for v in r[1:])))
    for i in [1, 2, 3, 6, 7, 8, 50, 97, 119, 120, 121, 0, -5, 100000]:
        r = con.execute("SELECT rowid, * FROM nums WHERE rowid=?", (i,)).fetchone()
        if r is None:
            out.append("get %d: none" % i)
        else:
            out.append("get %d: %s" % (i, "|".join(show(v) for v in r[1:])))
    ent = index_entries(con, "idx_label", "nums")
    out.append("index entries %d" % len(ent))
    for vals, rid in ent[:3]:
        out.append("%s -> %d" % ("|".join(show(v) for v in vals), rid))
    for label in ["label_0042", "label_0000", "label_9999"]:
        ids = [rid for vals, rid in ent if vals[0] == label]
        out.append("find %s: %s" % (label, ",".join(str(i) for i in ids)))
    loose = scan(con, "loose")
    out.append("loose %d" % len(loose))
    out.append("loose sum %d" % sum(r[1] for r in loose))
    for r in loose[:2] + loose[-1:]:
        out.append("%d|%s" % (r[0], "|".join(show(v) for v in r[1:])))
    r = con.execute("SELECT rowid, * FROM loose WHERE rowid=200").fetchone()
    out.append("get loose 200: %s" % "|".join(show(v) for v in r[1:]))
    out.append("get loose 201: %s" % ("none" if con.execute("SELECT 1 FROM loose WHERE rowid=201").fetchone() is None else "some"))
    con.close()
    return out


def crc(b):
    return zlib.crc32(b) & 0xFFFFFFFF


def overflow_out(path):
    con = sqlite3.connect(path)
    out = []
    size = con.execute("PRAGMA page_size").fetchone()[0]
    pages = con.execute("PRAGMA page_count").fetchone()[0]
    out.append("page size %d pages %d" % (size, pages))
    rows = scan(con, "docs")
    out.append("rows %d" % len(rows))
    for rid, _id, body, data in rows:
        data = data if data is not None else b""
        eb = body.encode()
        out.append("%d %d %d %d %d" % (rid, len(eb), crc(eb), len(data), crc(data)))
    ent = index_entries(con, "idx_body", "docs")
    out.append("index entries %d" % len(ent))
    for vals, rid in ent:
        eb = vals[0].encode()
        out.append("%d %d %d" % (rid, len(eb), crc(eb)))
    con.close()
    return out


# ---------------------------------------------------------------- the reader, ported


class Reader:
    """The algorithm of std/sqlite/sqlite.ail, over Python bytes."""

    def __init__(self, data):
        assert data[:16] == b"SQLite format 3\x00"
        ps = struct.unpack(">H", data[16:18])[0]
        self.ps = 65536 if ps == 1 else ps
        self.u = self.ps - data[20]
        self.data = data
        self.npages = len(data) // self.ps

    def varint(self, b, at):
        v = 0
        for i in range(8):
            c = b[at + i]
            v = (v << 7) | (c & 127)
            if c < 128:
                return v, i + 1
        v = (v << 8) | b[at + 8]
        if v >= 1 << 63:
            v -= 1 << 64
        return v, 9

    def page(self, p):
        start = (p - 1) * self.ps
        hdr = start + 100 if p == 1 else start
        ty = self.data[hdr]
        assert ty in (2, 5, 10, 13), (p, ty)
        n = struct.unpack(">H", self.data[hdr + 3:hdr + 5])[0]
        right, ptrs = 0, hdr + 8
        if ty in (2, 5):
            right = struct.unpack(">I", self.data[hdr + 8:hdr + 12])[0]
            ptrs = hdr + 12
        return start, ty, n, right, ptrs

    def cell(self, pg, i):
        start, ty, n, right, ptrs = pg
        return start + struct.unpack(">H", self.data[ptrs + 2 * i:ptrs + 2 * i + 2])[0]

    def payload(self, at, size, is_table):
        u = self.u
        x = u - 35 if is_table else (u - 12) * 64 // 255 - 23
        if size <= x:
            return self.data[at:at + size]
        m = (u - 12) * 32 // 255 - 23
        k = m + (size - m) % (u - 4)
        local = k if k <= x else m
        parts = [self.data[at:at + local]]
        left = size - local
        nxt = struct.unpack(">I", self.data[at + local:at + local + 4])[0]
        while left > 0:
            assert nxt != 0
            start = (nxt - 1) * self.ps
            take = min(left, u - 4)
            parts.append(self.data[start + 4:start + 4 + take])
            left -= take
            nxt = struct.unpack(">I", self.data[start:start + 4])[0]
        return b"".join(parts)

    def decode(self, rec):
        hlen, n = self.varint(rec, 0)
        types, at = [], n
        while at < hlen:
            t, k = self.varint(rec, at)
            types.append(t)
            at += k
        out, pos = [], hlen
        for t in types:
            if t == 0:
                out.append(None)
            elif t in (1, 2, 3, 4, 5, 6):
                w = {1: 1, 2: 2, 3: 3, 4: 4, 5: 6, 6: 8}[t]
                out.append(int.from_bytes(rec[pos:pos + w], "big", signed=True))
                pos += w
                continue
            elif t == 7:
                out.append(struct.unpack(">d", rec[pos:pos + 8])[0])
                pos += 8
                continue
            elif t == 8:
                out.append(0)
            elif t == 9:
                out.append(1)
            else:
                w = (t - 12) // 2
                chunk = rec[pos:pos + w]
                out.append(chunk if t % 2 == 0 else chunk.decode("utf-8"))
                pos += w
        return out

    def table(self, p):
        pg = self.page(p)
        start, ty, n, right, ptrs = pg
        rows = []
        if ty == 13:
            for i in range(n):
                at = self.cell(pg, i)
                sz, a = self.varint(self.data, at)
                rid, b = self.varint(self.data, at + a)
                rows.append((rid, self.decode(self.payload(at + a + b, sz, True))))
        elif ty == 5:
            for i in range(n):
                at = self.cell(pg, i)
                rows += self.table(struct.unpack(">I", self.data[at:at + 4])[0])
            rows += self.table(right)
        else:
            raise AssertionError("index page in table")
        return rows

    def index(self, p):
        pg = self.page(p)
        start, ty, n, right, ptrs = pg
        rows = []
        if ty == 10:
            for i in range(n):
                at = self.cell(pg, i)
                sz, a = self.varint(self.data, at)
                rows.append(self.decode(self.payload(at + a, sz, False)))
        elif ty == 2:
            for i in range(n):
                at = self.cell(pg, i)
                rows += self.index(struct.unpack(">I", self.data[at:at + 4])[0])
                sz, a = self.varint(self.data, at + 4)
                rows.append(self.decode(self.payload(at + 4 + a, sz, False)))
            rows += self.index(right)
        else:
            raise AssertionError("table page in index")
        return rows

    def depth(self, p):
        start, ty, n, right, ptrs = self.page(p)
        return 1 if ty in (10, 13) else 1 + self.depth(right)


def check_port(path):
    """The port must agree with SQLite on every table and index of the file."""
    data = open(path, "rb").read()
    rd = Reader(data)
    con = sqlite3.connect(path)
    schema = rd.table(1)
    names = {(r[1][0], r[1][1]) for r in schema}
    sq = {(t, n) for t, n in con.execute("SELECT type, name FROM sqlite_master")}
    assert names == sq, (names, sq)
    deepest = 0
    for _, v in schema:
        kind, name, tbl, root = v[0], v[1], v[2], v[3]
        if kind == "table":
            got = rd.table(root)
            want = scan(con, name)
            assert [g[0] for g in got] == [w[0] for w in want], name
            deepest = max(deepest, rd.depth(root))
            ipk = [r[1] for r in con.execute("PRAGMA table_info(%s)" % qi(name)) if r[5] and r[2].upper() == "INTEGER"]
            for (rid, vals), w in zip(got, want):
                for i, c in enumerate(vals):
                    wv = w[1 + i]
                    if c is None and ipk and columns(con, name)[i] == ipk[0]:
                        c = rid
                    if isinstance(wv, float) and isinstance(c, int):
                        c = float(c)
                    assert c == wv and type(c) == type(wv), (name, rid, i, c, wv)
        elif kind == "index":
            got = rd.index(root)
            want = index_entries(con, name, tbl)
            assert len(got) == len(want), name
            for g, (vals, rid) in zip(got, want):
                assert g[-1] == rid and g[:-1] == vals, (name, g, vals, rid)
            deepest = max(deepest, rd.depth(root))
    con.close()
    return deepest


def split_defs(sql):
    defs, cur, depth, quote = [], [], 0, " "
    for c in sql:
        if quote != " ":
            if depth > 0:
                cur.append(c)
            if c == quote:
                quote = " "
        elif c in "\"'`":
            quote = c
            if depth > 0:
                cur.append(c)
        elif c == "[":
            quote = "]"
            if depth > 0:
                cur.append(c)
        elif c == "(":
            depth += 1
            if depth > 1:
                cur.append(c)
        elif c == ")":
            depth -= 1
            if depth == 0:
                defs.append("".join(cur))
                return defs
            cur.append(c)
        elif c == "," and depth == 1:
            defs.append("".join(cur))
            cur = []
        elif depth > 0:
            cur.append(c)
    return defs


STOP = {"PRIMARY", "NOT", "NULL", "UNIQUE", "CHECK", "DEFAULT", "COLLATE", "REFERENCES", "GENERATED", "AS", "CONSTRAINT"}


def real_affinity(words):
    ty = ""
    for w in words:
        if w in STOP:
            break
        ty = ty + " " + w
    if any(k in ty for k in ("INT", "CHAR", "CLOB", "TEXT", "BLOB")):
        return False
    return any(k in ty for k in ("REAL", "FLOA", "DOUB"))


def parse_col(one):
    cs = one.strip()
    if not cs:
        return None
    first = cs[0]
    if first in "\"`['":
        close = "]" if first == "[" else first
        j = 1
        while j < len(cs) and cs[j] != close:
            j += 1
        name, rest = cs[1:j], cs[(j + 1 if j < len(cs) else j):]
    else:
        j = 0
        while j < len(cs) and not cs[j].isspace() and cs[j] != "(":
            j += 1
        name, rest = cs[:j], cs[j:]
        if name.upper() in ("CONSTRAINT", "PRIMARY", "UNIQUE", "CHECK", "FOREIGN"):
            return None
    words = rest.upper().split()
    alias = len(words) > 2 and words[0] == "INTEGER" and any(
        words[i] == "PRIMARY" and words[i + 1] == "KEY" for i in range(1, len(words) - 1))
    return name, alias, real_affinity(words)


def check_columns(path):
    con = sqlite3.connect(path)
    for name, sql in con.execute("SELECT name, sql FROM sqlite_master WHERE type='table'"):
        cols = [c for c in (parse_col(d) for d in split_defs(sql)) if c]
        info = list(con.execute("PRAGMA table_info(%s)" % qi(name)))
        assert [c[0] for c in cols] == [r[1] for r in info], (name, cols, info)
        for c, r in zip(cols, info):
            decl = r[2].upper()
            want_real = decl != "" and not any(k in decl for k in ("INT", "CHAR", "CLOB", "TEXT", "BLOB")) and any(
                k in decl for k in ("REAL", "FLOA", "DOUB"))
            assert c[2] == want_real, (name, c, decl)
            assert c[1] == (r[5] == 1 and decl == "INTEGER"), (name, c, r)
    con.close()


def write_out(name, lines):
    with open(os.path.join(CASES, name), "w", encoding="utf-8", newline="\n") as f:
        f.write("\n".join(lines) + "\n")


def main():
    if "--tiny" in sys.argv:
        print(tiny_ail(make_tiny()))
        return
    basic, big, over = make_basic(), make_big(), make_overflow()
    for p in (basic, big, over):
        d = check_port(p)
        check_columns(p)
        print("%s: port agrees with sqlite, deepest b-tree %d levels" % (os.path.basename(p), d))
    assert check_port(big) >= 3, "big.db must have an interior level above interior pages"
    write_out("std_sqlite.out", basic_out(basic))
    write_out("std_sqlite_big.out", big_out(big))
    write_out("std_sqlite_overflow.out", overflow_out(over))
    print("sqlite", sqlite3.sqlite_version)


if __name__ == "__main__":
    sys.exit(main())
