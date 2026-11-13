#!/usr/bin/env bash
# The FFI ABI check (ADR 1074): builds abi.c as a shared library with the host's
# C compiler, builds abi.ail natively with this checkout's compiler, runs it and
# compares with abi.out. Every struct class of System V x86-64, AAPCS64 (Linux
# and Apple) and Win64 passes both ways, and C calls AIL callbacks, one of which
# allocates enough for the collector to run while C is on the stack.
# Run from anywhere on macOS or Linux: tools/ffi-abi/run.sh (Windows: run.ps1).
set -euo pipefail
cd "$(dirname "$0")/../.."
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
cc=$(command -v cc || command -v clang || command -v gcc) || { echo "ffi-abi: no C compiler"; exit 1; }
case "$(uname -s)" in
  Darwin) lib=libailabi.dylib; "$cc" -O2 -dynamiclib tools/ffi-abi/abi.c -o "$tmp/$lib" ;;
  *) lib=libailabi.so; "$cc" -O2 -shared -fPIC tools/ffi-abi/abi.c -o "$tmp/$lib" ;;
esac
./ail build tools/ffi-abi/abi.ail -o "$tmp/abi"
if ! LD_LIBRARY_PATH="$tmp" DYLD_LIBRARY_PATH="$tmp" "$tmp/abi" > "$tmp/out" 2>&1; then
  cat "$tmp/out"
  echo "ffi-abi: RED (the program failed)"
  exit 1
fi
if diff tools/ffi-abi/abi.out "$tmp/out"; then
  echo "ffi-abi: GREEN"
else
  echo "ffi-abi: RED"
  exit 1
fi
