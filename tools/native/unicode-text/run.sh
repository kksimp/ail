#!/bin/sh
# Regenerates the tables of std.unicode and its conformance test from the
# Unicode 16.0.0 character database (tools/native/gen_unicode_text.py):
#
#     tools/native/unicode-text/run.sh
#
# Downloads the eight UCD files once into a cache, checks each against the
# hash pinned below, and stops on a mismatch. Needs python3 and curl.
set -eu
cd "$(dirname "$0")/../../.."
VERSION=16.0.0
BASE=${UCD_BASE:-https://www.unicode.org/Public/$VERSION/ucd}
CACHE=${UCD_CACHE:-${TMPDIR:-/tmp}/ail-ucd-$VERSION}
mkdir -p "$CACHE"

sha() {
    if command -v sha256sum >/dev/null 2>&1; then sha256sum "$1" | cut -d' ' -f1
    else shasum -a 256 "$1" | cut -d' ' -f1; fi
}

while read -r hash path; do
    name=$(basename "$path")
    if [ ! -f "$CACHE/$name" ] || [ "$(sha "$CACHE/$name")" != "$hash" ]; then
        curl -fsSL -o "$CACHE/$name.part" "$BASE/$path"
        mv "$CACHE/$name.part" "$CACHE/$name"
    fi
    got=$(sha "$CACHE/$name")
    if [ "$got" != "$hash" ]; then
        echo "$path: sha256 $got, pinned $hash" >&2
        exit 1
    fi
done <<'PINS'
ff58e5823bd095166564a006e47d111130813dcf8bf234ef79fa51a870edb48f UnicodeData.txt
4d4c03892dea9146d674b686e495df2d55a28d071ac474041d73518f887abddc DerivedNormalizationProps.txt
6f1f9c588eb4a5c718d9e8f93b782685e5c7fec872cf05e8e6878053599e09bb CaseFolding.txt
39d35161f2954497f69e08bdb9e701493f476a3d30222de20028feda36c1dabd DerivedCoreProperties.txt
d811971453e7075e1ad56fb1b301eece5aa80757b81f6156e74a1bfb3ae5ceb1 NormalizationTest.txt
c29360bd6f7132811d701d29069541e827eb44bfc4c8fbde8c370d6982689dc1 auxiliary/GraphemeBreakProperty.txt
ee2b9354d270ac061b29f09662cafea06341d77e704b8cc6bd72aaeeda363cb5 auxiliary/GraphemeBreakTest.txt
f1365a5173eee18e1f98b240cdc492e84a25f1ce7e0c9d1094eb29c41a22696a emoji/emoji-data.txt
PINS

python3 tools/native/gen_unicode_text.py "$CACHE"
