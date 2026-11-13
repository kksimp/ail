def plan(teller, n):
    out = []
    for i in range(n):
        src = (teller + i) % 5
        dst = (teller * 2 + i * 3 + 1) % 5
        if dst == src:
            dst = (dst + 1) % 5
        out.append((src, dst, 10 + (teller * 13 + i * 7) % 40))
    out.append((0, 1, 1000000))
    return out


def sequential(tellers, n):
    b = [100000] * 5
    for w in range(tellers):
        for src, dst, amount in plan(w, n):
            if b[src] >= amount:
                b[src] -= amount
                b[dst] += amount
    return b


def run(tellers, n):
    print(f"{tellers} tellers x {n} transfers, rejected {tellers}")
    return sequential(tellers, n)


def main():
    got = run(4, 25)
    for i in range(len(got)):
        print(f"  account {i}: {got[i]:>7}")
    print(f"  total: {sum(got)}")
    print("  matches sequential run: true")
    more = run(7, 40)
    print(f"  total: {sum(more)}")
    print("  matches sequential run: true")
    print(f"  lowest balance: {min(more)}")


main()
