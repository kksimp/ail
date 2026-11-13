def normalize(word):
    return "".join(c for c in word.lower() if c.isalpha())


def build_index(names, texts):
    index = {}
    for name, text in sorted(zip(names, texts)):
        seen = set()
        for raw in text.split():
            w = normalize(raw)
            if len(w) >= 2 and w not in seen:
                seen.add(w)
                index.setdefault(w, []).append(name)
    return index


def search(index, terms, need_all):
    scores = {}
    for t in terms:
        for name in index.get(normalize(t), []):
            scores[name] = scores.get(name, 0) + 1
    hits = [n for n in sorted(scores) if not need_all or scores[n] == len(terms)]
    return sorted(hits, key=lambda n: -scores[n])


def show(index, terms, need_all):
    mode = "all" if need_all else "any"
    hits = search(index, terms, need_all)
    print(f"{mode} of {' '.join(terms)}: {', '.join(hits)} ({len(hits)})")


def py_list(xs):
    return "[" + ", ".join(f'"{x}"' for x in xs) + "]"


def main():
    names = ["cats.txt", "dogs.txt", "pets.txt", "zoo.txt", "farm.txt"]
    texts = [
        "Cats sleep a lot. Cats purr, and cats hunt mice.\n",
        "Dogs bark and dogs fetch; some dogs hunt too.\n",
        "Pets: cats, dogs, fish. Fish sleep little!\n",
        "The zoo has lions and tigers; tigers hunt at night.\n",
        "A farm has dogs, cows and mice. Mice are small.\n",
    ]
    index = build_index(names, texts)
    print(f"{len(index)} words indexed from {len(names)} files")
    for w in ["hunt", "sleep", "mice", "tigers"]:
        print(f"{w:<7}{','.join(index.get(w, []))}")
    show(index, ["cats", "dogs"], True)
    show(index, ["cats", "dogs"], False)
    show(index, ["hunt", "mice", "fish"], False)
    show(index, ["hunt", "mice"], True)
    show(index, ["unicorn"], False)
    print("cleaned up: true")


main()
