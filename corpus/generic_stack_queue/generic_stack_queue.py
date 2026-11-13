def reverse_digits(n):
    st = []
    rest = n
    while rest > 0:
        st.append(rest % 10)
        rest //= 10
    out = 0
    place = 1
    while st:
        out += st.pop() * place
        place *= 10
    return out


for n in [12345, 907, 1000]:
    print(f"reverse {n} -> {reverse_digits(n)}")
words = []
for w in "to be or not".split(" "):
    words.append(w)
print(f"stack size {len(words)}, top {words[-1]}")
rota = ["ann", "bo", "cy"]
head = 0
rnd = 1
while rnd <= 7:
    if head >= len(rota):
        break
    who = rota[head]
    head += 1
    print(f"round {rnd}: {who}")
    if rnd <= 4:
        rota.append(f"{who}'")
    rnd += 1
print(f"left in queue: {len(rota) - head}")
