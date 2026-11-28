#!/usr/bin/env python3
"""std.sqlite writer check against real SQLite (std.sqlite.15-21).

For each scenario it makes a seeded list of operations, applies it to the SQLite library
(the reference) and to `runner.ail` (the AIL Writer), then checks that: every operation
has the same outcome (ok or a constraint failure); the file the Writer wrote passes
`PRAGMA integrity_check` (this checks every index against its table); every table and
`sqlite_sequence` holds the same typed rows as the reference; indexes answer the same
lookups; and a reopen plus `edit()` plus `bytes()` gives a file with the same content.

  python3 tests/sqlite/write/verify.py RUNNER_BINARY EDIT_BINARY [scenario ...]

EDIT_BINARY (edit.ail) also reopens every fixture of tests/sqlite/*.db made by SQLite and
checks the rewritten file against the original the same way.
"""
import os, random, sqlite3, subprocess, sys, tempfile

def enc(v):
    if v is None: return "N"
    if isinstance(v, bool): v = int(v)
    if isinstance(v, int): return "i:%d" % v
    if isinstance(v, float): return "r:%r" % v
    if isinstance(v, str): return "s:" + v.encode().hex()
    return "b:" + bytes(v).hex()

def rt(s, n=20):  # random text
    return "".join(random.choice("abcdefghij klé中'\"%") for _ in range(random.randint(0, n)))

def big(lo, hi):
    return bytes(random.getrandbits(8) for _ in range(random.randint(lo, hi)))

# kind -> value maker
def mk(kind):
    r = random.random()
    if kind == "int":
        return random.choice([0, 1, -1, 127, 128, -129, 32767, -32768, 8388607, -8388608, 2**31 - 1, -2**31,
                              2**47 - 1, -2**47, 2**62, -2**63, 2**63 - 1, random.randint(-1000, 1000)]) if r < .5 else random.randint(-10**6, 10**6)
    if kind == "intnull": return None if r < .2 else mk("int")
    if kind == "real": return random.choice([0.0, 1.5, -2.25, 1e300, 1e-300, 3.0, float(random.randint(-9, 9))]) if r < .4 else random.uniform(-1e6, 1e6)
    if kind == "text": return random.choice(["", "a", "it's", "12", "007", "3.5", "x" * 300]) if r < .4 else rt("", 40)
    if kind == "textnull": return None if r < .2 else mk("text")
    if kind == "bigtext": return "".join(random.choice("abcdef é") for _ in range(random.randint(0, 9000))) if r < .5 else rt("", 30)
    if kind == "blob": return big(0, 10) if r < .5 else big(0, 5000)
    if kind == "bigblob": return big(3000, 20000) if r < .3 else big(0, 300)
    if kind == "any":
        return random.choice([None, mk("int"), mk("real"), mk("text"), big(0, 40)])
    if kind == "anytext":  # for TEXT-affinity columns: a REAL there becomes text, which SQLite spells its own way (std.sqlite.18)
        return random.choice([None, mk("int"), mk("text"), big(0, 40)])
    raise ValueError(kind)

SCENARIOS = {}
def scenario(f):
    SCENARIOS[f.__name__] = f
    return f

def ins(table, kinds, alias=None, rid=None, n=1, nulls=()):
    ops = []
    for _ in range(n):
        vals = [mk(k) for k in kinds]
        if alias is not None:
            vals[alias] = rid() if rid else None
        ops.append(("INS", table, vals))
    return ops

@scenario
def basic():
    ddl = ["CREATE TABLE t(id INTEGER PRIMARY KEY, name TEXT NOT NULL, score REAL, n INTEGER, data BLOB)",
           "CREATE INDEX t_name ON t(name)", "CREATE INDEX t_ns ON t(n DESC, name)"]
    ops = []
    for _ in range(300):
        ops.append(("INS", "t", [None, mk("text"), mk("real"), mk("intnull"), mk("blob")]))
    ops.append(("INS", "t", [None, None, 1.0, 1, b""]))           # NOT NULL fails
    ops.append(("INS", "t", [5, "dup", 1.0, 1, b""]))             # rowid taken
    ops.append(("INS", "t", [10**9, "far", 2, "12", b""]))        # explicit rowid, text to integer
    ops.append(("INS", "t", [-5, "neg", 2.0, 2, b"x"]))
    ops.append(("INS", "t", [None, "after far", 2.0, 2, b"x"]))
    for i in range(1, 60, 3): ops.append(("UPD", "t", i, [i, "u%d" % i, 9.5, i, b"u"]))
    for i in range(2, 80, 4): ops.append(("DEL", "t", i))
    ops.append(("UPD", "t", 999999, [999999, "x", 1.0, 1, b""]))
    ops.append(("UPD", "t", 3, [4, "clash", 1.0, 1, b""]))        # move onto a taken rowid
    return ddl, ops, {"t": ["t_name", "t_ns"]}

@scenario
def unique():
    ddl = ["CREATE TABLE u(a TEXT UNIQUE, b INTEGER, c REAL, UNIQUE(b, c))",
           "CREATE TABLE p(k TEXT PRIMARY KEY, v INTEGER NOT NULL)",
           "CREATE TABLE q(id INTEGER, w TEXT, PRIMARY KEY(id))",
           "CREATE UNIQUE INDEX p_v ON p(v)"]
    ops = []
    for i in range(400):
        ops.append(("INS", "u", [random.choice([None, "k%d" % random.randint(0, 150)]), random.randint(0, 5), random.choice([None, 1.5, 2.5])]))
        ops.append(("INS", "p", ["key%d" % random.randint(0, 200), random.randint(0, 250)]))
        ops.append(("INS", "q", [random.choice([None, random.randint(1, 100)]), mk("text")]))
    for i in range(1, 40): ops.append(("UPD", "u", i, ["new%d" % i, 1, None]))
    for i in range(1, 40, 2): ops.append(("DEL", "p", i))
    for i in range(1, 30): ops.append(("INS", "p", ["again%d" % i, i]))
    return ddl, ops, {}

@scenario
def autoinc():
    ddl = ["CREATE TABLE a(id INTEGER PRIMARY KEY AUTOINCREMENT, x TEXT)",
           "CREATE TABLE b(id INTEGER PRIMARY KEY AUTOINCREMENT, y INT)", "CREATE INDEX a_x ON a(x)"]
    ops = []
    for i in range(50):
        ops.append(("INS", "a", [None, mk("text")]))
        ops.append(("INS", "b", [None, i]))
    ops.append(("DEL", "a", 50)); ops.append(("INS", "a", [None, "after delete"]))
    ops.append(("INS", "a", [1000, "jump"])); ops.append(("INS", "a", [None, "next"]))
    ops.append(("INS", "a", [5, "low"]))
    ops.append(("DEL", "b", 50)); ops.append(("DEL", "b", 49)); ops.append(("INS", "b", [None, 7]))
    return ddl, ops, {"a": ["a_x"]}

@scenario
def overflow():
    ddl = ["CREATE TABLE o(id INTEGER PRIMARY KEY, t TEXT, b BLOB, n INTEGER)",
           "CREATE INDEX o_t ON o(t)", "CREATE INDEX o_b ON o(b, n)"]
    ops = []
    for _ in range(150):
        ops.append(("INS", "o", [None, mk("bigtext"), mk("bigblob"), mk("int")]))
    ops.append(("INS", "o", [None, "z" * 100000, big(100000, 100001), 1]))   # 100 KB payloads
    ops.append(("INS", "o", [None, "y" * 4000, big(4000, 4001), 2]))
    for i in range(1, 100, 7): ops.append(("UPD", "o", i, [i, mk("bigtext"), mk("bigblob"), i]))
    for i in range(2, 100, 9): ops.append(("DEL", "o", i))
    return ddl, ops, {"o": ["o_t", "o_b"]}

@scenario
def types():
    ddl = ["CREATE TABLE m(a, b INTEGER, c REAL, d TEXT, e NUMERIC, f BLOB, g VARCHAR(10), h DOUBLE, i INT8)",
           "CREATE INDEX m_all ON m(b, c, d, e)", "CREATE INDEX m_a ON m(a DESC)"]
    ops = []
    for _ in range(500):
        ops.append(("INS", "m", [mk("any"), mk("any"), mk("any"), mk("anytext"), mk("any"), mk("any"), mk("anytext"), mk("any"), mk("any")]))
    ops.append(("INS", "m", [None, "12", "1.5", 77, "3.0", "7", 5, 3, "-9"]))
    ops.append(("INS", "m", ["  5 ", "5e2", "0x10", "1.5", "abc", None, "", 2, 1.0]))
    return ddl, ops, {"m": ["m_all", "m_a"]}

@scenario
def deep():
    ddl = ["CREATE TABLE d(id INTEGER PRIMARY KEY, k INTEGER, s TEXT)",
           "CREATE INDEX d_k ON d(k)", "CREATE INDEX d_s ON d(s)"]
    ops = []
    for i in range(70000):
        ops.append(("INS", "d", [None, random.randint(0, 10**6), "s%06d" % random.randint(0, 99999) + "x" * random.randint(0, 6)]))
    return ddl, ops, {"d": ["d_k", "d_s"]}

@scenario
def deep3():  # a table b-tree three levels deep
    ddl = ["CREATE TABLE d(id INTEGER PRIMARY KEY, k INTEGER)"]
    return ddl, [("INS", "d", [None, i % 97]) for i in range(260000)], {}

def run_ref(ddl, ops, path):
    con = sqlite3.connect(path)
    con.isolation_level = None
    res = []
    for s in ddl: con.execute(s)
    con.execute("BEGIN")
    for op in ops:
        try:
            if op[0] == "INS":
                con.execute("INSERT INTO %s VALUES (%s)" % (op[1], ",".join("?" * len(op[2]))), op[2]); res.append("ok")
            elif op[0] == "UPD":
                cols = [r[1] for r in con.execute("PRAGMA table_info(%s)" % op[1])]
                cur = con.execute("UPDATE %s SET %s WHERE rowid=?" % (op[1], ",".join('"%s"=?' % c for c in cols)), list(op[3]) + [op[2]])
                res.append("ok" if cur.rowcount else "err NoSuch")
            else:
                con.execute("DELETE FROM %s WHERE rowid=?" % op[1], (op[2],)); res.append("ok")
        except sqlite3.IntegrityError:
            res.append("err Constraint")
    con.execute("COMMIT")
    con.close()
    return res

def lines(ddl, ops):
    out = ["SQL\t" + s.encode().hex() for s in ddl]
    for op in ops:
        if op[0] == "INS": out.append("\t".join(["INS", op[1]] + [enc(v) for v in op[2]]))
        elif op[0] == "UPD": out.append("\t".join(["UPD", op[1], str(op[2])] + [enc(v) for v in op[3]]))
        else: out.append("\t".join(["DEL", op[1], str(op[2])]))
    return "\n".join(out) + "\n"

def typed(v):
    return (type(v).__name__, v)

def dump(path, indexes):
    con = sqlite3.connect(path)
    d = {}
    names = [r[0] for r in con.execute("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")]
    for t in names:
        d[t] = [tuple(typed(x) for x in row) for row in con.execute('SELECT rowid, * FROM "%s" ORDER BY rowid' % t)]
    d["#schema"] = sorted((r[0], r[1], r[2]) for r in con.execute("SELECT type, name, tbl_name FROM sqlite_master"))
    d["#integrity"] = con.execute("PRAGMA integrity_check").fetchall()
    d["#fk"] = con.execute("PRAGMA foreign_key_check").fetchall()
    for t, idxs in indexes.items():
        for ix in idxs:
            cols = [r[2] for r in con.execute('PRAGMA index_info("%s")' % ix)]
            d["#ix " + ix] = [tuple(typed(x) for x in row) for row in con.execute(
                'SELECT rowid, %s FROM "%s" INDEXED BY "%s" ORDER BY %s' % (",".join('"%s"' % c for c in cols), t, ix, ",".join('"%s"' % c for c in cols) + ", rowid"))]
    con.close()
    return d

def main():
    runner = sys.argv[1]
    editor = sys.argv[2]
    names = sys.argv[3:] or list(SCENARIOS)
    work = tempfile.mkdtemp(prefix="sqlite-write-")
    bad = 0
    here = os.path.dirname(os.path.abspath(__file__))
    for fx in sorted(os.listdir(os.path.join(here, ".."))):
        if not fx.endswith(".db"):
            continue
        src = os.path.join(here, "..", fx)
        dst = os.path.join(work, "edit-" + fx)
        p = subprocess.run([editor, src, dst], capture_output=True, text=True)
        if p.returncode != 0:
            print("FAIL edit %s: exit %d: %s" % (fx, p.returncode, (p.stdout + p.stderr)[-300:])); bad += 1; continue
        a, b = dump(src, {}), dump(dst, {})
        if b["#integrity"] != [("ok",)] or a != b:
            diff = [k for k in a if a[k] != b.get(k)]
            print("FAIL edit %s: integrity %s, differs in %s" % (fx, b["#integrity"][:2], diff[:4])); bad += 1
        else:
            print("pass edit %s" % fx)
    for name in names:
        random.seed(hash(name) & 0xffff if False else sum(map(ord, name)))
        ddl, ops, indexes = SCENARIOS[name]()
        ref, out1, out2, opsf = (os.path.join(work, name + s) for s in (".ref.db", ".ail.db", ".edit.db", ".ops"))
        for p in (ref,):
            if os.path.exists(p): os.remove(p)
        want = run_ref(ddl, ops, ref)
        open(opsf, "w").write(lines(ddl, ops))
        p = subprocess.run([runner, opsf, out1, out2], capture_output=True, text=True)
        if p.returncode != 0:
            print("FAIL %s: runner exit %d: %s" % (name, p.returncode, (p.stdout + p.stderr)[-400:])); bad += 1; continue
        got = p.stdout.split("\n")[:-1]
        # the runner prints one line per statement; the first len(ddl) are the CREATE lines
        got = got[len(ddl):]
        if got != want:
            diff = [(i, a, b) for i, (a, b) in enumerate(zip(want, got)) if a != b][:3]
            print("FAIL %s: outcomes differ (%d vs %d lines) first %s" % (name, len(want), len(got), diff)); bad += 1; continue
        ok = True
        dr = dump(ref, indexes)
        for label, f in (("write", out1), ("edit", out2)):
            da = dump(f, indexes)
            if da["#integrity"] != [("ok",)]:
                print("FAIL %s %s: integrity_check: %s" % (name, label, da["#integrity"][:3])); ok = False; continue
            for k in dr:
                if k in ("#schema", "#integrity", "#fk"): continue
                if da.get(k) != dr[k]:
                    a, b = da.get(k) or [], dr[k]
                    first = next((i for i in range(max(len(a), len(b))) if (a[i] if i < len(a) else None) != (b[i] if i < len(b) else None)), None)
                    print("FAIL %s %s: %s differs at %s: got %s want %s" % (name, label, k, first, a[first] if first is not None and first < len(a) else None, b[first] if first is not None and first < len(b) else None)); ok = False; break
            if da["#schema"] != dr["#schema"]:
                print("FAIL %s %s: schema differs\n got %s\n want %s" % (name, label, da["#schema"], dr["#schema"])); ok = False
        if ok:
            print("pass %s: %d ops, %d bytes" % (name, len(ops), os.path.getsize(out1)))
        else:
            bad += 1
    sys.exit(1 if bad else 0)

main()
