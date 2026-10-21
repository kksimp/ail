import re
from pathlib import Path


def main():
    text = (Path(__file__).resolve().parents[5] / "examples/agent-data/app.log").read_text()
    codes = [int(n) for n in re.findall(r"code=(\d+)", text)]
    print(f"{len(codes)} codes, sum {sum(codes)}")


if __name__ == "__main__":
    main()
