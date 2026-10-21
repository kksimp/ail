# Standard library: `std.regex`

Chapter key: `std`

> **Status:** Drafted. Specified and implemented in the bootstrap (ADRs
> 0438–0441). The Unicode classes are a documented subset (`std.regex.10`).

Regular expressions for text in AIL programs. The engine runs in time
proportional to the pattern times the text, so no pattern is a denial of
service (`std.regex.4`). The syntax is the RE2 subset below. Backreferences
and lookaround are not in it, and say so with their own rule
(`std.regex.21`).

## The module

**std.regex.1** — `use std.regex` provides `compile`, `is_match`, `find`,
`find_all`, `group_all`, `captures`, `replace`, `replace_first`, `split` and `escape`, and
the types `Regex`, `Match`, `Captures` and `RegexError`. Every function is
pure (`eff.infer.1`): it has no effects and needs no capability. The
functions take the `Regex` first: `regex.find(re, text)`. *(ADR 0440)*

**std.regex.2** — `compile(pat str) Regex ! RegexError` checks `pat` and
returns a `Regex`, which holds the pattern as its field `pat`. A pattern
that does not compile fails with a `RegexError` (`std.regex.11`–`22`). A
`Regex` is a value: copies are free, because the interpreter keeps the
compiled program by pattern text. Making a `Regex` by hand
(`regex.Regex(pat: "(")`) with a pattern that does not compile traps at its
first use. *(ADR 0440)*

**std.regex.3** — The pattern syntax is the following; everything else is a
compile-time error of a kind below. `.` is any character but `\n` (`(?s)`
makes it any); a literal character matches itself; `\` before any
punctuation character makes it a literal. Classes: `[abc]`, `[^abc]`,
`[a-z]`, `[[:alpha:]]` and `[[:^alpha:]]`, with `\d \w \s \D \W \S \pL \p{Name}
\PL` inside (`]` first in a class is a literal). Escapes: `\n \t \r \f \v \a`
and `\x41`, `\x{263A}`. Anchors: `^ $` (start and end of the text, or of a
line under `(?m)`), `\A`, `\z`, `\b`, `\B`. Repetition: `* + ?` and
`{n}`, `{n,}`, `{n,m}` with a count of at most 1000, each with a lazy form
that ends in `?`. A `{` that does not start a valid count is a literal.
Alternation `|`; groups `(...)`, `(?:...)`, `(?P<name>...)` and
`(?<name>...)`; flags `(?i)`, `(?s)`, `(?m)` and their negations, alone
(until the end of the group) or as `(?i:...)`. A pattern that compiles to
more than 50 000 instructions, or nests groups more than 200 deep, is too
large. *(ADR 0439)*

**std.regex.4** — A match is the leftmost one. Among matches that start at
the same place, the one the pattern prefers wins: alternatives left to right,
greedy repetition before lazy. This is the choice a backtracking engine
makes, and it is found without backtracking, in time proportional to the
pattern size times the text length. A repetition never runs an iteration that
adds nothing: where a loop would re-enter at the position where its previous
iteration began, that thread stops, so `(a*)*` on `b` matches the empty
string with group 1 not taking part. *(ADR 0438)*

**std.regex.5** — A `Match` has `start` and `end`, the 0-based character
offsets of the half-open span (the same unit as `str.len`, `json.err.1` and
`text[i]`), and `text`, the matched substring. `find(re, text) Match?` is the
first match and `none` when there is none. `is_match(re, text) bool` says
whether there is one. *(ADR 0440)*

**std.regex.6** — `find_all(re, text) List[Match]` is every match, left to
right. After a match the search continues where it ended; after an empty match
it continues one character later. An empty match that begins where the
previous match ended is not reported. No two matches overlap. `a*` on
`baaac` has the matches at 0–0, 1–4 and 5–5. *(ADR 0441)*

**std.regex.7** — `replace(re, text, rep) str` replaces every match from
`find_all` (`std.regex.6`) with `rep`, and `replace_first` only the first.
In `rep`, `$name`, `${name}`, `$1` and `${1}` stand for a group's text, with
`$0` the whole match; a group that took no part gives the empty string; `$$`
is a `$`; a `$` followed by nothing else is a literal `$`. `$name` takes the
longest run of letters, digits and `_`, so write `${1}x`, not `$1x`. Naming a
group the pattern does not have traps. *(ADR 0441)*

**std.regex.8** — `split(re, text) List[str]` is the text between the matches
of `find_all`. A match that is empty at the start of the text, or ends there,
gives no piece; the text after the last match is a piece unless the last match
is empty and at the end. No match gives the whole text. `,` splits `a,b,` into
`a`, `b` and the empty string; the empty pattern splits `abc` into `a`, `b`
and `c`. *(ADR 0441)*

**std.regex.9** — `captures(re, text) Captures?` is the first match with its
groups: `start`, `end`, `groups`, a `List[str?]` in which element 0 is the
whole match and element `i` is group `i` counted by its opening parenthesis
(`none` when it took no part), and `named`, a `Map[str, str]` of the named
groups that took part. *(ADR 0440)*

**std.regex.10** — Flags: `i` ignores case by simple case mapping (one
character to one character); `s` lets `.` match `\n`; `m` makes `^` and `$`
match at lines. `\d`, `\w`, `\s`, `\b` and the `[[:name:]]` classes are
ASCII, as in RE2. `\p{Name}` and `\P{Name}` (also `\pL`, `\p{^L}`) take a
Unicode class from this list: `L Lu Ll N Nd P S Z Zs Cc`, the long names
`Letter Uppercase_Letter Lowercase_Letter Number Decimal_Number Punctuation
Symbol Separator Alphabetic White_Space Any`, and the scripts `Latin Greek
Cyrillic Han Hiragana Katakana Hangul Arabic Hebrew Thai Devanagari`. The
bootstrap builds these from the character properties of its standard library
and fixed ranges, so a few rarely used characters may be classed differently
than in the Unicode tables; any other name is `bad_class` (`std.regex.18`).
`escape(s str) str` puts a `\` before every character that is special in a
pattern. *(ADR 0439)*

```ail
use std.regex

fn main()
    date := regex.compile(r"(?P<y>\d{4})-(?P<m>\d\d)") else return
    print(regex.find(date, "on 2026-10-20"))                # regex.Match(start: 3, end: 10, text: "2026-10")
    print(regex.replace(date, "2026-10", "$m/$y"))          # 10/2026
    sep := regex.compile(r",\s*") else return
    print(regex.split(sep, "a, b,c"))                       # ["a", "b", "c"]
```

## Errors

**std.regex.11** — `kind` is `unclosed_group` when a `(` has no `)`; `at` is
the `(`. Add the `)`, or write `\(` for a literal. *(ADR 0439)*

```ail rules=std.regex.11
use std.regex

fn main()
    _ := regex.compile(r"(ab") else err
        print(err.kind)
        return
```

**std.regex.12** — `kind` is `unopened_group` when a `)` has no `(`; `at` is
the `)`. Remove it, or write `\)`. *(ADR 0439)*

```ail rules=std.regex.12
use std.regex

fn main()
    _ := regex.compile(r"ab)") else err
        print(err.kind)
        return
```

**std.regex.13** — `kind` is `unclosed_class` when a `[` has no `]`; `at` is the
`[`. Add the `]`, or write `\[`. *(ADR 0439)*

```ail rules=std.regex.13
use std.regex

fn main()
    _ := regex.compile(r"[ab") else err
        print(err.kind)
        return
```

**std.regex.14** — `kind` is `bad_escape` for a `\` that does not start an
escape of `std.regex.3`: a trailing `\`, `\q` and other letters, `\x` without
hex digits, or a code that is not a character. Write `\\` for a backslash.
*(ADR 0439)*

```ail rules=std.regex.14
use std.regex

fn main()
    _ := regex.compile(r"a\q") else err
        print(err.kind)
        return
```

**std.regex.15** — `kind` is `bad_repeat` for a count above 1000, a count
whose lower bound is above its upper, or a repetition that follows another
(`a**`, `a{2}{3}`). Group the first with `(?:...)`. *(ADR 0439)*

```ail rules=std.regex.15
use std.regex

fn main()
    _ := regex.compile(r"a**") else err
        print(err.kind)
        return
```

**std.regex.16** — `kind` is `nothing_to_repeat` for a `*`, `+` or `?` with no
expression before it, such as at the start of the pattern or after `|` or
`(`. *(ADR 0439)*

```ail rules=std.regex.16
use std.regex

fn main()
    _ := regex.compile(r"*a") else err
        print(err.kind)
        return
```

**std.regex.17** — `kind` is `bad_range` for a class range that runs backwards
(`[z-a]`) or ends in a set (`[a-\d]`). *(ADR 0439)*

```ail rules=std.regex.17
use std.regex

fn main()
    _ := regex.compile(r"[z-a]") else err
        print(err.kind)
        return
```

**std.regex.18** — `kind` is `bad_class` for a `[[:name:]]` or `\p{Name}` that
is not in the list of `std.regex.10`. The message lists the supported names.
*(ADR 0439)*

```ail rules=std.regex.18
use std.regex

fn main()
    _ := regex.compile(r"[[:nope:]]") else err
        print(err.kind)
        return
```

**std.regex.19** — `kind` is `bad_group_name` for a group name that is empty,
starts with a digit, has a character other than letters, digits and `_`, is
not closed by `>`, or is used by two groups. *(ADR 0439)*

```ail rules=std.regex.19
use std.regex

fn main()
    _ := regex.compile(r"(?P<1a>x)") else err
        print(err.kind)
        return
```

**std.regex.20** — `kind` is `bad_flag` for a flag other than `i`, `s` and
`m`. *(ADR 0439)*

```ail rules=std.regex.20
use std.regex

fn main()
    _ := regex.compile(r"(?x)a") else err
        print(err.kind)
        return
```

**std.regex.21** — `kind` is `unsupported` for backreferences (`\1`,
`(?P=name)`) and lookaround (`(?=`, `(?!`, `(?<=`, `(?<!`). They cannot be
matched in time proportional to the text, which is `std.regex.4`'s promise.
Match the shape with groups and test the result in AIL instead. *(ADR 0438)*

```ail rules=std.regex.21
use std.regex

fn main()
    _ := regex.compile(r"(a)\1") else err
        print(err.kind)
        return
```

**std.regex.22** — `kind` is `too_large` for a pattern over the limits of
`std.regex.3`. Use smaller counts or fewer nested counted groups. *(ADR 0439)*

```ail rules=std.regex.22
use std.regex

fn main()
    _ := regex.compile(r"((a{100}){100}){100}") else err
        print(err.kind)
        return
```

**std.regex.23** — A `RegexError` has `pattern`, `at` (the 0-based character
offset in the pattern), `kind` (the short names of `std.regex.11`–`22`),
`rule` (the rule that explains the kind) and `message`. *(ADR 0440)*

```ail rules=std.regex.23
use std.regex

fn main()
    _ := regex.compile(r"(ab") else err
        print(err.rule)
        return
```
**std.regex.24** — `group_all(re, text, group int) List[str]` is the text of
group `group` in each match of `find_all` (`std.regex.6`), in the same order,
with `group` counted as in `std.regex.9` (`0` is the whole match). A group
that took no part in a match gives the empty string. `group` must name a group
the pattern has: a `group` below 0, or above the last group, traps. The
pattern `code=(\d+)` gives the digits of each `code=NNN`, with no prefix to
strip. *(ADR 0667)*

```ail tool=run,FILE expect="b" rules=std.regex.24
use std.regex

fn main()
    re := regex.compile(r"(\d)(\w)") else err
        print(err.kind)
        return
    print(regex.group_all(re, "1a 2b", 2))
```
