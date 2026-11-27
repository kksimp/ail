import hashlib, hmac, sys
n = int(sys.argv[1]) if len(sys.argv) > 1 else 200000
h, m = "seed", "key"
for i in range(n):
    h = hashlib.sha256(("%s%d" % (h, i)).encode()).hexdigest()
    m = hmac.new(h[:16].encode(), ("%s%d" % (m, i)).encode(), hashlib.sha256).hexdigest()
print(h)
print(m)
big = "0123456789abcdef" * 65536
d = ""
for _ in range(n // 10000):
    d = hashlib.sha256((d + big).encode()).hexdigest()
print(d)
