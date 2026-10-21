import re
from pathlib import Path

MAIL = re.compile(r"[\w.+-]+@[\w-]+\.[\w.]+")


def main():
    text = (Path(__file__).resolve().parents[5] / "examples/agent-data/records.txt").read_text()
    hidden = 0
    for line in text.splitlines():
        if MAIL.search(line):
            hidden += 1
        print(MAIL.sub("<email>", line))
    print(f"{hidden} lines redacted")


if __name__ == "__main__":
    main()
