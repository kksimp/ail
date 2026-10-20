import os
import tarfile
import tempfile
from pathlib import Path


def main():
    src = Path(__file__).resolve().parents[5] / "examples/configs"
    out = Path(os.environ.get("TMPDIR", "/tmp")) / "ail-backup-check.tar.gz"
    with tarfile.open(out, "w:gz") as tar:
        tar.add(src, arcname=".")
    with tarfile.open(out) as tar:
        names = sorted(tar.getnames())
    count = 0
    for name in names:
        if name == ".":
            continue
        count += 1
        print(f"  {name}")
    out.unlink()
    print(f"archived {count} files")


if __name__ == "__main__":
    main()
