# Python twin of examples/std_csv.ail: quoted fields in, rows over 10 units out.
import csv
import io

text = 'region,product,units\nnorth,"bolts, steel",24\nsouth,washers,5\nnorth,nuts,11\neast,"gear ""XL""",12\n'

recs = list(csv.DictReader(io.StringIO(text)))
big = []
for r in recs:
    units = int(r["units"]) if r["units"].lstrip("-").isdigit() else 0
    if units > 10:
        big.append(r)

out = io.StringIO()
w = csv.DictWriter(out, fieldnames=["region", "product", "units"], lineterminator="\n")
w.writeheader()
w.writerows(big)
print(out.getvalue())
print(f"{len(big)} of {len(recs)} rows")
