import sqlite3, sys
n = int(sys.argv[1]) if len(sys.argv) > 1 else 50000
d = sqlite3.connect(":memory:")
d.execute("CREATE TABLE items (id INTEGER PRIMARY KEY, grp INTEGER, val INTEGER, name TEXT)")
seed = 3
rows = []
for i in range(n):
    seed = (seed * 3877 + 29573) % 139968
    rows.append((i, seed % 50, seed % 1000, "item-%d" % i))
with d:
    for r in rows:
        d.execute("INSERT INTO items (id, grp, val, name) VALUES (?, ?, ?, ?)", r)
agg = d.execute("SELECT grp, COUNT(*) AS c, SUM(val) AS s FROM items GROUP BY grp ORDER BY grp").fetchall()
found = nc = 0
for k in range(n // 10):
    for (name,) in d.execute("SELECT name FROM items WHERE id = ?", (k * 10,)):
        found += 1
        nc += len(name)
print("groups %d, rows %d, val sum %d, found %d, name chars %d" % (len(agg), sum(a[1] for a in agg), sum(a[2] for a in agg), found, nc))
