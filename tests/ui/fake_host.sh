#!/usr/bin/env bash
# a fake std.app host: replies to each view with the next scripted event
read -r first
echo '{"v":1,"kind":"start"}'
echo '{"kind":"resize","id":"grid","dx":640,"dy":480,"cell_w":8,"cell_h":16}'
evs=('{"kind":"key","id":"grid","text":"a","shift":true}' '{"kind":"click","id":"grid","x":50,"y":50,"text":"left"}' '{"kind":"scroll","id":"grid","dx":0,"dy":48}' '{"kind":"resize","id":"grid","dx":400,"dy":240,"cell_w":8,"cell_h":16}' '{"kind":"close"}')
i=0
while read -r line; do
  echo "$line" | grep -q '"t":"view"' || continue
  if [ $i -lt ${#evs[@]} ]; then echo "${evs[$i]}"; i=$((i+1)); fi
done
