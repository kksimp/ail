from collections import Counter
from pathlib import Path


def field(line, key):
    for part in line.split(" "):
        if part.startswith(f"{key}="):
            return part[len(key) + 1:]
    return None


def main():
    text = (Path(__file__).resolve().parents[5] / "examples/agent-data/app.log").read_text()
    codes = Counter()
    users = Counter()
    errors = 0
    for line in text.splitlines():
        if " ERROR " not in line:
            continue
        errors += 1
        codes[field(line, "code") or "none"] += 1
        users[field(line, "user") or "anon"] += 1
    print(f"{errors} errors")
    for c in sorted(codes):
        print(f"code {c}: {codes[c]}")
    for u in sorted(users, key=lambda u: -users[u]):
        print(f"user {u}: {users[u]}")


if __name__ == "__main__":
    main()
