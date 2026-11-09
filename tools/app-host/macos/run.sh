#!/usr/bin/env bash
# Builds the std.app host for macOS (ADR 0838) and, given arguments, runs
# an AIL program with it. From the repository root:
#   tools/app-host/macos/run.sh                      # build only
#   tools/app-host/macos/run.sh examples/app_todo.ail
#   tools/app-host/macos/run.sh --native ../garlic/garlic.ail URL   # compiled, not interpreted
# Needs the Xcode command line tools (swiftc); no third-party packages.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
app="$here/build/AIL App Host.app"
exe="$app/Contents/MacOS/AILAppHost"

[ "$(uname -s)" = Darwin ] || { echo "app-host: macOS only" >&2; exit 2; }
[ "$(uname -m)" = arm64 ] || { echo "app-host: Apple Silicon only (ADR 0004)" >&2; exit 2; }
if ! command -v swiftc >/dev/null 2>&1; then
    echo "app-host: installing the Xcode command line tools (a system dialog opens); run this again when it finishes" >&2
    xcode-select --install || true
    exit 2
fi

if [ ! -x "$exe" ] || [ -n "$(find "$here/Sources" "$here/run.sh" -newer "$exe" 2>/dev/null | head -n 1)" ]; then
    echo "app-host: building $app" >&2
    mkdir -p "$app/Contents/MacOS"
    cat >"$app/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleExecutable</key><string>AILAppHost</string>
<key>CFBundleIdentifier</key><string>us.voltare.ail.apphost</string>
<key>CFBundleName</key><string>AIL App Host</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleShortVersionString</key><string>1</string>
<key>LSMinimumSystemVersion</key><string>13.0</string>
<key>NSHighResolutionCapable</key><true/>
</dict></plist>
PLIST
    swiftc -O -parse-as-library -target arm64-apple-macos13 \
        -framework SwiftUI -framework AppKit -framework GameController \
        "$here"/Sources/*.swift -o "$exe"
fi

# Garlic's Dock icon (../garlic/icon); the host reads AIL_APP_ICON.
for a in "$@"; do
    case "$a" in
        */garlic/garlic.ail|garlic.ail) export AIL_APP_ICON="$root/../garlic/icon/garlic-512.png" ;;
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
    exec "$root/ail" run "$@"
fi
