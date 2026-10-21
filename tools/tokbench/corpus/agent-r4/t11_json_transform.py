import json, sys
data = json.load(open(sys.argv[1]))
out = [{"name": u["name"], "age": u["age"] + 1} for u in data if u["age"] > 18]
print(json.dumps(out))
