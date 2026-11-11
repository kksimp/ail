#!/usr/bin/env bash
# Builds the std.app host for Linux (ADR 0838) and, given arguments, runs
# an AIL program with it. From the repository root:
#   tools/app-host/linux/run.sh                      # build only
#   tools/app-host/linux/run.sh examples/app_todo.ail
#   tools/app-host/linux/run.sh --native ../garlic/garlic.ail URL   # compiled, not interpreted
# Needs a C compiler, pkg-config and the GTK 3 headers; installs them with apt
# when they are missing. No other packages.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
exe="$here/build/ail-app-host"

[ "$(uname -s)" = Linux ] || { echo "app-host: Linux only (macOS has tools/app-host/macos)" >&2; exit 2; }

if [ ! -x "$exe" ] || [ "$here/host.c" -nt "$exe" ] || [ "$here/run.sh" -nt "$exe" ]; then
    if ! command -v gcc >/dev/null 2>&1 || ! pkg-config --exists gtk+-3.0 2>/dev/null; then
        echo "app-host: installing gcc, pkg-config and libgtk-3-dev (sudo)" >&2
        sudo apt-get install -y build-essential pkg-config libgtk-3-dev >&2
    fi
    echo "app-host: building $exe" >&2
    mkdir -p "$here/build"
    # shellcheck disable=SC2046
    gcc -O2 -Wall -Wextra -o "$exe" "$here/host.c" $(pkg-config --cflags --libs gtk+-3.0 fontconfig) -lm
fi

# Fonts the host prefers for text (Inter) and for colour emoji; a missing one only costs looks.
if ! fc-list : family 2>/dev/null | grep -qi '^Inter\b' || ! fc-list : family 2>/dev/null | grep -qi 'Noto Color Emoji'; then
    echo "app-host: installing fonts-inter fonts-noto-core fonts-noto-color-emoji (sudo)" >&2
    sudo apt-get install -y fonts-inter fonts-noto-core fonts-noto-color-emoji >&2 || echo "app-host: font install failed; continuing with the fonts present" >&2
fi

# Garlic's window icon (../garlic/icon); the host reads AIL_APP_ICON.
for a in "$@"; do
    case "$a" in
        */garlic/garlic.ail|garlic.ail) export AIL_APP_ICON="$root/../garlic/icon/garlic-256.png" ;;
    esac
done

if [ "${1:-}" = "--native" ]; then
    # Build the program natively (ADR 0022) and run it against this host.
    [ "$#" -ge 2 ] || { echo "usage: run.sh --native PROGRAM.ail [ARGS...]" >&2; exit 2; }
    prog="$2"; shift 2
    bin="$here/build/native-$(basename "$prog" .ail)"
    # Never run an older build: drop it first, and stop if this one fails.
    rm -f "$bin"
    if ! (cd "$(dirname "$prog")" && "$root/ail" build "$(basename "$prog")" -o "$bin") || [ ! -x "$bin" ]; then
        echo "app-host: the native build of $prog FAILED (see the errors above); nothing was run" >&2
        exit 1
    fi
    AIL_APP_HOST="$exe" exec "$bin" "$@"
fi

if [ "$#" -gt 0 ]; then
    AIL_APP_HOST="$exe" exec "$root/ail" run "$@"
fi
