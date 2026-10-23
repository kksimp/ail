#!/usr/bin/env bash
# Git dependencies and ail.lock in the native loader (mod.pkg.7-8, ADR 0856):
# fetch from a local repository, write the lock, then refuse a stale one under
# AIL_PKG_FROZEN, and compare the program with the Rust loader's.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$root"
t="$(mktemp -d)"
trap 'rm -rf "$t"' EXIT
export AIL_HOME="$t/home" AIL_ROOT="$root"
mkdir "$t/repo" "$t/app"
(cd "$t/repo" && git init -q && printf 'pub fn hi() str\n    "hi"\n' > shapes.ail && printf 'package geo\n' > ail.pkg \
  && git add -A && git -c user.email=a@b -c user.name=n commit -qm x)
sha="$(git -C "$t/repo" rev-parse HEAD)"
printf 'package app\ndep geo git file://%s/repo rev %s\n' "$t" "$sha" > "$t/app/ail.pkg"
printf 'use geo.shapes\nfn main()\n    print(shapes.hi())\n' > "$t/app/main.ail"
AIL_PKG_FROZEN=1 ./ail run compiler load "$t/app/main.ail" 2>&1 | grep -q 'mod.pkg.8' || { echo "FAIL frozen without a lock"; exit 1; }
AIL_PKG_WRITE_LOCK=1 tools/native-cli/load-parity.sh "$t/app/main.ail"
grep -q "commit $sha" "$t/app/ail.lock" || { echo "FAIL lock not written"; exit 1; }
AIL_PKG_FROZEN=1 AIL_PKG_OFFLINE=1 ./ail run compiler load "$t/app/main.ail" 2>&1 | grep -q 'mod.pkg' && { echo "FAIL locked cache refused"; exit 1; }
echo "pkg-git ok"
