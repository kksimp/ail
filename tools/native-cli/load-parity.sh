#!/usr/bin/env bash
# The native program loader against the Rust one (ADR 0855): for each FILE, the
# program `ail run compiler load` builds must equal `ail api --ast --program`.
#   tools/native-cli/load-parity.sh [FILE...]     default: examples/*.ail
# Prints one line per differing file (with the first differing field) and a count.
set -uo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$root"
export AIL_ROOT="$root"
files=("$@")
[ ${#files[@]} -gt 0 ] || files=(examples/*.ail)
out="$(mktemp -d)"
./ail run compiler load "${files[@]}" > "$out/native.txt" 2> "$out/native.err"
python3 - "$out" "${files[@]}" <<'P'
import json, subprocess, sys
out, files = sys.argv[1], sys.argv[2:]
text = open(out + "/native.txt").read()
if len(files) == 1:
    chunks = {files[0]: text}
else:
    chunks, cur = {}, None
    for line in text.split("\n"):
        if line.startswith("== ") and line[3:] in files:
            cur = line[3:]; chunks[cur] = ""
        elif cur is not None:
            chunks[cur] += line + "\n"
bad = 0
for f in files:
    want = subprocess.run(["./ail", "api", "--ast", "--program", f], capture_output=True, text=True).stdout
    got = chunks.get(f, "")
    try:
        a, b = json.loads(got.split("\n", 1)[0]), json.loads(want.split("\n", 1)[0])
    except ValueError:
        bad += 1; print("BAD   ", f, "(no program)"); continue
    # Fields only the native program carries, empty (`asks`, expr.hole.3), aren't differences.
    a = {k: v for k, v in a.items() if k in b or v not in (None, [])}
    if a != b:
        bad += 1
        k = next((k for k in b if a.get(k) != b[k]), "?")
        print("DIFF  ", f, k)
    elif got.split("\n")[1:] != want.split("\n")[1:] and got.rstrip("\n").split("\n")[1:] != want.rstrip("\n").split("\n")[1:]:
        bad += 1
        print("DIAGS ", f, got.rstrip("\n").split("\n")[1:][:3], want.rstrip("\n").split("\n")[1:][:3])
print(f"{len(files) - bad}/{len(files)} equal")
sys.exit(1 if bad else 0)
P
