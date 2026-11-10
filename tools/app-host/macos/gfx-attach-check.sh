#!/usr/bin/env bash
# Checks that a live std.gfx window publishes the attach socket (ADR 0979,
# std.gfx.10): runs a small game with `./ail run` (attach on by default), then asks
# `ail attach screen` and `ail attach observe` what is on screen.
#   tools/app-host/macos/gfx-attach-check.sh
# Opens a window briefly; run it on the Mac only, with the designer's OK.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
tmp="$(mktemp -d)"
pid=""
trap '[ -z "$pid" ] || kill "$pid" 2>/dev/null || true; rm -rf "$tmp"' EXIT

cat >"$tmp/game.ail" <<'AIL'
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
    host.run(mut s, "attach check", 320, 240) else err
        print(err.why)
AIL

export TMPDIR="$tmp"
"$here/run.sh" "$tmp/game.ail" >"$tmp/out" 2>&1 &
pid=$!
for _ in $(seq 1 60); do
    ls "$tmp"/ail-attach-*.json >/dev/null 2>&1 && break
    sleep 0.5
done
if ! ls "$tmp"/ail-attach-*.json >/dev/null 2>&1; then
    echo "gfx-attach-check: the game published no attach socket"
    cat "$tmp/out"
    exit 1
fi
sleep 1
screen="$("$root/ail" attach screen)"
echo "$screen"
observe="$("$root/ail" attach observe)"
echo "$observe"
case "$screen" in *box*) ;; *) echo "gfx-attach-check: no box on the screen"; exit 1 ;; esac
case "$observe" in *gfx.cmds*) echo "gfx-attach-check: ok" ;; *) echo "gfx-attach-check: no gfx.cmds"; exit 1 ;; esac
