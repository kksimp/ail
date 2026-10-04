#!/usr/bin/env bash
# One-command token measurement for macOS and Linux.
#
#   tools/tokbench/run.sh                  # measure every corpus file (offline tokenizers)
#   tools/tokbench/run.sh FILE...          # measure specific files
#   tools/tokbench/run.sh --claude         # also count with the current Claude tokenizer
#                                          # (needs ANTHROPIC_API_KEY or `ant auth login`)
#   tools/tokbench/run.sh --json           # machine-readable output
#
# The first run creates a private Python environment in tools/tokbench/.venv
# and downloads tokenizer files into tools/tokbench/.cache. Nothing is
# installed system-wide. Set PYTHON=/path/to/python to choose the interpreter.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"   # file arguments stay relative to the caller's directory

python="${PYTHON:-}"
if [ -z "$python" ]; then
    for candidate in python3 python; do
        if command -v "$candidate" >/dev/null 2>&1; then python="$candidate"; break; fi
    done
fi
if [ -z "$python" ]; then
    echo "error: Python 3.10 or newer is required (macOS: brew install python)" >&2
    exit 1
fi
if ! "$python" -c 'import sys; sys.exit(sys.version_info < (3, 10))'; then
    echo "error: $("$python" --version 2>&1) is too old; Python 3.10 or newer is required" >&2
    exit 1
fi

venv_python="$here/.venv/bin/python"
if [ ! -x "$venv_python" ]; then
    echo "setting up Python environment in tools/tokbench/.venv ..."
    "$python" -m venv "$here/.venv"
fi
if [ ! -f "$here/.venv/.installed" ] || [ "$here/requirements.txt" -nt "$here/.venv/.installed" ]; then
    "$venv_python" -m pip install --quiet --upgrade pip
    "$venv_python" -m pip install --quiet -r "$here/requirements.txt"
    touch "$here/.venv/.installed"
fi
for arg in "$@"; do
    if [ "$arg" = "--claude" ] && ! "$venv_python" -c 'import anthropic' 2>/dev/null; then
        "$venv_python" -m pip install --quiet anthropic
    fi
done
if [ ! -f "$here/.cache/.complete" ]; then
    "$venv_python" "$here/fetch_tokenizers.py"
fi

exec "$venv_python" "$here/tokbench.py" "$@"
