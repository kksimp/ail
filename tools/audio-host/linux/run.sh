#!/usr/bin/env bash
# Builds the std.audio host for Linux (ADR 0949). From the repository root:
#   tools/audio-host/linux/run.sh           # build only
#   tools/audio-host/linux/run.sh --beep    # build, then play a half-second 440 Hz tone
#   tools/audio-host/linux/run.sh --test    # build, then check the protocol on ALSA's "null" device
# Needs a C compiler and the ALSA headers (libasound2-dev); installs them with
# apt when they are missing. AIL_ALSA_DEVICE picks another ALSA device.
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
exe="$here/build/ail-audio-host"
[ "$(uname -s)" = Linux ] || { echo "audio-host: Linux only (macOS has tools/audio-host/macos)" >&2; exit 2; }
if [ ! -x "$exe" ] || [ "$here/host.c" -nt "$exe" ] || [ "$here/run.sh" -nt "$exe" ]; then
    if ! command -v gcc >/dev/null 2>&1 || ! pkg-config --exists alsa 2>/dev/null; then
        echo "audio-host: installing gcc, pkg-config and libasound2-dev (sudo)" >&2
        sudo apt-get install -y build-essential pkg-config libasound2-dev >&2
    fi
    echo "audio-host: building $exe" >&2
    mkdir -p "$here/build"
    # shellcheck disable=SC2046
    gcc -O2 -Wall -Wextra -pthread -o "$exe" "$here/host.c" $(pkg-config --cflags --libs alsa)
fi
case "${1:-}" in
--beep)
    AIL_AUDIO_HOST="$exe" exec "$root/ail" run "$root/tools/audio-host/beep.ail"
    ;;
--test)
    # 4 frames of mono silence (8 bytes of zeros) and a 20 ms recording, on the null device.
    out="$(printf 'play 8000 1 AAAAAAAAAAA=\nrec 20 8000 1\nbogus\n' | AIL_ALSA_DEVICE=null "$exe")"
    printf '%s\n' "$out" | sed -n 1,3p | cut -c1-20
    [ "$(printf '%s\n' "$out" | sed -n 1p)" = ready ] || { echo "audio-host: no ready line" >&2; exit 1; }
    [ "$(printf '%s\n' "$out" | sed -n 2p)" = done ] || { echo "audio-host: play did not finish" >&2; exit 1; }
    printf '%s\n' "$out" | sed -n 3p | grep -q '^data 8000 1 ' || { echo "audio-host: bad rec reply" >&2; exit 1; }
    printf '%s\n' "$out" | sed -n 4p | grep -q '^error unknown request' || { echo "audio-host: bad error reply" >&2; exit 1; }
    # Streaming on the silent clock (`--null`): 4 frames, then the status lines.
    out="$(printf 'open_stream 8000 1\nwrite AAAAAAAAAAA=\nflush\nclose_stream\nclock\n' | "$exe" --null)"
    [ "$(printf '%s\n' "$out" | sed -n 2p)" = ok ] || { echo "audio-host: open_stream failed" >&2; exit 1; }
    printf '%s\n' "$out" | sed -n 3p | grep -q '^played [0-9]* buffered [0-9]*$' || { echo "audio-host: bad write reply" >&2; exit 1; }
    [ "$(printf '%s\n' "$out" | sed -n 4p)" = "played 0 buffered 0" ] || { echo "audio-host: flush did not zero the clock" >&2; exit 1; }
    [ "$(printf '%s\n' "$out" | sed -n 5p)" = ok ] || { echo "audio-host: close_stream failed" >&2; exit 1; }
    echo "audio-host: protocol ok"
    ;;
esac
