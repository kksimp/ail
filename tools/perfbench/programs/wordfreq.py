# Python twin of wordfreq.ail (CPython 3, one thread).
import sys

WORDS = ["the", "of", "and", "to", "in", "fox", "jumps", "lazy", "dog", "river", "stone", "moss"]


def main():
    n = int(sys.argv[1]) if len(sys.argv) > 1 else 200000
    counts = {}
    seed = 7
    for _ in range(n):
        seed = (seed * 3877 + 29573) % 139968
        w = WORDS[seed % len(WORDS)]
        counts[w] = counts.get(w, 0) + 1
    for w in WORDS:
        print(f"{w} {counts.get(w, 0)}")
    print(f"distinct {len(counts)}")


main()
