"""Lengthens every identifier in a source file (outside strings and comments) so
that lines wrap: `lengthen.py IN OUT`. Used by parity.sh; the output isn't a
meaningful program, only a formatter input."""
import re, sys

KW = set("and any as break continue defer elif else ensures enum error extern false fn for if impl in invariant match move mut none not or par pub ref requires return select self spawn struct todo trait true unique unsafe use while with then int str bool float char".split())
out, in_multiline = [], False
for line in open(sys.argv[1]).read().split("\n"):
    if '"""' in line:
        if line.count('"""') == 1:
            in_multiline = not in_multiline
        out.append(line)
    elif in_multiline or '"' in line or "'" in line or "#" in line or line.strip() == "":
        out.append(line)
    else:
        out.append(re.sub(r"\b([a-z_][a-z_0-9]{2,})\b", lambda m: m.group(1) if m.group(1) in KW else m.group(1) + "_longer", line))
open(sys.argv[2], "w").write("\n".join(out))
