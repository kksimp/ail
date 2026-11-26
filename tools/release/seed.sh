#!/usr/bin/env bash
# Build and publish the native `ail` seeds (ADR 0855). The designer runs this
# on the Mac; nothing publishes automatically.
#   tools/release/seed.sh TAG [--upload]
# Checks the self-host fixed point (tools/self-host/run.sh), builds the native
# CLI (compiler/main.ail) with the stage-2 compiler for each target, writes
# tools/release/seeds.sha256 and SEED_TAG, and with --upload makes the
# release TAG on kksimp/ail with `gh` (it has the repository's visibility). The tag
# is made at the commit the seeds were built from (HEAD, which must be pushed) and
# the release is not marked Latest, so it never replaces the version release.
# Commit the two files afterwards.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$root"
tag="${1:?usage: tools/release/seed.sh TAG [--upload]}"
upload="${2:-}"
out="$(mktemp -d)"
echo "seed: self-host fixed point ..." >&2
ONLY=none tools/self-host/run.sh "$out/sh" >&2 || { echo "seed: the fixed point isn't reached; no seeds built" >&2; exit 1; }
stage2="$out/sh/stage2"
sums="$out/seeds.sha256"; : > "$sums"
for pair in aarch64-macos:aarch64-macos x86_64-linux:x86_64-linux aarch64-linux:aarch64-linux x86_64-windows.exe:x86_64-windows; do
    name="ail-seed-${pair%%:*}"; target="${pair##*:}"
    echo "seed: building $name ..." >&2
    "$stage2" build compiler/main.ail compiler/rt/native "$target" 0 0 std > "$out/$name.hex"
    perl -ne 'chomp; print pack("H*", $_)' < "$out/$name.hex" > "$out/$name"
    chmod +x "$out/$name"
    if command -v sha256sum >/dev/null 2>&1; then sum="$(sha256sum "$out/$name" | cut -d' ' -f1)"
    else sum="$(shasum -a 256 "$out/$name" | cut -d' ' -f1)"; fi
    echo "$sum  $name" >> "$sums"
done
cp "$sums" tools/release/seeds.sha256
echo "$tag" > tools/release/SEED_TAG
cat tools/release/seeds.sha256 >&2
if [ "$upload" = --upload ]; then
    gh release create "$tag" --repo kksimp/ail --target "$(git rev-parse HEAD)" --latest=false --title "Native ail seeds $tag" --notes "ADR 0855 seeds; verified by tools/release/seeds.sha256." \
        "$out"/ail-seed-aarch64-macos "$out"/ail-seed-x86_64-linux "$out"/ail-seed-aarch64-linux "$out"/ail-seed-x86_64-windows.exe
    echo "seed: binaries are in $out" >&2
    echo "seed: uploaded $tag. Commit tools/release/seeds.sha256 and SEED_TAG." >&2
else
    echo "seed: built in $out; rerun with --upload to publish" >&2
fi
