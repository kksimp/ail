def clean(text):
    return "".join(ch.lower() if ch.isalpha() else " " for ch in text)


def count_words(text):
    counts = {}
    for w in clean(text).split():
        counts[w] = counts.get(w, 0) + 1
    return counts


def main():
    docs = [
        "The cat sat on the mat. The mat was warm.",
        "A dog and a cat met; the dog barked!",
        "Warm mats, warm cats, and a warm dog.",
        "",
        "On the mat: a cat, a dog, and the sun.",
    ]
    counts = {}
    for d in docs:
        for w, n in count_words(d).items():
            counts[w] = counts.get(w, 0) + n
    ranked = sorted(sorted(counts), key=lambda w: -counts[w])
    for w in ranked[:6]:
        print(f"{w:<6}{counts[w]:>3}")
    print(f"distinct: {len(counts)}, total: {sum(counts.values())}")
    singles = [w for w in sorted(counts) if counts[w] == 1]
    print(f"seen once: {' '.join(singles)}")


main()
