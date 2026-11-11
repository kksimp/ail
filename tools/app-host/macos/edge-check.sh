#!/usr/bin/env bash
# Checks edge mode (std.app.25, ADR 0981) on the macOS host: opens a small
# edge-mode app several times and fails unless every launch tells the app the
# room the window controls take (inset_left and inset_top above 0, on both
# canvases) and gives the page canvas the same size each time. A host that
# loses the race between the first view and its window reports inset_left 0
# and leaves a title bar's height unused.
#   tools/app-host/macos/edge-check.sh [LAUNCHES]     (12 by default)
# Opens windows briefly; run it on the Mac only, with the designer's OK.
set -uo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
runs="${1:-12}"
tmp="$(mktemp -d)"
pid=""
trap '[ -z "$pid" ] || { pkill -P "$pid" 2>/dev/null; kill "$pid" 2>/dev/null; } || true; rm -rf "$tmp"' EXIT

cat >"$tmp/edge.ail" <<'AIL'
use std.app
use std.env
use std.fs

## An app shaped like a browser: a strip the app draws under the window
## controls, and a page canvas that fills the rest. It writes the insets and
## sizes the host reports to the file named by AIL_EDGE_REPORT.
struct Probe
    path str
    w float

impl app.App for Probe
    fn update(mut self, ev app.Event)
        if ev.kind == "resize" and ev.id == "strip"
            w = ev.dx
        if ev.kind == "start" or ev.kind == "resize"
            row := "{ev.kind} id={ev.id} left={ev.inset_left} top={ev.inset_top} dx={ev.dx} dy={ev.dy}\n"
            fs.append_text(path, row) else return

    fn view(self) app.Node
        app.column([
            app.canvas("strip", 0.0, 40.0, [app.draw_rect(0.0, 0.0, w, 40.0, 0x202124)]),
            app.canvas("page", 0.0, 0.0, [app.draw_rect(0.0, 0.0, 4000.0, 4000.0, 0xffffff)]),
        ])

    fn title(self) str
        "edge check"

    fn edge(self) bool
        true

fn main()
    mut p := Probe(path: env.get("AIL_EDGE_REPORT") else "edge-report.txt", w: 1.0)
    app.run(mut p) else err
        print(err.message)
AIL

"$here/run.sh" || { echo "edge-check: the host did not build"; exit 1; }
host="$here/build/AIL App Host.app/Contents/MacOS/AILAppHost"
if ! (cd "$tmp" && "$root/ail" build edge.ail -o "$tmp/edge") || [ ! -x "$tmp/edge" ]; then
    echo "edge-check: the probe app did not build"
    exit 1
fi

# The last value of FIELD on the last line of the report for canvas ID.
last() {
    grep " id=$1 " "$2" | tail -n 1 | tr ' ' '\n' | grep "^$3=" | cut -d= -f2
}
# Whether a number is above zero.
above_zero() {
    awk -v v="${1:-0}" 'BEGIN { exit !(v + 0 > 0) }'
}

bad=0
sizes=""
n=1
while [ "$n" -le "$runs" ]; do
    report="$tmp/report-$n"
    : >"$report"
    AIL_APP_HOST="$host" AIL_EDGE_REPORT="$report" "$tmp/edge" >"$tmp/out-$n" 2>&1 &
    pid=$!
    # Wait for the first page size, then let the window settle.
    for _ in $(seq 1 40); do
        grep -q " id=page " "$report" && break
        sleep 0.25
    done
    sleep 1.5
    pkill -P "$pid" 2>/dev/null
    kill "$pid" 2>/dev/null
    wait "$pid" 2>/dev/null
    pid=""
    sl="$(last strip "$report" left)"; st="$(last strip "$report" top)"
    pl="$(last page "$report" left)"; pt="$(last page "$report" top)"
    size="$(last page "$report" dx)x$(last page "$report" dy)"
    verdict=ok
    if ! above_zero "$sl" || ! above_zero "$st" || ! above_zero "$pl" || ! above_zero "$pt"; then
        verdict="FAIL (an inset is 0 or missing)"
        bad=$(( bad + 1 ))
    fi
    echo "launch $n: strip left=${sl:-?} top=${st:-?}  page left=${pl:-?} top=${pt:-?} size=$size  $verdict"
    if [ "$verdict" != ok ]; then
        sed 's/^/    /' "$report"
        sed 's/^/    host: /' "$tmp/out-$n" | tail -n 5
    fi
    sizes="$sizes$size
"
    n=$(( n + 1 ))
done

kinds="$(printf '%s' "$sizes" | sort -u | grep -c .)"
if [ "$kinds" -ne 1 ]; then
    echo "edge-check: the page canvas was not the same size in every launch:"
    printf '%s' "$sizes" | sort | uniq -c
    bad=$(( bad + 1 ))
fi
if [ "$bad" -ne 0 ]; then
    echo "edge-check: FAILED"
    exit 1
fi
echo "edge-check: ok ($runs launches)"
