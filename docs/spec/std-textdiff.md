# Standard library: `std.textdiff`

Chapter key: `std`

> **Status:** Drafted (ADR 1218; rules 11-12: ADR 1268). Written in AIL (`std/textdiff/textdiff.ail`).

Differences between two texts: a shortest line diff, a patience line diff, a word diff, `diff -u` output, applying
such a patch, and the edit distance between two strings. The module is named
`textdiff` because `diff` is a common local name (`std.use.4`). Every function
is pure (`eff.infer.1`).

**std.textdiff.1** — `lines(a str, b str) List[Edit]` gives the lines of `a`
and `b` as one script, in order: `Keep` for a line in both, `Remove` for a line
only in `a`, `Add` for a line only in `b`. The script is shortest (Myers'
algorithm): no other script has fewer `Add` plus `Remove` lines. Within a run of
changes between two `Keep` lines, every `Remove` comes before every `Add`, as
`diff` writes them. When several shortest scripts exist, which one is given is
fixed but not specified.

**std.textdiff.2** — `Edit` has `kind EditKind`, `text str` (the line without
its line break), `a int`, `b int` and `nl bool`. `EditKind` is `Keep`, `Add` or
`Remove`. `a` is the 1-based number of the line in the old text and is 0 for an
`Add`; `b` is the number in the new text and is 0 for a `Remove`. A `Keep` has
both. `nl` is false only for a last line that has no line break.

**std.textdiff.3** — A text is split at `\n`. A final `\n` adds no empty line,
and an empty text has no lines. A last line without a `\n` is a different line
from the same text with one, so `"a\nb"` against `"a\nb\n"` is one `Remove`
and one `Add`, as in `diff`. A `\r` is part of its line.

```ail rules=std.textdiff.1,std.textdiff.2,std.textdiff.3
use std.textdiff

fn main()
    for e in textdiff.lines("a\nb\nc\n", "a\nx\nc\nd\n")
        print("{e.kind.name()} {e.a} {e.b} {e.text}")
    # Keep 1 1 a, Remove 2 0 b, Add 0 2 x, Keep 3 3 c, Add 0 4 d
    last := textdiff.lines("a\nb", "a\nb\n")[1]
    print("{last.kind.name()} {last.nl}")   # Remove false
```

**std.textdiff.4** — `unified(a str, b str, name_a str, name_b str, context int)
str` is the diff of `a` against `b` in the unified format of `diff -u`: the
lines `--- name_a` and `+++ name_b`, then one `@@ -A,L +B,L @@` block per group
of changes, with `context` unchanged lines before and after each change (a
negative `context` is 0). Changes separated by at most `2 * context` unchanged
lines share a block. A length of 1 is written without `,L`, and an empty side
starts at the line before it (`-0,0`). A last line without a `\n` is followed by
`\ No newline at end of file`. There are no timestamps. Equal texts give the
empty string, with no header.

```ail rules=std.textdiff.4
use std.textdiff

fn main()
    patch := textdiff.unified("one\ntwo\nthree\n", "one\n2\nthree\nfour\n", "old.txt", "new.txt", 1)
    print(patch)
    # --- old.txt
    # +++ new.txt
    # @@ -1,3 +1,4 @@
    #  one
    # -two
    # +2
    #  three
    # +four
```

**std.textdiff.5** — `stat(a str, b str) (int, int)` is the number of lines
added and the number removed, in that order: the `Add` and `Remove` lines of
`lines(a, b)`.

```ail rules=std.textdiff.5
use std.textdiff

fn main()
    s := textdiff.stat("a\nb\nc\n", "a\nx\nc\nd\n")
    print("added {s.0}, removed {s.1}")   # added 2, removed 1
```

**std.textdiff.6** — `distance(a str, b str) int` is the Levenshtein distance
between the two strings, counted in characters (not bytes): the fewest
insertions, deletions and substitutions of one character that turn `a` into
`b`.

**std.textdiff.7** — `similarity(a str, b str) float` is `1.0` minus
`distance(a, b)` over the character length of the longer string: 1.0 for equal
strings (two empty strings included), 0.0 when no character can stay.

```ail rules=std.textdiff.6,std.textdiff.7
use std.textdiff

fn main()
    print(textdiff.distance("kitten", "sitting"))   # 3
    sim := textdiff.similarity("kitten", "sitting")
    print("{sim:.2}")                               # 0.57
```

**std.textdiff.8** — `apply(text str, patch str) str ! PatchError` applies a
unified patch for one file, as `unified` or `diff -u` writes it, and gives the
new text. Lines before the first `@@` (`---`, `+++`, `diff` lines) are skipped.
Each block's context and removed lines must be in `text` exactly, at the line
its header gives, and blocks must be in order and not overlap. There is no
search for another offset and no fuzz. A `\ No newline at end of file` line
marks the line before it as having no line break. `apply(a, unified(a, b, ..))`
is `b` for every `a` and `b`.

```ail rules=std.textdiff.8
use std.textdiff

fn main() !
    old := "red\ngreen\nblue\n"
    fresh := "red\nviolet\nblue\n"
    patch := textdiff.unified(old, fresh, "a", "b", 3)
    print(textdiff.apply(old, patch)? == fresh)   # true
```

**std.textdiff.9** — `PatchError` is an error with `kind PatchKind`, `why str`
and `hunk int` (1-based). Its text is `why (hunk N)`. `PatchKind` is `Malformed`
for a patch that is not a unified patch (a bad `@@` header, a hunk line that
starts with something else than a space, `+` or `-`, a hunk shorter or longer
than its header says, a line after the last hunk, blocks out of order) and
`Mismatch` for a patch that does not fit `text` (a context or removed line that
is not there, a block that starts after the last line).

```ail rules=std.textdiff.9
use std.textdiff

fn main()
    r := textdiff.apply("a\nb\n", "@@ -1,2 +1,2 @@\n a\n-c\n+d\n") else err
        print("{err.kind.name()}: {err}")   # Mismatch: line 2 does not match (hunk 1)
        ""
    print(r.len())
```

**std.textdiff.10** — `lines`, `unified` and `stat` take time proportional to
`(N + M) * D` for `N` and `M` lines and `D` changes, and room proportional to
`D * D`; lines the texts share at the start and the end cost nothing more. `distance` takes time
proportional to the product of the two lengths after the shared start and end
are dropped.

```ail rules=std.textdiff.10
use std.textdiff

fn main()
    mut a := List[str]()
    for i in 0..2000
        a.push("line {i}")
    old := a.join("\n")
    a[1000] = "changed"
    edits := textdiff.lines(old, a.join("\n"))   # one change: fast however long the texts are
    print(edits.len())
```

**std.textdiff.11** — `words(a str, b str) List[Edit]` diffs two texts as words. Details: [below](#more-on-stdtextdiff11).

```ail rules=std.textdiff.11
use std.textdiff

fn main()
    for e in textdiff.words("the quick fox", "the slow fox")
        print("{e.kind.name()} [{e.text}]")
    # Keep [the], Keep [ ], Remove [quick], Add [slow], Keep [ ], Keep [fox]
```

**std.textdiff.12** — `patience(a str, b str) List[Edit]` diffs lines as
`std.textdiff.1` does, in the same form and with the same line splitting, by the
patience method: lines that occur exactly once in each text, and in the same
order, are fixed as `Keep`; the stretches between them are diffed the same way,
and a stretch with no such line is diffed as in `std.textdiff.1`. Details: [below](#more-on-stdtextdiff12).

```ail rules=std.textdiff.12
use std.textdiff

fn main()
    for e in textdiff.patience("A\nB\nC\nD\n", "A\nC\nB\nD\n")
        print("{e.kind.name()} {e.text}")
    # Keep A, Remove B, Keep C, Add B, Keep D
```

Rejected alternatives for 11 and 12: a `patience` flag on `lines` (a second
result shape and a bool argument an agent must remember; two names are searched
by their words); word diff as a `lines` mode on a split text (loses whitespace,
so the texts cannot be rebuilt); histogram diff (not asked for, less known).

### `std.textdiff` reference

Details moved out of the numbered rules above, so `ail explain` stays short.

#### More on std.textdiff.11

A word is a maximal run of non-whitespace characters or a maximal run of whitespace characters, so joining the `text` of every `Keep` and `Remove` gives `a` and joining every `Keep` and `Add` gives `b`. The script is shortest and ordered as in `std.textdiff.1`. `text` is the word as it is (a whitespace word may hold a `\n`), `a` and `b` count words from 1 as in `std.textdiff.2`, and `nl` is false.

#### More on std.textdiff.12

The script is correct (`Keep` and `Remove` give `a`, `Keep` and `Add` give `b`) but not always shortest; it follows the structure of the text, such as moved blocks between unique lines. It takes time proportional to that of `lines` plus `N log N` for `N` lines.
