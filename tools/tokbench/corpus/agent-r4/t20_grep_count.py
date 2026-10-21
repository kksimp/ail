import re, sys
pat = re.compile(r"ERROR (\d+)")
n = sum(1 for l in open(sys.argv[1]) if pat.search(l))
print(n)
