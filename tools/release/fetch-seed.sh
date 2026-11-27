#!/usr/bin/env bash
# Download the native compiler seed for this host, from seeds.voltare.us first and then the repository's release,
# and install it as .native/seed (ADR 0855). The SHA-256 in
# tools/release/seeds.sha256 must match; nothing unverified is installed.
# The seed builds compiler/main.ail into the current .native/ail (see ./ail).
# Tries the plain release-download URL first (no login); if that fails, as it does
# while the repository is private, falls back to `gh` or GITHUB_TOKEN over the REST
# API (`gh release download` needs GraphQL, which some hosts block).
#   tools/release/fetch-seed.sh [TAG]     TAG defaults to the first line of tools/release/SEED_TAG
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
case "$(uname -s)-$(uname -m)" in
    Darwin-arm64) asset=ail-seed-aarch64-macos ;;
    Linux-x86_64) asset=ail-seed-x86_64-linux ;;
    Linux-aarch64 | Linux-arm64) asset=ail-seed-aarch64-linux ;;
    *) echo "fetch-seed: unsupported host $(uname -s) $(uname -m)" >&2; exit 2 ;;
esac
tag="${1:-$(head -n 1 "$root/tools/release/SEED_TAG" 2>/dev/null || true)}"
want="$(awk -v a="$asset" '$2 == a { print $1 }' "$root/tools/release/seeds.sha256" 2>/dev/null || true)"
[ -n "$tag" ] && [ -n "$want" ] || { echo "fetch-seed: no seed is published yet for $asset (tools/release/seeds.sha256)" >&2; exit 2; }
repo="kksimp/ail"
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
# AIL_SEED_HOST unset: the default mirror; set to empty or `none`: no mirror (ADR 1210).
host="${AIL_SEED_HOST-https://seeds.voltare.us}"; [ "$host" = none ] && host=""
sum256() { if command -v sha256sum >/dev/null 2>&1; then sha256sum "$1" | cut -d' ' -f1; else shasum -a 256 "$1" | cut -d' ' -f1; fi; }
# A mirror copy that fails the pinned hash is discarded, and the next source is tried.
if command -v curl >/dev/null 2>&1 && [ -n "$host" ] &&
    curl -fsSL --max-time 120 -o "$tmp/$asset" "$host/ail/$tag/$asset" 2>/dev/null && [ "$(sum256 "$tmp/$asset")" = "$want" ]; then
    :
elif rm -f "$tmp/$asset"; command -v curl >/dev/null 2>&1 &&
    curl -fsSL -o "$tmp/$asset" "https://github.com/$repo/releases/download/$tag/$asset" 2>/dev/null; then
    :
elif command -v gh >/dev/null 2>&1 && gh auth status >/dev/null 2>&1; then
    id="$(gh api "repos/$repo/releases/tags/$tag" --jq ".assets[] | select(.name == \"$asset\") | .id")"
    [ -n "$id" ] || { echo "fetch-seed: release $tag has no asset $asset" >&2; exit 2; }
    gh api -H "Accept: application/octet-stream" "repos/$repo/releases/assets/$id" > "$tmp/$asset"
elif [ -n "${GITHUB_TOKEN:-}" ]; then
    api="https://api.github.com/repos/$repo/releases/tags/$tag"
    id="$(curl -fsS -H "Authorization: Bearer $GITHUB_TOKEN" "$api" |
        python3 -c 'import json,sys; print(next(a["id"] for a in json.load(sys.stdin)["assets"] if a["name"]==sys.argv[1]))' "$asset")"
    curl -fsSL -H "Authorization: Bearer $GITHUB_TOKEN" -H "Accept: application/octet-stream" \
        -o "$tmp/$asset" "https://api.github.com/repos/$repo/releases/assets/$id"
else
    echo "fetch-seed: can't download $asset from release $tag (needs curl; for a private copy, sign in with gh or set GITHUB_TOKEN)" >&2; exit 2
fi
if command -v sha256sum >/dev/null 2>&1; then got="$(sha256sum "$tmp/$asset" | cut -d' ' -f1)"
else got="$(shasum -a 256 "$tmp/$asset" | cut -d' ' -f1)"; fi
[ "$got" = "$want" ] || { echo "fetch-seed: SHA-256 mismatch for $asset (got $got, want $want); not installing" >&2; exit 1; }
mkdir -p "$root/.native"
install -m 755 "$tmp/$asset" "$root/.native/seed"
echo "fetch-seed: installed $asset ($tag) as .native/seed" >&2
