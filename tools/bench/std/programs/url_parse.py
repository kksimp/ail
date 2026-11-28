import sys
from urllib.parse import urlsplit
n = int(sys.argv[1]) if len(sys.argv) > 1 else 100000
ports = chars = 0
for i in range(n):
    u = urlsplit("https://host%d.example.com:%d/a/b%d?q=%d&r=x#f%d" % (i % 50, 8000 + i % 100, i, i, i))
    ports += u.port or 0
    chars += len(u.hostname) + len(u.path) + len(u.query) + len(u.fragment)
print("urls %d, ports %d, chars %d" % (n, ports, chars))
