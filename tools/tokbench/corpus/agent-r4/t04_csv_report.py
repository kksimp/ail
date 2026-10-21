import csv, sys
total = 0
for row in csv.DictReader(open(sys.argv[1])):
    total += int(row["qty"])
print(total)
