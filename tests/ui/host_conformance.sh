#!/usr/bin/env bash
# Plays tests/ui/host_conformance.txt against a std.app host, with no AIL toolchain.
#   tests/ui/host_conformance.sh PLATFORM HOST [ARGS...]     PLATFORM is mac, linux or win
# Prints `conformance: GREEN` or the first `conformance: RED ...` line (ADR 1044).
here="$(cd "$(dirname "$0")" && pwd)"
plat="$1"; shift
[ -n "$plat" ] && [ $# -ge 1 ] || { echo "usage: host_conformance.sh PLATFORM HOST [ARGS...]"; exit 2; }
script="$here/host_conformance.txt"
out="$(mktemp)"; trap 'rm -f "$out"' EXIT
coproc HOST { "$@" 2>/dev/null; }
exec 3>&"${HOST[1]}"
cat <&"${HOST[0]}" > "$out" &
reader=$!
want() { case ",$1," in *,all,*|*",$plat,"*) return 0;; esac; return 1; }
expects=(); absents=(); firsts=(); orders=()
while IFS= read -r line; do
  case "$line" in ''|'#'*) continue;; esac
  p="${line%% *}"; rest="${line#* }"; kind="${rest%% *}"; text="${rest#* }"
  want "$p" || continue
  case "$kind" in
    send) printf '%s\n' "$text" >&3;;
    expect) expects+=("$text");; absent) absents+=("$text");;
    first) firsts+=("$text");; order) orders+=("$text");;
  esac
done < "$script"
for _ in $(seq 1 40); do
  ok=1
  for e in "${expects[@]}"; do grep -qF -- "$e" "$out" || ok=0; done
  [ $ok = 1 ] && break
  sleep 0.5
done
exec 3>&-
for _ in $(seq 1 20); do kill -0 "$HOST_PID" 2>/dev/null || break; sleep 0.5; done
if kill -0 "$HOST_PID" 2>/dev/null; then kill "$HOST_PID" 2>/dev/null; echo "conformance: RED the host did not exit when its input closed"; exit 1; fi
for e in "${expects[@]}"; do grep -qF -- "$e" "$out" || { echo "conformance: RED no output line contains: $e"; exit 1; }; done
for a in "${absents[@]}"; do grep -qF -- "$a" "$out" && { echo "conformance: RED an output line contains: $a"; exit 1; }; done
for f in "${firsts[@]}"; do head -n1 "$out" | grep -qF -- "$f" || { echo "conformance: RED the first line lacks: $f"; exit 1; }; done
for o in "${orders[@]}"; do
  a="${o%%|*}"; b="${o#*|}"
  la="$(grep -nF -- "$a" "$out" | head -1 | cut -d: -f1)"; lb="$(grep -nF -- "$b" "$out" | head -1 | cut -d: -f1)"
  [ -z "$lb" ] && continue
  { [ -n "$la" ] && [ "$la" -lt "$lb" ]; } || { echo "conformance: RED $a does not come before $b"; exit 1; }
done
echo "conformance: GREEN"
