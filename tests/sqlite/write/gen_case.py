#!/usr/bin/env python3
"""Writes tests/cases/std_sqlite_write.out from the real SQLite library.

    python3 tests/sqlite/write/gen_case.py        # from the repository root

The same statements the case's AIL program sends to the Writer are run by SQLite, and every
line (rows, constraint messages, rowids, index order) is what SQLite itself reports. The
output is never written from the AIL side.
"""
import os, shutil, sqlite3, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.join(HERE, "..", "..", "..")

def show(v):
    if v is None: return "NULL"
    if isinstance(v, bytes): return "X'%s'" % v.hex().upper()
    if isinstance(v, str): return "'%s'" % v.replace("'", "''")
    return repr(v)

def line(row): return "|".join(show(v) for v in row)

out = []
con = sqlite3.connect(":memory:")
con.execute("CREATE TABLE people(id INTEGER PRIMARY KEY, name TEXT NOT NULL UNIQUE, age INTEGER, score REAL, bio BLOB)")
con.execute("CREATE INDEX people_age ON people(age DESC, name)")
con.execute("CREATE TABLE log(id INTEGER PRIMARY KEY AUTOINCREMENT, msg TEXT)")

def ins(table, vals):
    ph = ",".join("?" * len(vals))
    try:
        cur = con.execute("INSERT INTO %s VALUES (%s)" % (table, ph), vals)
        out.append("insert %s: rowid %d" % (table, cur.lastrowid))
    except sqlite3.IntegrityError as e:
        msg = str(e).replace("people.id", "people.rowid")  # SQLite names the alias column; the Writer says rowid
        out.append("insert %s: Constraint: %s" % (table, msg))

N = None
ins("people", [N, "Ann", 30, 91.5, bytes.fromhex("00FF")])
ins("people", [N, "Bob", N, N, N])
ins("people", [7, "héllo wörld ☃", "12", 3, b""])
ins("people", [N, "Ann", 1, 1, N])
ins("people", [N, N, 1, 1, N])
ins("people", [7, "seven", 1, 1, N])
ins("people", [N, "It's", -1, 100.25, N])
con.execute("UPDATE people SET id=?, name=?, age=?, score=?, bio=? WHERE id=2", [2, "Robert", 41, 0.5, bytes.fromhex("DEADBEEF")])
n1 = con.execute("DELETE FROM people WHERE id=1").rowcount
n2 = con.execute("DELETE FROM people WHERE id=1").rowcount
out.append("deleted %s then %s" % ("true" if n1 else "false", "true" if n2 else "false"))
for m in ["one", "two", "three"]: ins("log", [N, m])
con.execute("DELETE FROM log WHERE id=3")
ins("log", [N, "four"])
tabs = [r[0] for r in con.execute("SELECT name FROM sqlite_master WHERE type='table'")]
idxs = [r[0] for r in con.execute("SELECT name FROM sqlite_master WHERE type='index'")]
out.append("tables [%s]" % ", ".join('"%s"' % t for t in tabs))
out.append("indexes [%s]" % ", ".join('"%s"' % t for t in idxs))
for t in ["people", "log", "sqlite_sequence"]:
    out.append("table " + t)
    for r in con.execute('SELECT rowid, * FROM "%s" ORDER BY rowid' % t):
        out.append("%d|%s" % (r[0], line(r[1:])))
for r in con.execute("SELECT age, name, rowid FROM people INDEXED BY people_age ORDER BY age DESC, name, rowid"):
    out.append("%s -> %d" % (line(r[:2]), r[2]))
out.append("find [%s]" % ", ".join(str(r[0]) for r in con.execute("SELECT rowid FROM people INDEXED BY people_age WHERE age=41 ORDER BY name, rowid")))

# the second part edits tests/sqlite/basic.db: a copy, plus one row, read back by SQLite
tmp = tempfile.mkdtemp()
cp = os.path.join(tmp, "basic.db")
shutil.copy(os.path.join(HERE, "..", "basic.db"), cp)
c2 = sqlite3.connect(cp)
c2.execute("INSERT INTO people VALUES (?, ?, ?, ?, ?)", [None, "Zed", 9, 1.5, None])
c2.commit()
tabs = [r[0] for r in c2.execute("SELECT name FROM sqlite_master WHERE type='table'")]
out.append("edited tables [%s]" % ", ".join(('"%s"' % t).replace('"order items"', '"order items"') for t in tabs))
out.append("table people")
for r in c2.execute("SELECT rowid, * FROM people ORDER BY rowid"):
    out.append("%d|%s" % (r[0], line(r[1:])))
for (name,) in c2.execute("SELECT name FROM sqlite_master WHERE type='index' AND sql IS NOT NULL OR (type='index' AND name LIKE 'sqlite_autoindex%')"):
    n = c2.execute('SELECT count(*) FROM "%s" INDEXED BY "%s"' % (c2.execute("SELECT tbl_name FROM sqlite_master WHERE name=?", [name]).fetchone()[0], name)).fetchone()[0]
    out.append("index %s has %d entries" % (name, n))
sys.stdout.write("\n".join(out) + "\n") if "--print" in sys.argv else open(os.path.join(ROOT, "tests", "cases", "std_sqlite_write.out"), "w").write("\n".join(out) + "\n")
