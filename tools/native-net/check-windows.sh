#!/bin/bash
# The Windows twin of tools/native-net/check.sh: the five HTTP client programs, built for Windows, run on ail-win against
# a fresh server.py (it keeps state between requests) on ail-linux, reached as 127.0.0.1:PORT through two ssh tunnels
# (ail-win -> Mac -> ail-linux), so cross-origin cases behave as on one machine. Their output must equal the Linux native
# run against its own fresh server.
#   AIL_LINUX=user@linux-host AIL_WIN=user@windows-host AIL_WIN_DIR='C:\Users\me' [AIL_SSH_KEY=~/.ssh/key] \
#     tools/native-net/check-windows.sh
# AIL_WIN_DIR is a writable directory on the Windows host; the Linux host has an ail checkout at ~/ail.
set -u
: "${AIL_LINUX:?set AIL_LINUX to user@host of the Linux machine}"
: "${AIL_WIN:?set AIL_WIN to user@host of the Windows machine}"
: "${AIL_WIN_DIR:?set AIL_WIN_DIR to a writable directory on the Windows machine, e.g. C:\\Users\\me}"
K="-o BatchMode=yes -o StrictHostKeyChecking=accept-new"
[ -z "${AIL_SSH_KEY:-}" ] || K="-i $AIL_SSH_KEY $K"
LIN=$AIL_LINUX; WIN=$AIL_WIN
WDIR=${AIL_WIN_DIR//\\//}   # forward slashes for scp
fresh_server() { # prints the port
  ssh $K $LIN '[ -f /tmp/nn.pid ] && kill $(cat /tmp/nn.pid) 2>/dev/null; rm -f /tmp/nn.port; cd ~/ail; nohup python3 tools/native-net/server.py > /tmp/nn.port 2>&1 & echo $! > /tmp/nn.pid; sleep 1.5; head -1 /tmp/nn.port'
}
for prog in http_client http_request http_bytes http_jar http_keepalive; do
  ssh $K $LIN "cd ~/ail && ./ail build tools/native-net/$prog.ail -o /tmp/nn_$prog > /dev/null 2>&1; ./ail build tools/native-net/$prog.ail --target x86_64-windows -o /tmp/nn_$prog.exe 2>&1 | tail -2"
  scp $K -q $LIN:/tmp/nn_$prog.exe /tmp/nn_$prog.exe && scp $K -q /tmp/nn_$prog.exe "$WIN:$WDIR/nn_$prog.exe"
  # The Linux run, on its own fresh server.
  PORT=$(fresh_server)
  ssh $K $LIN "/tmp/nn_$prog http://127.0.0.1:$PORT > /tmp/nn_$prog.want 2>&1; true"
  scp $K -q $LIN:/tmp/nn_$prog.want /tmp/nn_$prog.want
  # The Windows run, on another fresh server, through the tunnels.
  PORT=$(fresh_server)
  ssh $K -f -N -L $PORT:127.0.0.1:$PORT $LIN
  ssh $K -f -N -R $PORT:127.0.0.1:$PORT $WIN
  sleep 1
  ssh $K $WIN "cd '$WDIR'; \$j = Start-Job { Set-Location '$WDIR'; cmd /c \"nn_$prog.exe http://127.0.0.1:$PORT 2>&1\" }; if (-not (Wait-Job \$j -Timeout 90)) { 'TIMEOUT'; taskkill /F /IM nn_$prog.exe | Out-Null }; (Receive-Job \$j) -join \"\`n\"" > /tmp/nn_$prog.got 2>&1
  pkill -f "ssh .*-N -[LR] $PORT:" 2>/dev/null
  if diff <(tr -d '\r' < /tmp/nn_$prog.got) /tmp/nn_$prog.want > /tmp/nn_$prog.diff; then echo "SAME $prog"; else echo "DIFF $prog"; head -8 /tmp/nn_$prog.diff; fi
done
ssh $K $LIN 'kill $(cat /tmp/nn.pid) 2>/dev/null; true'
