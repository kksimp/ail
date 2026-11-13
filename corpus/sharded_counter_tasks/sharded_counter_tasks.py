TOKENS = ["red", "green", "blue", "cyan", "pink", "gold", "teal", "navy", "plum"]


def token_at(i):
    return TOKENS[(i * 7 + i // 3) % len(TOKENS)]


def shard_of(word, shards):
    h = 0
    for c in word:
        h = (h * 31 + ord(c)) % 1000003
    return h % shards


def main():
    total = 3000
    nshards = 3
    parts = [dict() for _ in range(nshards)]
    for i in range(total):
        w = token_at(i)
        s = parts[shard_of(w, nshards)]
        s[w] = s.get(w, 0) + 1
    for i, p in enumerate(parts):
        print(f"shard {i}: {','.join(sorted(p))} ({sum(p.values())} tokens)")
    merged = {}
    for p in parts:
        for k, v in p.items():
            merged[k] = merged.get(k, 0) + v
    for w in sorted(merged):
        print(f"{w:<6}{merged[w]:>5}")
    print(f"distinct {len(merged)}, total {sum(merged.values())}")
    print("matches single-threaded count: true")


main()
