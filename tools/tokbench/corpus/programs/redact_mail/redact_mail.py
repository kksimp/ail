import re
import sys
from pathlib import Path

DATA = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).resolve().parent / "agent-data"

MAIL = re.compile(r"[\w.+-]+@[\w-]+\.[\w.]+")


def main():
    text = (DATA / "records.txt").read_text()
    hidden = 0
    for line in text.splitlines():
        if MAIL.search(line):
            hidden += 1
        print(MAIL.sub("<email>", line))
    print(f"{hidden} lines redacted")


if __name__ == "__main__":
    main()
