import json

items = [{"id": float(i), "name": f"item-{i}", "tags": ["a", f"b{i % 7}"], "ok": i % 3 == 0} for i in range(20000)]
text = json.dumps(items, separators=(",", ":"))
doc = json.loads(text)
total = 0.0
oks = 0
for item in doc:
    total += item["id"]
    if item["ok"]:
        oks += 1
print(f"{len(text)} chars, id sum {total}, ok {oks}")
