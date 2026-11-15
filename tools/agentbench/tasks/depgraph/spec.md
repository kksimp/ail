# depgraph

Write `depgraph`, a command-line program. It reads a dependency list from stdin and answers one question about it.

Input, one item per line:
- `A -> B` means A needs B.
- `A` alone declares a node with no dependencies yet.
- An empty line or a line starting with `#` is ignored.
- A name is one or more of `a-z A-Z 0-9 _ . -`. The separator is exactly one space, `->`, one space. Any other line (extra or missing spaces, a third name, an illegal character, a carriage return) is bad: skip it and print `error: line K: bad line` to stderr (K is 1-based). Repeated edges count once. `A -> A` is a self-dependency. There are at most 500 distinct names.

Commands (first argument), output on stdout, one name per line:
- `order`: every node, dependencies first. Among nodes ready at the same time, the smallest name in byte order comes first.
- `cycles`: every node that lies on a cycle (reaches itself through one or more edges), in ascending byte order.
- `deps NAME`: everything NAME needs, directly or not, ascending. NAME itself appears only if it lies on a cycle.
- `rdeps NAME`: everything that needs NAME, directly or not, ascending, under the same rule.

Exit codes, first that applies wins: 1 for a missing, unknown or malformed command, including a wrong number of arguments and a NAME argument that is not a valid name by the rule above (print `usage: depgraph order|cycles|deps NAME|rdeps NAME` to stderr, do not read stdin); 4 if `deps`/`rdeps` names a node that is not in the input (print `error: unknown ` and the name to stderr, e.g. `error: unknown lib`, nothing on stdout); 3 if `order` finds a cycle (print `error: cycle` to stderr, nothing on stdout); 2 if any line was bad (stdout is still printed); else 0.

Layout: sources live in `src/` (`src/main.ail`, `src/main.c` or `src/main.rs`); `./build` builds `./depgraph`; `./test` runs the visible cases.
