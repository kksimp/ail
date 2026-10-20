import json
import sys
import urllib.error
import urllib.request
from pathlib import Path


def load(url):
    try:
        with urllib.request.urlopen(url) as r:
            return r.read().decode()
    except urllib.error.URLError as err:
        kind = "refused" if isinstance(err.reason, ConnectionRefusedError) else "other"
        print(f"offline ({kind}), using the local copy")
        return (Path(__file__).resolve().parents[5] / "examples/agent-data/status.json").read_text()


def main():
    url = sys.argv[1] if len(sys.argv) > 1 else "http://127.0.0.1:1/status.json"
    doc = json.loads(load(url))
    down = 0
    for svc in doc["services"]:
        name = svc["name"]
        ms = float(svc["ms"])
        if svc["up"]:
            print(f"{name:<8}up    {ms}ms")
        else:
            down += 1
            print(f"{name:<8}DOWN")
    print(f"{down} down")


if __name__ == "__main__":
    main()
