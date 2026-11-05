#!/usr/bin/env bash
# Builds the std.audio host for macOS (ADR 0949). From the repository root:
#   tools/audio-host/macos/run.sh           # build only
#   tools/audio-host/macos/run.sh --beep    # build, then play a half-second 440 Hz tone
# Needs the Xcode command line tools (swiftc); no third-party packages.
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exe="$here/build/ail-audio-host"
[ "$(uname -s)" = Darwin ] || { echo "audio-host: macOS only" >&2; exit 2; }
[ "$(uname -m)" = arm64 ] || { echo "audio-host: Apple Silicon only (ADR 0004)" >&2; exit 2; }
command -v swiftc >/dev/null 2>&1 || { echo "audio-host: install the Xcode command line tools (xcode-select --install)" >&2; exit 2; }
if [ ! -x "$exe" ] || [ -n "$(find "$here/Sources" "$here/run.sh" -newer "$exe" 2>/dev/null | head -n 1)" ]; then
    echo "audio-host: building $exe" >&2
    mkdir -p "$here/build"
    swiftc -O -target arm64-apple-macos13 -framework AVFoundation "$here"/Sources/*.swift -o "$exe"
fi
if [ "${1:-}" = "--beep" ]; then
    root="$(cd "$here/../../.." && pwd)"
    AIL_AUDIO_HOST="$exe" exec "$root/ail" run "$root/tools/audio-host/beep.ail"
fi
