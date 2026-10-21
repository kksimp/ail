import json
import os
import sys


def main():
    # The example prints the same lines from the local copy when there is no
    # network, so this twin reads the copy: the output is the same either way.
    here = os.path.dirname(os.path.abspath(__file__))
    name = os.path.basename(sys.argv[1]) if len(sys.argv) > 1 else "tasks.json"
    with open(os.path.join(here, "..", "..", "..", "..", "..", "examples", "data", name), encoding="utf-8") as f:
        body = f.read()
    lines = body.splitlines()
    print(f"{name}: {len(lines)} lines, {len(body)} characters")
    print(lines[0])


if __name__ == "__main__":
    main()
