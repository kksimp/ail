import gzip, sys, zlib
n = int(sys.argv[1]) if len(sys.argv) > 1 else 400000
W = ["the", "of", "and", "to", "in", "fox", "jumps", "lazy", "dog", "river", "stone", "moss"]
seed, ws = 7, []
for _ in range(n):
    seed = (seed * 3877 + 29573) % 139968
    ws.append(W[seed % len(W)])
text = " ".join(ws).encode()
z = gzip.compress(text)
back = gzip.decompress(z)
print("bytes %d, crc %d, round trip %s" % (len(text), zlib.crc32(text), "true" if back == text else "false"))
