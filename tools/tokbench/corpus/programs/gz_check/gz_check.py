import gzip
from pathlib import Path


def main():
    text = (Path(__file__).resolve().parents[5] / "examples/agent-data/records.txt").read_text()
    big = (text * 50).encode()
    packed = gzip.compress(big)
    print(f"gzip is smaller: {str(len(packed) < len(big)).lower()}")
    back = gzip.decompress(packed)
    print(f"round trip equal: {str(back.decode() == big.decode()).lower()}")


if __name__ == "__main__":
    main()
