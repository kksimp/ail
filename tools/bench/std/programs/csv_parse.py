import csv, io, sys
n = int(sys.argv[1]) if len(sys.argv) > 1 else 100000
text = "\n".join('%d,"name, %d",%d,"say ""hi"" %d"' % (i, i, i % 97, i % 5) for i in range(n)) + "\n"
rows = list(csv.reader(io.StringIO(text, newline="")))
qty = sum(int(r[2]) for r in rows)
chars = sum(len(r[1]) + len(r[3]) for r in rows)
print("rows %d, qty %d, chars %d" % (len(rows), qty, chars))
