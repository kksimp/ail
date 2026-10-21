import sys
from collections import Counter
words = open(sys.argv[1]).read().split()
for w, n in Counter(words).most_common(3):
    print(w, n)
