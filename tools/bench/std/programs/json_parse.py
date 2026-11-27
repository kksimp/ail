import json, sys
n = int(sys.argv[1]) if len(sys.argv) > 1 else 50000
parts = []
for i in range(n):
    flag = "true" if i % 3 == 0 else "false"
    parts.append('{"id":%d,"name":"item-%d","tags":["a","b%d"],"ok":%s}' % (i, i, i % 7, flag))
text = "[" + ",".join(parts) + "]"
def total(doc):
    return sum(x["id"] for x in doc), sum(1 for x in doc if x["ok"])
doc = json.loads(text)
ids, oks = total(doc)
ids2, oks2 = total(json.loads(json.dumps(doc)))
print("records %d, id sum %d, ok %d, round trip %s" % (n, ids, oks, "true" if (ids, oks) == (ids2, oks2) else "false"))
