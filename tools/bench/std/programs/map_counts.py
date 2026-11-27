import sys
n = int(sys.argv[1]) if len(sys.argv) > 1 else 1000000
seed = 5
a, b = {}, {}
for _ in range(n):
    seed = (seed * 3877 + 29573) % 139968
    k = seed % 50021
    a[k] = a.get(k, 0) + 1
    key = "key%d" % (seed % 20011)
    b[key] = b.get(key, 0) + 1
sq = sum(v * v + k % 7 for k, v in a.items())
sq2 = sum(v * v + len(k) for k, v in b.items())
print("int keys %d, check %d" % (len(a), sq))
print("str keys %d, check %d" % (len(b), sq2))
