# Standard library: `std.i18n`

Chapter key: `std`

> **Status:** Drafted. Written in AIL (`std/i18n/i18n.ail`, ADR 1273).

Message catalogs for programs that speak more than one language: read gettext `.po` and `.mo` files or a JSON
file, look a message up by its source text, pick the plural form of a count by the locale's rule, fall back from
`fr-CA` to `fr` to `en`, and fill `{name}` holes. Every function is pure (`eff.infer.1`); reading the files is
the caller's job (`fs.read_text`, `fs.read_bytes`).

## The module

**std.i18n.1** — `i18n` is in scope without `use` (`std.use.4`); a local named `i18n` hides it. A `Catalog` holds
the messages of one locale: `locale` (a tag with `-`, so `fr_CA` reads `fr-CA`), `rule` (a `PluralRule`, `std.i18n.5`)
and the messages. `i18n.catalog(locale)` makes an empty one, `c.add(ctx, msgid, forms)` adds or replaces a message
(`ctx` is `""` for none, `forms` is one text per plural form), `c.len()` counts the messages and `c.named(locale)`
is a copy with another locale; a rule that came from a file stays, one from the locale's table follows the new
locale. *(ADR 1273)*

```ail rules=std.i18n.1
fn main()
    mut c := i18n.catalog("de")
    c.add("", "Save", ["Speichern"])
    print("{c.locale} {c.len()} {c.rule.nplurals}")    # de 1 2
```

**std.i18n.2** — `c.tr(msgid)` is the translation of `msgid`, or `msgid` itself when the catalog has none or the
translation is empty. The source text is the key, so an untranslated program still reads correctly.
`c.lookup("", msgid)` gives `str?` when the caller needs to know. *(ADR 1273)*

```ail rules=std.i18n.2
fn main()
    mut c := i18n.catalog("de")
    c.add("", "Save", ["Speichern"])
    print(c.tr("Save"))      # Speichern
    print(c.tr("Cancel"))    # Cancel
```

**std.i18n.3** — `c.trc(ctx, msgid)` looks up `msgid` in the context `ctx` (gettext `msgctxt`), so a word that
translates differently in two places has two entries. A message with a context is not found by `tr`, and the
reverse. *(ADR 1273)*

```ail rules=std.i18n.3
fn main()
    mut c := i18n.catalog("de")
    c.add("menu", "Open", ["Öffnen"])
    print(c.trc("menu", "Open"))    # Öffnen
    print(c.tr("Open"))             # Open
```

**std.i18n.4** — `c.trn(one, other, n)` is the form of the plural message `one` that the catalog's rule picks for
the count `n`; `c.trnc(ctx, one, other, n)` is the same in a context. `one` is the key, as `msgid` is in
gettext. With no translation the English pair answers: `one` when `n` is 1, `other` otherwise. A message with
fewer forms than the rule asks for uses its last form. Put the count in the text with `{count}` and
`format` (`std.i18n.13`). *(ADR 1273)*

```ail rules=std.i18n.4
fn main()
    mut c := i18n.catalog("fr")
    c.add("", "file", ["fichier", "fichiers"])
    print(c.trn("file", "files", 0))    # fichier
    print(c.trn("file", "files", 2))    # fichiers
    print(c.trn("dog", "dogs", 2))      # dogs
```

**std.i18n.5** — `i18n.plural_rule(locale)` is a `PluralRule` with `nplurals int` and `expr str`, a gettext
expression over `n`, chosen by the language part of the locale (`pt_BR` is `pt`). `r.index(n)` is the form
index, 0 up to `nplurals - 1`. The table covers `ja ko zh vi th id ms my lo km` (one form, `0`), `fr tr hi bh ak ln mg ti`
(`n > 1`), `ru uk be sr hr bs`, `pl`, `cs sk`, `ar`, `lt`, `lv`, `sl`, `ro`, `ga` and `cy`; every other language has the
English rule `n != 1`. These are the gettext rules, which can differ from CLDR categories (Turkish has
`n > 1` here). A negative count is read as its absolute value. *(ADR 1273)*

```ail rules=std.i18n.5
fn main()
    r := i18n.plural_rule("ru")
    print("{r.nplurals} {r.index(1)} {r.index(2)} {r.index(5)} {r.index(21)}")    # 3 0 1 2 0
```

**std.i18n.6** — `i18n.parse_forms(header)?` reads a `Plural-Forms` header, `nplurals=3; plural=EXPR;`. The
expression is C-style over `n`: numbers, `+ - * / % == != < > <= >= && || !`, parentheses and `a ? b : c`, with
the usual precedence; booleans are 1 and 0 and a division by zero is 0. A header with no `nplurals` or `plural`,
an expression that does not parse, or one that gives an index outside `0` to `nplurals - 1` for `n` of 0 or 1,
fails with `I18nError`. *(ADR 1273)*

```ail rules=std.i18n.6
fn main() !
    r := i18n.parse_forms("nplurals=2; plural=(n != 1);")?
    print("{r.nplurals} {r.index(1)} {r.index(3)}")    # 2 0 1
```

**std.i18n.7** — `i18n.from_po(text)?` reads a gettext `.po` text: `msgctxt`, `msgid`, `msgid_plural`, `msgstr`,
`msgstr[N]`, string continuation lines, and `#` comments. `\n`, `\t`, `\r`, `\"` and `\\` are resolved. An entry
marked `#, fuzzy`, one whose strings are all empty, and the header are not messages. The locale comes from the
header's `Language:` line and the plural rule from `Plural-Forms:` (the locale's table when absent). A bad line
fails with `I18nError` and its line number. *(ADR 1273)*

```ail rules=std.i18n.7
fn main() !
    po := r"""
msgid ""
msgstr ""
"Language: ru\n"

msgid "Save"
msgstr "Сохранить"

#, fuzzy
msgid "Skip"
msgstr "Пропустить"
"""
    c := i18n.from_po(po)?
    saved := c.tr("Save")
    print("{c.locale} {c.len()} {saved}")    # ru 1 Сохранить
```

**std.i18n.8** — `i18n.from_mo(data)?` reads the bytes of a compiled `.mo` file, in either byte order. The
original of a message is `msgid`, optionally preceded by its context and byte 4, and optionally followed by byte 0
and the plural original; the translation holds the forms split at byte 0. The header entry gives the locale and
rule as in `std.i18n.7`. A file that is too short, has a wrong magic number, has a table or string outside the
file, or has a string that is not UTF-8 fails with `I18nError` with line 0. The hash table is not used. *(ADR 1273)*

```ail rules=std.i18n.8
fn main()
    if c := i18n.from_mo(b"not a catalog")
        print(c.len())
    else err
        print(err)    # a .mo file starts with a 28-byte header (line 0)
```

**std.i18n.9** — `i18n.chain(locale)` is a locale and its parents, most specific first: `zh-Hant-TW`, `zh-Hant`, `zh`;
`_` works as `-`, a `.UTF-8` or `@euro` suffix is dropped, and an empty locale has no chain.
`i18n.pick(cats, wanted)` makes a `Bundle` of the catalogs whose locale is in the chain of a wanted locale, in
that order, each used once, tags compared without regard to case: `pick(cats, ["fr-CA", "en"])` tries `fr-CA`, then
`fr`, then `en`. `i18n.bundle(cats)` keeps the order given. `b.tr`, `b.trc`, `b.trn` and `b.trnc` work as on a
`Catalog`, and the first catalog that has the message answers, with its own plural rule; the English text
answers when none has it. *(ADR 1273)*

```ail rules=std.i18n.9
fn main()
    mut fr := i18n.catalog("fr")
    fr.add("", "Cancel", ["Annuler"])
    mut en := i18n.catalog("en")
    en.add("", "Save", ["Save"])
    b := i18n.pick([en, fr], ["fr-CA", "en"])
    print(b.cats.len())               # 2
    print(b.tr("Cancel"))             # Annuler
    print(i18n.chain("zh_Hant_TW").join(" "))    # zh-Hant-TW zh-Hant zh
```

**std.i18n.10** — `i18n.from_json(text)?` reads a catalog from a JSON object with these optional members:
`"locale"`, `"plural"` (a `Plural-Forms` header), `"messages"` (an object from `msgid` to a text, or to a list
with one text per plural form) and `"contexts"` (an object from a context to such a messages object). Anything
else is ignored. *(ADR 1273)*

```ail rules=std.i18n.10
fn main() !
    c := i18n.from_json(r"""{"locale":"fr","messages":{"file":["fichier","fichiers"]}}""")?
    print(c.trn("file", "files", 5))    # fichiers
```

**std.i18n.11** — `i18n.negotiate(accept)` is the locales of an `Accept-Language` header, highest `q` first (equal
weights keep header order); `*` and weight 0 are dropped. Pass the result to `pick`. *(ADR 1273)*

```ail rules=std.i18n.11
fn main()
    print(i18n.negotiate("fr-CH, fr;q=0.9, en;q=0.8, *;q=0.5").join(" "))    # fr-CH fr en
```

**std.i18n.12** — `error struct I18nError` has `why str` and `line int` (1-based for `.po`, 0 for `.mo` and JSON), and
its text is `WHY (line N)`. *(ADR 1273)*

```ail rules=std.i18n.12
fn main()
    if c := i18n.from_po("msgid foo")
        print(c.len())
    else err
        print(err)    # expected a string in double quotes (line 1)
```

**std.i18n.13** — `i18n.format(msg, vars)` replaces each `{name}` in `msg` by `vars["name"]`; a name is letters,
digits, `_`, `-` and `.`. `{{` and `}}` are one literal brace. A name that `vars` does not have stays as
written, so a missing value shows in the output and never stops the program. Write messages in raw strings
(`r"..."`) so their braces are not interpolation (`lex.str.2`). *(ADR 1273)*

```ail rules=std.i18n.13
fn main()
    print(i18n.format(r"Hi {name}, {n} new {{x}}", {"name": "Ann", "n": "3"}))    # Hi Ann, 3 new {x}
```

**std.i18n.14** — Out of scope, and what to do instead: writing `.po` or `.mo` files (use gettext's tools),
ICU message syntax (`{n, plural, ...}`), number, date and currency formatting by locale, and reading the
user's locale from the environment (`env.get("LANG")`, then `chain`). A program that needs more builds the text
in AIL. *(ADR 1273)*

```ail rules=std.i18n.14
fn main()
    print(i18n.chain("en_US.UTF-8").join(" "))    # en-US en
```
