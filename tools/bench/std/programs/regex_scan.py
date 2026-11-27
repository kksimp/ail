import re, sys
n = int(sys.argv[1]) if len(sys.argv) > 1 else 100000
W = ["alpha7", "beta42", "gamma", "delta1234", "eps", "zeta99", "eta12345", "theta", "iota3", "kappa808"]
seed, ws = 11, []
for _ in range(n):
    seed = (seed * 3877 + 29573) % 139968
    ws.append(W[seed % len(W)])
text = " ".join(ws)
found = re.findall(r"[a-z]+[0-9]{2,4}", text)
swapped = re.sub(r"[aeiou]", "_", text)
print("tokens %d, token bytes %d, replaced %d %s" % (len(found), sum(len(m) for m in found), len(swapped), swapped[:30]))
