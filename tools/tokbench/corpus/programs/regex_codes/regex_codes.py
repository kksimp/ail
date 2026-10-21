import re
import sys
from pathlib import Path

DATA = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).resolve().parent / "agent-data"


def main():
    text = (DATA / "app.log").read_text()
    codes = [int(n) for n in re.findall(r"code=(\d+)", text)]
    print(f"{len(codes)} codes, sum {sum(codes)}")


if __name__ == "__main__":
    main()
