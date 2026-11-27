# Standard library: `std.template`

Chapter key: `std`

> **Status:** Drafted. Written in AIL (`std/template/template.ail`, ADR 1243).

Text templates: a template is text with tags between double braces, and `render` fills it from a `Json` value.
It makes web pages, emails, reports and config files from data. Every function is pure (`eff.infer.1`).
The language is small on purpose: names and paths, `#each`, `#if`, `#unless`, comments. There are no partials,
helpers or lambdas (`std.template.12`).

## The module

**std.template.1** — `template` is in scope without `use` (`std.use.4`); `use std.template` stays valid. A local
named `template` hides it. `render(tpl str, data Json) str ! TemplateError` replaces every tag of `tpl` by the
value it names in `data`. A `{{name}}` is HTML escaped (`std.template.4`); text outside tags is copied as it
is. `data` is any `Json`, and `v.to_json()` makes one from a struct with `impl ToJson` (`std.template.11`).
*(ADR 1243)*

```ail rules=std.template.1
fn main() !
    data := parse_json(r"""{"name":"R&D <team>"}""")?
    print(template.render(r"<p>Hello, {{name}}!</p>", data)?)    # <p>Hello, R&amp;D &lt;team&gt;!</p>
```

**std.template.2** — `compile(src str) Template ! TemplateError` reads a template once, and `t.render(data)` and
`t.render_text(data)` fill it as often as needed. `render(tpl, data)` is `compile(tpl)?.render(data)?`. A
`Template` is a plain value that holds no data. All syntax errors (`std.template.10`) are found by `compile`;
`render` only fails on the data. *(ADR 1243)*

```ail rules=std.template.2
fn main() !
    t := template.compile(r"{{n}}!")?
    print(t.render(parse_json(r"""{"n":1}""")?)?)    # 1!
    print(t.render(parse_json(r"""{"n":"a"}""")?)?)  # a!
```

**std.template.3** — A name is a path of letters, digits, `_` and `-` joined by dots: `{{user.name}}`. The first
part is looked up in the data of the innermost `#each` first, then in each level outward to the top, so inside a
loop a row's fields and the top-level fields are both visible. A later part is an object member, or a number
that indexes a list (`{{user.tags.1}}`). `this` is the value of the innermost level, and `this.name` starts
there and does not look outward. A string is written as it is, `null` as nothing, and any other value as its
JSON text (`3`, `true`, `[1,"x"]`). *(ADR 1243)*

```ail rules=std.template.3
fn main() !
    data := parse_json(r"""{"user":{"name":"Cy","tags":["tea","jazz"]}}""")?
    print(template.render(r"{{user.name}} likes {{user.tags.1}}", data)?)    # Cy likes jazz
```

**std.template.4** — `{{name}}` writes `&`, `<`, `>`, `"` and `'` as `&amp;`, `&lt;`, `&gt;`, `&quot;` and `&#39;`.
`{{{name}}}` and `{{&name}}` write the value as it is. `render_text` (and `Template.render_text`) writes every
tag as it is, for text that is not HTML. `escape_html(s str) str` is the escaping alone. *(ADR 1243)*

```ail rules=std.template.4
fn main() !
    data := parse_json(r"""{"name":"R&D <team>"}""")?
    print(template.render(r"{{{name}}} {{&name}}", data)?)    # R&D <team> R&D <team>
    print(template.render_text(r"{{name}}", data)?)           # R&D <team>
    print(template.escape_html("a < b"))                     # a &lt; b
```

**std.template.5** — A name that has no value in the data fails with `TemplateError` of kind `Missing` and the
message `no value for "NAME"`, so a misspelled name is found at once and never renders as an empty hole. A
member that is `null` has a value: it writes nothing. `#if` and `#unless` are the exception: a name with no
value is false there, so they test whether data is there. *(ADR 1243)*

```ail rules=std.template.5
fn main() !
    data := parse_json(r"""{"name":"x"}""")?
    if out := template.render(r"{{nme}}", data)
        print(out)
    else err
        print("{err.kind.name()}: {err}")    # Missing: no value for "nme" (line 1)
```

**std.template.6** — A block tag (`#each`, `#if`, `#unless`, `else`, the closing tags) or a comment that is alone
on its line leaves no line behind: the spaces before it and the spaces and line break after it are dropped. A
tag with other text on its line, and a `{{name}}` tag, never change the text around them. *(ADR 1243)*

```ail rules=std.template.6
fn main() !
    tpl := r"""
        <ul>
          {{#each xs}}
          <li>{{this}}</li>
          {{/each}}
        </ul>
        """
    print(template.render(tpl, parse_json(r"""{"xs":["a","b"]}""")?)?)
    # <ul>
    #   <li>a</li>
    #   <li>b</li>
    # </ul>
```

**std.template.7** — `{{#if name}}A{{else}}B{{/if}}` writes `A` when `name` is true and `B` otherwise; the `else`
part is optional. `#unless` is the opposite. A value is false when it is `null`, `false`, `""`, `[]` or `{}`,
or when the name has no value (`std.template.5`); every other value is true, including the number 0. *(ADR 1243)*

```ail rules=std.template.7
fn main() !
    data := parse_json(r"""{"admin":false,"n":0}""")?
    print(template.render(r"{{#if admin}}root{{else}}guest{{/if}} {{#unless banned}}ok{{/unless}}", data)?)
    # guest ok
    print(template.render(r"{{#if n}}zero is true{{/if}}", data)?)    # zero is true
```

**std.template.8** — `{{#each name}}...{{/each}}` writes its body once for each element of a list, or each member of
an object, and nothing for `null`; any other value is a `WrongType` error. Inside the body `this` is the element,
and `@index` (from 0), `@count`, `@first`, `@last` and `@key` (the member name, for an object) describe the
position. An `{{else}}` in the body is written instead of it when there is nothing to loop over. Loops nest, and
`@index` and the other names belong to the innermost loop. *(ADR 1243)*

```ail rules=std.template.8
fn main() !
    data := parse_json(r"""{"xs":["a","b","c"],"m":{"x":1,"y":2},"e":[]}""")?
    print(template.render(r"{{#each xs}}{{@index}}:{{this}}{{#unless @last}}, {{/unless}}{{/each}}", data)?)
    # 0:a, 1:b, 2:c
    print(template.render(r"{{#each m}}{{@key}}={{this}} {{/each}}", data)?)    # x=1 y=2
    print(template.render(r"{{#each e}}{{this}}{{else}}nothing{{/each}}", data)?)    # nothing
```

**std.template.9** — `{{! text }}` is a comment and writes nothing. To write `{{` itself put a backslash before
it: `\{{` writes `{{`. *(ADR 1243)*

```ail rules=std.template.9
fn main() !
    print(template.render(r"a{{! hidden }}b \{{c}}", parse_json("{}")?)?)    # ab {{c}}
```

**std.template.10** — `error struct TemplateError` has `kind TemplateKind`, `why str` and `line int` (1-based, the
line of the tag), and its text is `WHY (line N)`. The kinds are `Unclosed` (`{{` with no `}}`), `BadTag` (an empty
tag, a name with other characters, an unknown block such as `#foo`, a block tag without exactly one name, an
unknown `@` name), `Unbalanced` (a block never closed, a closing tag that closes nothing or another block, an
`else` outside a block or a second `else` in one block), `Missing` (`std.template.5`) and `WrongType`
(`std.template.8`). `@index` and the other loop names outside a loop are `Missing`. *(ADR 1243)*

```ail rules=std.template.10
fn main() !
    for tpl in [r"a {{b", r"{{#if a}}x", r"{{/if}}", r"{{a b}}"]
        if out := template.render(tpl, parse_json("{}")?)
            print(out)
        else err
            print("{err.kind.name()}: {err}")
    # Unclosed: a tag is never closed (line 1)
    # Unbalanced: the block "if" is never closed (line 1)
    # Unbalanced: a closing tag has no block to close (line 1)
    # BadTag: "a b" is not a name: use letters, digits, _, - and dots (line 1)
```

**std.template.11** — A template with `{{` in an AIL string needs care, because `"{{x}}"` is a compile-time error
(`lex.str.2`): write it in a raw string, `r"""..."""` (`lex.str.6`), or read it from a file with `fs.read`
and pass the text to `compile`. Data comes from `parse_json`, or from `v.to_json()` of a struct that has
`impl ToJson for T` (`json.derive.1`); a field name is the path part. *(ADR 1243)*

```ail rules=std.template.11
struct Page
    title str
    items List[str]

impl ToJson for Page

fn main() !
    p := Page(title: "R&D", items: ["a", "b"])
    print(template.render(r"{{title}}: {{#each items}}[{{this}}]{{/each}}", p.to_json())?)    # R&amp;D: [a][b]
```

**std.template.12** — The language has no partials (`{{> name}}`), helpers, lambdas, `../` parent paths, changing
delimiters or whitespace control marks (`{{~`); a name is only looked up, never computed. A program that needs
more builds the text in AIL. *(ADR 1243)*

```ail rules=std.template.12
fn main() !
    data := parse_json(r"""{"a":1}""")?
    if out := template.render(r"{{> head}}", data)
        print(out)
    else err
        print(err.kind.name())    # BadTag
```
