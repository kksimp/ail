#!/usr/bin/env bash
# Runner job: the network interop checks on ail-linux, against real implementations.
#   net-job.sh SHA [module ...]      modules default to: smtp sse s3 oauth2
# It only SSHes to the VM (no build here). It checks out SHA in its own worktree ~/ail-wt/net-interop, pulls the docker
# images (chrislusf/seaweedfs for s3, oryd/hydra:v2.3.0 for oauth2) once, and runs tests/interop/run.sh. smtp and sse need no docker
# (aiosmtpd, curl, requests and sseclient from the venv). Prints the tip it tested and one PASS/FAIL line per case.
set -u
SHA=${1:?usage: net-job.sh SHA [module ...]}; shift
ssh -i ~/.ssh/ail_lanes -o BatchMode=yes kaleb@192.168.4.145 bash -s -- "$SHA" "$@" <<'REMOTE'
set -u
SHA=$1; shift
MODS=${*:-smtp sse s3 oauth2}
if ! command -v docker >/dev/null; then sudo apt-get install -y -q docker.io >/dev/null 2>&1; fi
sudo apt-get install -y -q python3-venv curl openssl >/dev/null 2>&1
sudo systemctl start docker >/dev/null 2>&1
if docker info >/dev/null 2>&1; then export DOCKER=docker; else export DOCKER="sudo docker"; fi
WT=$HOME/ail-wt/net-interop
git -C ~/ail fetch -q origin "$SHA" 2>/dev/null || git -C ~/ail fetch -q origin
mkdir -p ~/ail-wt
if [ -d "$WT" ]; then git -C "$WT" reset -q --hard; git -C "$WT" clean -qfd tests/interop; git -C "$WT" checkout -q --detach "$SHA" || exit 2
else git -C ~/ail worktree add -q --detach "$WT" "$SHA" || exit 2; fi
cd "$WT" || exit 2
echo "tip $(git rev-parse HEAD) docker=$DOCKER"
for img in chrislusf/seaweedfs oryd/hydra:v2.3.0; do $DOCKER image inspect $img >/dev/null 2>&1 || $DOCKER pull -q $img >/dev/null 2>&1 || echo "pull failed: $img"; done
tests/interop/run.sh $MODS
$DOCKER ps -aq --filter name=ail-interop | xargs -r $DOCKER rm -f -v >/dev/null 2>&1
REMOTE
