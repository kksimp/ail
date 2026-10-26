#!/usr/bin/env bash
# Runs the agent-facing commands on a small program with a bug and prints what each one says, so
# gaps in what an agent fixing a bug sees can be read off one transcript.
#   tools/agent/survey.sh            from the repository root (needs ./ail built); nothing is written outside a temp dir
cd "$(git rev-parse --show-toplevel)" || exit 1
ail="$PWD/ail"
d=$(mktemp -d); trap 'rm -rf "$d"' EXIT
mkdir "$d/p"; cd "$d/p" || exit 1
git init -q . && git config user.email a@b && git config user.name a
cat > main.ail <<'A'
struct Cart
    total int

fn add(mut c Cart, n int)
    c.total += n - 1

fn show(c Cart) str
    "{c.total}"

fn twice(mut c Cart, n int)
    add(mut c, n)
    add(mut c, n)

@test
fn adds()
    mut c := Cart(total: 0)
    add(mut c, 2)
    assert_eq(c.total, 2)

@test
fn shows()
    assert_eq(show(Cart(total: 1)), "1")

@test
fn doubles()
    mut c := Cart(total: 0)
    twice(mut c, 3)
    assert_eq(c.total, 6)

fn main()
    mut c := Cart(total: 0)
    twice(mut c, 5)
    print(show(c))
A
git add . && git commit -qm base
step() { echo; echo "=== \$ ail $*"; "$ail" "$@" 2>&1 | head -${LINES_MAX:-90}; echo "[exit ${PIPESTATUS[0]}]"; }
step test main.ail
step api main.ail
step ask main.ail --help
echo '[{"op":"replace_body","id":"fn:add","body":"c.total += n"}]' > e.json
step edit --dry-run --impact --diff main.ail < e.json
"$ail" edit main.ail < e.json >/dev/null 2>&1
step diff --rev HEAD main.ail
step ask main.ail --affected --json
step test --affected main.ail
# a formatting-only change
sed -i 's/c.total += n$/c.total  +=  n/' main.ail
step ask main.ail --affected --json
git checkout -q main.ail
# a signature change: the callers break
echo '[{"op":"replace","id":"fn:add","text":"fn add(mut c Cart, n int, by int)\n    c.total += n * by"}]' > s.json
step edit --dry-run --impact main.ail < s.json
# the new version does not check
sed -i 's/c.total += n - 1/c.total += nope/' main.ail
step ask main.ail --affected --json
step diff --rev HEAD main.ail
git checkout -q main.ail
echo; echo "=== done"
