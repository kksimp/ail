#!/usr/bin/env bash
# Checks that a std.gfx window's canvas is exactly the surface it asked for
# (ADR 0957, std.gfx.6): opens a 320x240 game for a few seconds on the Mac, reads
# the host's report, and fails unless canvas == surface in points.
#   tools/app-host/macos/gfx-size-check.sh
# Opens a window briefly; run it on the Mac only, with the designer's OK.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
tmp="$(mktemp -d)"
pid=""
trap '[ -z "$pid" ] || kill "$pid" 2>/dev/null || true; rm -rf "$tmp"' EXIT

cat >"$tmp/size.ail" <<'AIL'
use std.gfx
use std.gfx.host

struct Square
    x float

impl gfx.Game for Square
    fn update(mut self, input gfx.Input, dt float)
        x += 60.0 * dt
        if x > 300.0
            x = 0.0

    fn draw(self, mut f gfx.Frame)
        gfx.clear(mut f, 0x202020)
        gfx.rect(mut f, x, 100.0, 20.0, 20.0, 0xff8800)

fn main()
    mut s := Square(x: 0.0)
    host.run(mut s, "size check", 320, 240) else err
        print(err.why)
AIL

AIL_GFX_REPORT="$tmp/report" "$here/run.sh" "$tmp/size.ail" >"$tmp/out" 2>&1 &
pid=$!
for _ in $(seq 1 60); do
    [ -s "$tmp/report" ] && break
    sleep 0.5
done
if [ ! -s "$tmp/report" ]; then
    echo "gfx-size-check: no report from the host"
    cat "$tmp/out"
    exit 1
fi
line="$(head -n 1 "$tmp/report")"
echo "$line"
case "$line" in
    *"surface=320x240 canvas=320x240 "*) echo "gfx-size-check: ok" ;;
    *) echo "gfx-size-check: the canvas is not the surface"; exit 1 ;;
esac
