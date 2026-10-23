#!/usr/bin/env bash
# The doc examples of the AIL `ail test` (tools/ailtest, test.doc.1-3) against the Rust one:
# for each fixture, the same tests under the same names, the same pass or fail, a failure at
# the same line, and the same text for an output mismatch (test.doc.2). Wording of other
# traps differs by design (native versus interpreter), so only their line is compared.
#   tools/ailtest/doctest-parity.sh [FILE...]   default: tools/ailtest/fixtures/*.ail
set -u
cd "$(git rev-parse --show-toplevel)"
files=("$@"); [ ${#files[@]} -eq 0 ] && files=(tools/ailtest/fixtures/*.ail)
norm() { # status, then normalised output lines
  awk '/^  / {next}
       /^[0-9]+ passed/ {sub(/, 0 skipped/, ""); print; next}
       /^ok / {print; next}
       /^skip / {print; next}
       /^FAIL / { if ($0 ~ /trap\[test\.doc\.2\]/) {print; next}
                  name=$2; n=split($0, a, " at "); split(a[n], b, ":"); print "FAIL " name " at " b[1]; next}
       /: error\[/ {print; exit}
       {print}'
}
fail=0; n=0
for f in "${files[@]}"; do
  n=$((n+1))
  ./ail test "$f" >/tmp/dtp-r.out 2>/tmp/dtp-r.err; rc=$?
  ./ail run tools/ailtest/ailtest.ail "$f" >/tmp/dtp-n.out 2>/tmp/dtp-n.err; nc=$?
  if [ $rc = 2 ]; then cat /tmp/dtp-r.err >>/tmp/dtp-r.out; cat /tmp/dtp-n.err >>/tmp/dtp-n.out; fi
  { echo "exit $rc"; norm </tmp/dtp-r.out; } >/tmp/dtp-r.norm
  { echo "exit $nc"; norm </tmp/dtp-n.out; } >/tmp/dtp-n.norm
  if ! diff -u /tmp/dtp-r.norm /tmp/dtp-n.norm >/tmp/dtp.diff; then
    fail=$((fail+1)); echo "DIFF $f"; head -20 /tmp/dtp.diff
  fi
done
[ $fail = 0 ] && echo "DOCTEST PARITY GREEN ($n files)" || echo "DOCTEST PARITY RED ($fail of $n)"
exit $((fail > 0))
