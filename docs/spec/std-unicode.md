# Standard library: `std.unicode`

Chapter key: `std`

> **Status:** Drafted. Written in AIL (`std/unicode/unicode.ail`), ADR 1256.

Unicode text beyond single characters: the four normalization forms of
[UAX #15](https://www.unicode.org/reports/tr15/), case folding and caseless
matching (The Unicode Standard, section 3.13), mark stripping, extended
grapheme clusters and word segmentation
([UAX #29](https://www.unicode.org/reports/tr29/)), display width
([UAX #11](https://www.unicode.org/reports/tr11/)) and line breaking
([UAX #14](https://www.unicode.org/reports/tr14/)). Every
function is pure (`eff.infer.1`) and total: any `str` is valid input and no
call fails or traps. The module is in scope without `use` (`std.use.4`).

`str` methods work on characters (code points, `type.str.1`); this module is
for the cases where that is not what a reader sees: `"é"` can be one
character or `e` and a combining accent, and `"🇫🇷"` is two characters but
one flag.

## Data

**std.unicode.1** — The tables come from one version of the Unicode
Character Database, which `version() str` returns (`"16.0.0"`). Details: [below](#more-on-stdunicode1). *(ADR 1256)*
`tests/cases/std_unicode_conformance.ail` and `tests/cases/std_unicode_segments.ail`
run the same files through the module (every line of the first, grapheme and
word files; every third line of the line break file). A code point that is unassigned in

```ail rules=std.unicode.1
fn main()
    print(unicode.version())    # 16.0.0
```

## Normalization

**std.unicode.2** — `nfd(s str) str` is `s` in Normalization Form D: each
character replaced by its full canonical decomposition, Hangul syllables
decomposed by arithmetic, then the canonical ordering algorithm applied (marks
of a lower combining class move before marks of a higher one, never across a
class-0 character). Details: [below](#more-on-stdunicode2). *(ADR 1256)*

```ail rules=std.unicode.2
fn main()
    print(unicode.nfd("\u{e9}").len())   # 2: e and U+0301
    print(unicode.nfkd("\u{fb01}"))      # fi
    print(unicode.nfkd("x\u{b2}"))       # x2
```

**std.unicode.3** — `nfc(s str) str` is Normalization Form C: `nfd(s)`
followed by canonical composition, which joins each starter with a later
character it is not blocked from into their primary composite (characters
excluded from composition, such as U+0958, stay decomposed). Details: [below](#more-on-stdunicode3). *(ADR 1256)*

```ail rules=std.unicode.3
fn main()
    print(unicode.nfc("e\u{301}") == "\u{e9}")       # true
    print(unicode.nfc("\u{1100}\u{1161}"))           # 가
    print(unicode.nfkc("\u{ff48}\u{ff45}\u{ff4c}\u{ff4c}\u{ff4f}"))   # hello
```

**std.unicode.4** — `is_nfc(s str) bool`, `is_nfd`, `is_nfkc` and `is_nfkd`
are whether `s` already equals its normal form: `is_nfc(s)` is
`nfc(s) == s`. *(ADR 1256)*

```ail rules=std.unicode.4
fn main()
    print(unicode.is_nfc("\u{e9}"))      # true
    print(unicode.is_nfd("\u{e9}"))      # false
    print(unicode.is_nfkc("\u{fb01}"))   # false
```

## Case

**std.unicode.5** — `casefold(s str) str` is `s` with full case folding: each
character replaced by its folding of status C or F in `CaseFolding.txt`
(`ß` to `ss`, `Σ` and `ς` to `σ`, `ǅ` to `ǆ`). Details: [below](#more-on-stdunicode5). *(ADR 1256)*

```ail rules=std.unicode.5
fn main()
    print(unicode.casefold("Stra\u{df}e"))      # strasse
    print(unicode.casefold("\u{3a3}\u{391}\u{3a3}"))   # σασ
```

**std.unicode.6** — `caseless_eq(a str, b str) bool` is the canonical
caseless match of Unicode section 3.13 (D145):
`nfd(casefold(nfd(a))) == nfd(casefold(nfd(b)))`. Details: [below](#more-on-stdunicode6). *(ADR 1256)*

```ail rules=std.unicode.6
fn main()
    print(unicode.caseless_eq("STRASSE", "stra\u{df}e"))   # true
    print(unicode.caseless_eq("\u{c5}", "a\u{30a}"))       # true
    print(unicode.caseless_eq("a", "b"))                   # false
```

## Marks

**std.unicode.7** — `strip_marks(s str) str` is `s` decomposed canonically,
without its nonspacing marks (general category Mn), then composed again
(NFC). Details: [below](#more-on-stdunicode7). *(ADR 1256)*

```ail rules=std.unicode.7
fn main()
    print(unicode.strip_marks("Cr\u{e8}me br\u{fb}l\u{e9}e"))   # Creme brulee
    print(unicode.strip_marks("K\u{f8}benhavn"))               # København
```

## Grapheme clusters

**std.unicode.8** — `graphemes(s str) List[str]` splits `s` into its extended
grapheme clusters by the rules GB3 to GB999 of UAX #29: a base character with
its combining marks, `\r\n`, a Hangul syllable written as jamo, an emoji with
its modifiers or `ZWJ` sequence, a pair of regional indicators (a flag), and
an Indic consonant cluster joined by a virama (GB9c) are each one cluster. Details: [below](#more-on-stdunicode8). *(ADR 1256)*

```ail rules=std.unicode.8
fn main()
    print(unicode.grapheme_count("e\u{301}x"))                        # 2
    print(unicode.grapheme_count("\u{1f1eb}\u{1f1f7}\u{1f1e9}\u{1f1ea}"))   # 2: two flags
    print(unicode.grapheme_count("\u{1f468}\u{200d}\u{1f469}\u{200d}\u{1f467}"))   # 1
    print(unicode.graphemes("\u{928}\u{92e}\u{938}\u{94d}\u{924}\u{947}").len())   # 3
    print(unicode.graphemes("ab").join("|"))                          # a|b
```

### `std.unicode` reference

Details moved out of the numbered rules above, so `ail explain` stays short.

#### More on std.unicode.1

They are written by `tools/native/gen_unicode_text.py`, which checks them against the standard's conformance files (`NormalizationTest.txt`, `GraphemeBreakTest.txt`) before writing; `tests/cases/std_unicode_conformance.ail` runs the same files through the module. A code point that is unassigned in that version has no decomposition, no folding, combining class 0 and grapheme class Other.

#### More on std.unicode.2

`nfkd(s str) str` is Normalization Form KD: the same with compatibility decompositions (`ﬁ` to `fi`, `①` to `1`, `²` to `2`, fullwidth to plain).

#### More on std.unicode.3

`nfkc(s str) str` is Normalization Form KC: `nfkd(s)` followed by canonical composition. Two strings that are canonically equivalent have the same `nfc` and the same `nfd`; compare those, not the strings, when text comes from different sources.

#### More on std.unicode.5

The result is for comparing and keying text without case, not for display: use `s.to_lower()` to show text in lower case. Folding does not depend on language (no Turkish dotless `i` rules).

#### More on std.unicode.6

It ignores case and canonical equivalence, so `"STRASSE"` matches `"straße"` and `"Å"` matches `"a"` followed by U+030A. To key a map by caseless text, store `unicode.nfd(unicode.casefold(unicode.nfd(s)))`.

#### More on std.unicode.7

It removes accents that are marks (`é`, `ü`, `å`) and keeps letters whose shape has no decomposition (`ø`, `ł`, `đ`). It is not a transliteration: non-Latin scripts keep their letters and lose only their marks.

#### More on std.unicode.8

Joining the result gives `s` back; an empty `s` gives `[]`. `grapheme_count(s str) int` is `graphemes(s).len()` without building the pieces. Use these to count, cut or reverse what a reader sees as characters: `unicode.graphemes(s).take(10).join("")` keeps whole clusters.

## Display width

**std.unicode.9** — `char_width(c char) int` is the terminal columns one
character takes: 0 for a control character (code under 32, or 127 to 159, a
tab included), a nonspacing or enclosing mark (general category Mn, Me), a
format character (Cf, such as the zero-width space and joiner), a line or
paragraph separator, and a Hangul medial vowel or final consonant (U+1160 to
U+11FF, U+D7B0 to U+D7FF), which draws inside the syllable before it; 2 for a
character whose East Asian Width ([UAX #11](https://www.unicode.org/reports/tr11/))
is W or F, which includes every emoji with an emoji presentation; 1 for the
rest, ambiguous (A) characters, the soft hyphen (U+00AD), which terminals
draw, and unassigned characters included, except that an unassigned code point
in the CJK ideograph blocks and in planes 2 and 3 is 2, as UAX #11 says. `east_asian_width(c char) str` is the width
class itself: `"N"`, `"A"`, `"H"`, `"Na"`, `"W"` or `"F"`. *(ADR-TBD)*

```ail rules=std.unicode.9
fn main()
    print(unicode.char_width('a'))                  # 1
    print(unicode.char_width('日'))                 # 2
    print(unicode.char_width(char.from_code(0x301) else ' '))   # 0: a combining accent
    print(unicode.east_asian_width('日'))           # W
    print(unicode.east_asian_width('é'))            # A
```

**std.unicode.10** — `width(s str) int` is the columns `s` takes, counted by
extended grapheme cluster (`std.unicode.8`) so that what a terminal draws as
one cell group counts once. A cluster takes the width of its widest character;
two regional indicators (a flag) take 2; a character that has the Emoji
property followed by U+FE0F (the emoji presentation selector) takes 2, as in
`"❤️"`, where `"❤"` alone takes 1; an emoji `ZWJ` sequence takes 2; and an
Indic conjunct (a consonant, a virama and a consonant) takes 1. Escape
sequences are ordinary control characters here: `std.textwrap.width` skips
them first. Terminals differ on sequences; this is the common behaviour of
those that segment by grapheme cluster. `s.len()` counts characters, `grapheme_count` counts clusters, `width` counts
columns. *(ADR-TBD)*

```ail rules=std.unicode.10
fn main()
    print(unicode.width("hello"))                           # 5
    print(unicode.width("日本語"))                          # 6
    print(unicode.width("e\u{301}"))                        # 1
    print(unicode.width("\u{1F1EB}\u{1F1F7}"))              # 2: a flag
    print(unicode.width("\u{2764}\u{FE0F}"))                # 2
    print(unicode.width("\u{1F468}\u{200D}\u{1F469}"))     # 2
```

## Line breaking

**std.unicode.11** — `line_segments(s str) List[str]` cuts `s` into the
pieces after which a line may break, by the line breaking algorithm of
[UAX #14](https://www.unicode.org/reports/tr14/) (rules LB2 to LB31, the
default, untailored behaviour, with the classes AI, SG and XX read as AL, SA
as CM or AL by general category, and CJ as NS, as rule LB1 says). Joining the
result gives `s` back. A piece ends with the spaces that follow its text. A
piece that ends with a newline character (`\n`, `\r\n`, a vertical tab or form
feed, U+0085, U+2028, U+2029) ends a line (a mandatory break); after any other
piece a line may break but need not. The text is not cut inside a word, after
an opening bracket, before a closing one or before a no-break space; Chinese
and Japanese text may break between most characters; `"fox-jumps"` may break
after the hyphen. An empty `s` gives `[]`. To wrap, add pieces to a line while
`width` of the line fits. *(ADR-TBD)*

```ail rules=std.unicode.11
fn main()
    print(unicode.line_segments("The quick fox-jumps").join("|"))   # The |quick |fox-|jumps
    print(unicode.line_segments("a\u{A0}b c").join("|"))            # a b |c  (no break at the no-break space)
    print(unicode.line_segments("\u{65E5}\u{672C}\u{8A9E}").len())   # 3
```

## Word segmentation

**std.unicode.12** — `word_segments(s str) List[str]` cuts `s` at every word
boundary of [UAX #29](https://www.unicode.org/reports/tr29/) (rules WB3 to
WB999): each word, each run of spaces, each punctuation mark and each
ideograph is a segment, so joining the result gives `s` back. `"can't"`,
`"3.14"`, `"foo_bar"` and a letter with its accents are one segment each;
`"Hello, world"` gives `["Hello", ",", " ", "world"]`. The algorithm uses no
dictionary, so text in a script written without spaces (Chinese, Japanese,
Thai) is cut by character class only: Han characters one at a time, a run of
Katakana or of Thai letters whole. `words(s str) List[str]` is the segments
that hold a letter or a digit (an ALetter, Hebrew letter, numeric or Katakana
character, or one for which `c.is_alpha()` holds), in order, so it drops
spaces and punctuation. An empty `s` gives `[]` from both. *(ADR-TBD)*

```ail rules=std.unicode.12
fn main()
    print(unicode.word_segments("Hello, world!").join("|"))        # Hello|,| |world|!
    print(unicode.words("Hello, world! 3.14 can't").join("|"))     # Hello|world|3.14|can't
    print(unicode.words("... ---").len())                          # 0
```
