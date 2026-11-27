import json
import sys


def size(at, default):
    if len(sys.argv) <= at + 1:
        return default
    return int(sys.argv[at + 1])


def record(i):
    return {
        "id": i,
        "name": "item %d\t\"q\"\nline é" % i,
        "score": (i % 7) + 0.5,
        "ok": i % 2 == 0,
        "note": None,
        "tags": ["a%d" % (i % 5), "tag é %d" % (i % 11), "x"],
    }


n = size(0, 2000)
reps = size(1, 20)
text = json.dumps([record(i) for i in range(n)], separators=(",", ":"), ensure_ascii=False)
same = 0
for _ in range(reps):
    doc = json.loads(text)
    if json.dumps(doc, separators=(",", ":"), ensure_ascii=False) == text:
        same += 1
print("records %d chars %d same %d" % (n, len(text), same))
