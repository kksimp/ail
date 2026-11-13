def main():
    scored = []
    for p in range(3):
        for s in range(5):
            value = (p * 31 + s * 17) % 50
            scored.append((p, s, value * value + 1))
    scored.sort(key=lambda t: t[0] * 1000 + t[1])
    for p, s, score in scored:
        print(f"p{p}#{s} -> {score}")
    totals = [0, 0, 0]
    for p, s, score in scored:
        totals[p] += score
    for p in range(3):
        print(f"producer {p} total {totals[p]}")
    print(f"items: {len(scored)}, grand total: {sum(totals)}")


main()
