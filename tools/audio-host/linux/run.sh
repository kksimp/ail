#!/usr/bin/env bash
# Builds the std.audio host for Linux (ADR 0949). From the repository root:
#   tools/audio-host/linux/run.sh
# Needs a C compiler. ALSA is loaded at run time (libasound.so.2), so no headers.
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exe="$here/build/ail-audio-host"
[ "$(uname -s)" = Linux ] || { echo "audio-host: Linux only" >&2; exit 2; }
command -v gcc >/dev/null 2>&1 || { echo "audio-host: install gcc (build-essential)" >&2; exit 2; }
if [ ! -x "$exe" ] || [ "$here/host.c" -nt "$exe" ] || [ "$here/run.sh" -nt "$exe" ]; then
    echo "audio-host: building $exe" >&2
    mkdir -p "$here/build"
    gcc -O2 -Wall -Wextra -o "$exe" "$here/host.c" -ldl
fi
