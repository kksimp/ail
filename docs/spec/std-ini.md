# Standard library: `std.ini`

Chapter key: `std`

> **Status:** Drafted. Implemented in AIL (`std/ini/ini.ail`, ADR 1220).

INI files and `.env` files. `parse` reads INI text into the `Json` value
([std-json](std-json.md)), the shape `toml.parse` gives, so the accessors
`get`, `str` and `at` read every config format. INI has no types, so every
value is a `Str`. `write` is its inverse, and `parse_env` reads a `.env` file
into a map. Every function is pure (`eff.infer.1`).

## The module

**std.ini.1** — `use std.ini` provides `parse(text str) Json ! IniError`,
`write(j Json) str`, `parse_env(text str) Map[str, str] ! IniError` and the
error type `IniError`. *(ADR 1220)*

**std.ini.2** — `parse` gives a `Json.Obj`. A key before the first section is a
member of it with a `Str` value. A `[name]` line starts a section: a member
named `name` whose value is an `Obj` of the keys below it, up to the next
section. The name is kept as written, so `[a.b]` is the member `a.b`, not a
nested table. Members are in file order. Text with no keys is `{}`. *(ADR 1220)*

```ail rules=std.ini.1,std.ini.2
use std.ini

fn main() !
    doc := ini.parse("name = demo\n[server]\nhost = h\nport = 8080\n[a.b]\nk = v\n")?
    print(doc.to_text())   # {"name":"demo","server":{"host":"h","port":"8080"},"a.b":{"k":"v"}}
    server := doc.get("server") else Json.Null
    print(server.str("host") else "")
```

**std.ini.3** — A line is blank, a comment, a section header or `key = value`.
A comment line starts with `;` or `#`. The key and value are split at the
first `=` or `:`, and both are trimmed. A bare value ends at a `;` or `#` that
follows a blank, so `url = http://x/a#b` keeps its `#`. The value may be
empty. A section header may be followed by a comment. *(ADR 1220)*

```ail rules=std.ini.3
use std.ini

fn main() !
    doc := ini.parse("; note\n# note\na = 1 ; one\nb: two\nc =\nu = http://x/a#b\n[s] # sec\n")?
    print(doc.to_text())   # {"a":"1","b":"two","c":"","u":"http://x/a#b","s":{}}
```

**std.ini.4** — A value that starts with `"` is read to the closing `"` with the
escapes `\n`, `\t`, `\r`, `\"` and `\\`; any other backslash stays as written.
A value that starts with `'` is read literally to the closing `'`. Only a
comment may follow the closing quote, and a quote left open fails with
`IniError`. *(ADR 1220)*

```ail rules=std.ini.4
use std.ini

fn main() !
    doc := ini.parse("a = \"x ; \\\"y\\\"\"\nb = ' p\\n '\n")?
    print(doc.to_text())   # {"a":"x ; \"y\"","b":" p\\n "}
```

**std.ini.5** — `parse_env` reads a `.env` file into a `Map[str, str]` in file
order. It never reads the process environment (`std.env`). *(ADR 1220)*

**std.ini.6** — A `.env` line is blank, a `#` comment or `KEY=value`, with an
optional `export ` before the key. A key starts with a letter or `_` and holds
letters, digits, `_`, `.` and `-`. Blanks around `=` are dropped. A bare
value is trimmed and ends at a `#` that follows a blank (or starts the value).
A `"` value takes the escapes `\n`, `\r`, `\t`, `\"`, `\\` and `\$`; a `'`
value is literal. Both may span lines. Only a comment may follow the closing
quote. *(ADR 1220)*

**std.ini.7** — `${NAME}` in a bare or `"` value is replaced by the value of
`NAME` among the keys read earlier in the same file, or by the empty text when
there is none. `${NAME:-text}` gives `text` when `NAME` is missing or empty.
`$NAME` without braces and anything in a `'` value stay as written. An open
`${` fails with `IniError`. *(ADR 1220)*

```ail rules=std.ini.5,std.ini.6,std.ini.7
use std.ini

fn main() !
    src := "# c\nHOST=db\nexport PORT = 5432   # port\nURL=pg://$\{HOST}:$\{PORT}/$\{NAME:-app}\nRAW='$\{HOST}'\nMSG=\"a\nb\"\n"
    m := ini.parse_env(src)?
    for k, v in m
        shown := v.replace("\n", "|")
        print("{k}={shown}")
```

**std.ini.8** — `write(j)` gives INI text for an `Obj`: the members that are not
objects as `key = value` lines, then each object member as a `[name]` line and
its members. A blank line goes before each section header. The text ends in a
line break. Anything but an `Obj`, and an empty `Obj`, gives `""`. *(ADR 1220)*

**std.ini.9** — `write` prints a `Str` as is, in `"..."` with the escapes of
`std.ini.4` when reading it back bare would change it (it has outer blanks, or
holds `"`, `\`, `;`, `#`, a line break or a tab, or starts with `'`). `Null` is
an empty value. A `Bool`, `Num`, `Arr` or `Obj` inside a section, or an `Arr`
at the top, is written as its JSON text, which reads back as a `Str`. Keys are
written as given, so a key with `=`, `:` or a line break does not read back.
*(ADR 1220)*

```ail rules=std.ini.8,std.ini.9
use std.ini

fn main() !
    doc := Json.Obj([
        JsonMember("name", Json.Str("x y")),
        JsonMember("s", Json.Obj([
            JsonMember("k", Json.Str("a;b")),
            JsonMember("n", Json.Num(3.0)),
        ])),
    ])
    text := ini.write(doc)
    print(text.trim_end())
    print(ini.parse(text)?.to_text())   # {"name":"x y","s":{"k":"a;b","n":"3"}}
```

**std.ini.10** — A key set twice in one section, a key set twice before the first
section (in `parse_env`, anywhere), and a section opened twice, or on the name
of a top-level key, fail with `IniError`. No later value silently replaces an
earlier one. *(ADR 1220)*

**std.ini.11** — A leading byte order mark is ignored, and `\r\n` is read as
`\n`. A tab or space around a key, a value or a header is dropped. *(ADR 1220)*

**std.ini.12** — `IniError` is an error with `why str` and `line int` (1-based,
the line where the text is wrong); its text is `WHY (line N)`. The causes are
an unclosed or empty section header, text after a header or a closing quote, a
line with no `=` or `:`, an empty key, an unterminated quote or `${`, and a
duplicate (`std.ini.10`). *(ADR 1220)*

```ail rules=std.ini.10,std.ini.11,std.ini.12
use std.ini

fn why(text str) str
    _ := ini.parse(text) else err
        return "{err}"
    "ok"

fn why_env(text str) str
    _ := ini.parse_env(text) else err
        return "{err}"
    "ok"

fn main() !
    print(why("a = 1\na = 2\n"))        # duplicate key 'a' (line 2)
    print(why_env("A=\"open\n"))         # unterminated quoted value (line 1)
    doc := ini.parse("\u{feff}a = 1\r\nb = 2\r\n")?
    print(doc.to_text())                 # {"a":"1","b":"2"}
```

## Rejected alternatives

- Typing values (`8080` as a number, `true` as a bool): INI has no types, and a
  guess reads `007` and `yes` wrongly. `j.str("port")` then `to_int()` states
  what the program expects.
- Dotted section names as nested tables (`[a.b]`): files in the wild use the
  dot as part of the name (`[remote "origin"]` style names, php.ini). A program
  that wants nesting splits the name.
- Last value wins for a repeated key: hides a mistake in a file that looks fine
  (ADR 1077 order, AI understanding first). Fails with the line instead.
- Continuation lines and `${VAR}` from the process environment: an INI value that
  continues on the next line is not portable, and reading `std.env` would make
  `parse_env` impure; a program passes `env.get` results in itself.
