import json

DOCS = """{"name": "api", "port": 8080, "workers": 8, "level": "info", "debug": false}
{"name": "", "port": 70000, "workers": 0, "level": "loud"}
{"name": "web", "port": "80", "workers": 4}
{"name": "x",
{"name": "batch", "port": 9000, "workers": 64, "level": "debug"}
"""

LEVELS = ["debug", "info", "warn", "error"]


def get_int(doc, key):
    v = doc.get(key)
    if isinstance(v, int) and not isinstance(v, bool):
        return v
    return -1


def validate(text):
    try:
        doc = json.loads(text)
    except ValueError:
        return ["not valid json"]
    problems = []
    name = doc.get("name")
    if not isinstance(name, str) or name == "":
        problems.append("name: required, non-empty text")
    port = get_int(doc, "port")
    if port < 1 or port > 65535:
        problems.append("port: need an integer in 1..65535")
    workers = get_int(doc, "workers")
    if workers < 1 or workers > 64:
        problems.append("workers: need an integer in 1..64")
    level = doc.get("level", "info")
    if level not in LEVELS:
        problems.append(f"level: '{level}' is not one of {'|'.join(LEVELS)}")
    return problems


def main():
    bad = 0
    n = 0
    for text in DOCS.splitlines():
        n += 1
        problems = validate(text)
        if not problems:
            print(f"doc {n}: ok")
        else:
            bad += 1
            print(f"doc {n}: {len(problems)} problems")
            for p in problems:
                print(f"  - {p}")
    print(f"{n - bad} of {n} documents valid")


main()
