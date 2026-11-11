#!/usr/bin/env bash
# Checks the Linux std.app host's protocol with no AIL toolchain (the Linux twin of
# tools/app-host/windows/smoke.ps1): plays tests/ui/host_conformance.txt, then maps a
# shared surface and expects a `vsync` for it (std.gfx.8). Builds the host first and
# runs under xvfb-run when there is no display.
#   tools/app-host/linux/smoke.sh
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
"$here/run.sh" >&2 || { echo "smoke: RED the host did not build"; exit 1; }
exe="$here/build/ail-app-host"
if [ -z "${DISPLAY:-}" ] && [ -z "${WAYLAND_DISPLAY:-}" ]; then
    command -v xvfb-run >/dev/null 2>&1 || { echo "smoke: no display and no xvfb-run (apt install xvfb)"; exit 2; }
    exec xvfb-run -a "$0" "$@"
fi

r="$("$root/tests/ui/host_conformance.sh" linux "$exe")"
echo "$r"
[ "$r" = "conformance: GREEN" ] || { echo "smoke: RED"; exit 1; }

# std.gfx.8: a 4 by 4 surface of two buffers behind a 4096-byte guard; the host maps it, sizes the
# canvas to it, and answers each drawn frame with `vsync`.
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
truncate -s $((4096 * 2 + 16 * 4 * 2)) "$tmp/surf"
export AIL_GFX_REPORT="$tmp/report"
out="$tmp/out"; : > "$out"; mkfifo "$tmp/in"
"$exe" < "$tmp/in" > "$out" 2>/dev/null &
HOST_PID=$!
exec 3> "$tmp/in"
printf '%s\n' '{"v":1,"t":"view","title":"gfx","focus":"gfx","tree":{"kind":"Canvas","id":"gfx","w":4,"h":4,"ops":[]}}' >&3
printf '%s\n' "{\"v\":1,\"t\":\"surface\",\"id\":\"gfx\",\"w\":4,\"h\":4,\"stride\":16,\"offset\":4096,\"count\":2,\"path\":\"$tmp/surf\"}" >&3
printf '%s\n' '{"v":1,"t":"present","id":"gfx","buf":1}' >&3
for _ in $(seq 1 40); do
    grep -q '"kind":"vsync"' "$out" && grep -q '"text":"1"' "$out" && [ -s "$tmp/report" ] && break
    sleep 0.5
done
exec 3>&-
for _ in $(seq 1 20); do kill -0 "$HOST_PID" 2>/dev/null || break; sleep 0.5; done
kill "$HOST_PID" 2>/dev/null
grep -q '"kind":"vsync"' "$out" || { echo "smoke: RED no vsync for the surface"; exit 1; }
grep -q '"text":"1"' "$out" || { echo "smoke: RED no vsync after present of buffer 1"; exit 1; }
grep -q 'surface=4x4 canvas=4x4' "$tmp/report" 2>/dev/null || { echo "smoke: RED the canvas is not the surface's size: $(cat "$tmp/report" 2>/dev/null)"; exit 1; }
echo "smoke: GREEN"
