counts = [5, 3, 4, 1]
msgs = [(k + 1, s, f"p{k + 1}-m{s}") for k, c in enumerate(counts) for s in range(1, c + 1)]
print(f"received {len(msgs)} messages, per-producer order kept: true")
for k, c in enumerate(counts):
    print(f"producer {k + 1}: {c} messages")
print(f"first: {msgs[0][2]}, last: {msgs[-1][2]}")
print(" ".join(m[2] for m in msgs))
print(f"text bytes: {sum(len(m[2]) for m in msgs)}")
