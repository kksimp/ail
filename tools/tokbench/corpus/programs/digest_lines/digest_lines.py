import hashlib
import zlib
from pathlib import Path


def main():
    text = (Path(__file__).resolve().parents[5] / "examples/agent-data/records.txt").read_text()
    for line in text.splitlines():
        digest = hashlib.sha256(line.encode()).hexdigest()
        print(f"{digest} {zlib.crc32(line.encode())} {line}")


if __name__ == "__main__":
    main()
