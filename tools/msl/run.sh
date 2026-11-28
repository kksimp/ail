#!/bin/bash
# The MSL emitter against its goldens (kksimp/ail#1599): `ail gpu-msl` over tests/msl/NAME.ail must print NAME.msl.
#   tools/msl/run.sh           check
#   tools/msl/run.sh --freeze  rewrite the goldens (review the diff)
cd "$(dirname "$0")/../.." || exit 1
fail=0
for f in tests/msl/*.ail; do
    g="${f%.ail}.msl"
    out=$(./ail run compiler gpu-msl "$f" 2>/dev/null)
    if [ "$1" = "--freeze" ]; then printf '%s\n' "$out" > "$g"; continue; fi
    if [ "$out" != "$(cat "$g")" ]; then echo "RED $f"; diff <(printf '%s\n' "$out") "$g" | head -20; fail=1; fi
done
[ $fail = 0 ] && echo "MSL GREEN"
exit $fail
