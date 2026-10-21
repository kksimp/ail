import json, urllib.request
r = urllib.request.urlopen("https://example.com/api")
print(json.loads(r.read())["name"])
