#!/usr/bin/env bash
# Runner job: run the interop checks on ail-linux for a commit. Usage: linux-job.sh SHA [module ...]
# It only SSHes to the VM (no build here), prints the tip it tested and the summary.
set -u
SHA=${1:?usage: linux-job.sh SHA [module ...]}; shift
ssh -i ~/.ssh/ail_lanes -o BatchMode=yes kaleb@192.168.4.145 bash -s -- "$SHA" "$@" <<'REMOTE'
set -u
SHA=$1; shift
sudo apt-get install -y -q python3-venv zstd brotli tar >/dev/null 2>&1
cd ~/ail && git fetch -q origin "$SHA" 2>/dev/null || git fetch -q origin
git checkout -q --detach "$SHA" || exit 2
echo "tip $(git rev-parse HEAD)"
tests/interop/run.sh "$@"
REMOTE
