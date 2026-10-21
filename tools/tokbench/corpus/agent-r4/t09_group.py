from collections import defaultdict
g = defaultdict(list)
for w in "the cat the dog".split():
    g[len(w)].append(w)
print(dict(g))
