def sum_odd_squares(n):
    return sum(v * v for v in range(1, n + 1) if v % 2 == 1)


def count_words(texts):
    counts = {}
    for line in texts:
        for w in line.split():
            w = w.lower()
            counts[w] = counts.get(w, 0) + 1
    return counts


def main():
    print(f"odd squares up to 10: {sum_odd_squares(10)}")
    print(f"odd squares up to 100: {sum_odd_squares(100)}")
    texts = [
        "the quick brown fox",
        "jumps over the lazy dog",
        "The dog barks and the fox runs",
        "",
        "quick quick quick",
    ]
    counts = count_words(texts)
    order = sorted(sorted(counts), key=lambda w: -counts[w])
    for w in order[:5]:
        print(f"{w:<6}{counts[w]:>3}")
    print(f"{len(counts)} distinct words, {sum(counts.values())} in total")


main()
