import hashlib
import sys
import zlib
from pathlib import Path

DATA = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).resolve().parent / "agent-data"


def main():
    text = (DATA / "records.txt").read_text()
    for line in text.splitlines():
        digest = hashlib.sha256(line.encode()).hexdigest()
        print(f"{digest} {zlib.crc32(line.encode())} {line}")


if __name__ == "__main__":
    main()
