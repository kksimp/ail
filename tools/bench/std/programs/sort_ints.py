import sys
n = int(sys.argv[1]) if len(sys.argv) > 1 else 1000000
seed, xs = 42, []
for _ in range(n):
    seed = (seed * 3877 + 29573) % 139968
    xs.append(seed * 7 + len(xs) % 13)
s = sorted(xs)
chk = 0
for x in s:
    chk = (chk * 31 + x) % 1000000007
ss = []
for j in range(n // 4):
    seed = (seed * 3877 + 29573) % 139968
    ss.append("k%d-%d" % (seed, j % 1000))
o = sorted(ss)
print("ints %d, check %d, first %d, last %d" % (len(s), chk, s[0], s[n - 1]))
print("strings %d, first %s, last %s" % (len(o), o[0], o[-1]))
