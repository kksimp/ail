import gzip
import sys
from pathlib import Path

DATA = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).resolve().parent / "agent-data"


def main():
    text = (DATA / "records.txt").read_text()
    big = (text * 50).encode()
    packed = gzip.compress(big)
    print(f"gzip is smaller: {str(len(packed) < len(big)).lower()}")
    back = gzip.decompress(packed)
    print(f"round trip equal: {str(back.decode() == big.decode()).lower()}")


if __name__ == "__main__":
    main()
