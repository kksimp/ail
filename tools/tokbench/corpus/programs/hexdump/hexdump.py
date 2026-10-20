import subprocess
from pathlib import Path


def main():
    data = (Path(__file__).resolve().parents[5] / "examples/agent-data/status.json").read_bytes()
    for offset in range(0, len(data), 16):
        chunk = data[offset:offset + 16]
        print(f"{offset:>6}  {' '.join(f'{b:02x}' for b in chunk)}")
    print(f"{len(data)} bytes")


if __name__ == "__main__":
    main()
