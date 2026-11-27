#!/usr/bin/env bash
# Runner job: the database interop checks on ail-linux, against real servers in docker.
#   db-job.sh SHA [module ...]      modules default to: redis postgres mysql mariadb
# It only SSHes to the VM (no build here). It checks out SHA there; when std/db/{redis,postgres,mysql}.ail are not
# in that tree yet (their PRs #1563, #1546 and #1559 are unmerged) it merges the PR heads in, keeping both sides of
# the one-line registration conflicts (merge=union). It works in its own worktree ~/ail-wt/db-interop. Prints the tip it tested and the
# PASS/FAIL line of every case. Needs the docker images redis:7, postgres:16, mysql:8.4 and mariadb:11 (pulled once).
set -u
SHA=${1:?usage: db-job.sh SHA [module ...]}; shift
ssh -i ~/.ssh/ail_lanes -o BatchMode=yes kaleb@192.168.4.145 bash -s -- "$SHA" "$@" <<'REMOTE'
set -u
SHA=$1; shift
MODS=${*:-redis postgres mysql mariadb}
if ! command -v docker >/dev/null; then sudo apt-get install -y -q docker.io python3-venv >/dev/null 2>&1; fi
sudo systemctl start docker >/dev/null 2>&1
if docker info >/dev/null 2>&1; then export DOCKER=docker; else export DOCKER="sudo docker"; fi
sudo apt-get install -y -q python3-venv >/dev/null 2>&1
# its own worktree: ~/ail (the shared checkout of the gates) is only fetched into, never reset or checked out
WT=$HOME/ail-wt/db-interop
git -C ~/ail fetch -q origin "$SHA" 2>/dev/null || git -C ~/ail fetch -q origin
mkdir -p ~/ail-wt
if [ -d "$WT" ]; then git -C "$WT" reset -q --hard; git -C "$WT" clean -qfd tests/interop; git -C "$WT" checkout -q --detach "$SHA" || exit 2
else git -C ~/ail worktree add -q --detach "$WT" "$SHA" || exit 2; fi
cd "$WT" || exit 2
printf '* merge=union\n' > "$WT.attrs"
for pr in "1563 redis" "1546 postgres" "1559 mysql"; do
  set -- $pr
  [ -f "std/db/$2.ail" ] && continue
  git fetch -q origin "pull/$1/head" && git -c core.attributesFile="$WT.attrs" -c user.name=job -c user.email=job@localhost merge -q --no-edit FETCH_HEAD >/dev/null 2>&1 || { echo "MERGE FAIL #$1"; git merge --abort 2>/dev/null; }
done
echo "tip $(git rev-parse HEAD) (merged: $(git log --merges --oneline $SHA..HEAD | wc -l) PR heads) docker=$DOCKER"
for img in redis:7 postgres:16 mysql:8.4 mariadb:11; do $DOCKER image inspect $img >/dev/null 2>&1 || $DOCKER pull -q $img >/dev/null 2>&1 || echo "pull failed: $img"; done
tests/interop/run.sh $MODS
$DOCKER ps -aq --filter name=ail-interop | xargs -r $DOCKER rm -f -v >/dev/null 2>&1
REMOTE
