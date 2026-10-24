#!/usr/bin/env bash
# Native std.ui against a fake std.app host (ADR 0863): builds window.ail with
# the native compiler and compares its output with window.out. Needs a stage-1
# compiler: STAGE1=PATH (as for tools/self-host/run.sh).
set -e
cd "$(dirname "$0")/../.."
: "${STAGE1:?set STAGE1 to a native compiler executable}"
t=$(mktemp -d)
"$STAGE1" build tests/ui/window.ail compiler/rt/native "${TARGET:-x86_64-linux}" 0 0 std > "$t/hex"
perl -ne 'chomp; print pack("H*", $_)' < "$t/hex" > "$t/exe"; chmod +x "$t/exe"
AIL_APP_HOST="$PWD/tests/ui/fake_host.sh" "$t/exe" | diff - tests/ui/window.out && echo ok
