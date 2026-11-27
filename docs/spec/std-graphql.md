# Standard library: `std.graphql`

Chapter key: `std`

> **Status:** Drafted. Implemented in AIL (`std/graphql/graphql.ail`, ADR 1277).

GraphQL executable documents and a small client. `parse` reads a document
(operations, fragments, variables, directives) into a tree of plain enums and
structs, `write` prints a tree as text, and a `Client` sends a query as a JSON
`POST` over `std.net` and decodes the reply. Parsing and writing are pure
(`eff.infer.1`); the client uses the `net` effect. Type system definitions
(schemas) are not read.

## The module

**std.graphql.1** — `use std.graphql` provides `parse(text str) Document !
ParseError`, `write(doc Document, indent int) str`, `quote(s str) str`,
`client(endpoint str) Client`, `request_json(query str, vars Json, operation
str) Json` and `decode_reply(body str, status int) Reply ! ClientError`, the
types below, and the methods `Document.operation`, `Document.fragment`,
`Document.problems`, `Value.to_json`, `OpKind.word`, `Client.header`,
`Client.execute`, `Client.query`, `Client.run`, `Reply.ok`, `Reply.need` and
`Reply.get`. *(ADR 1277)*

**std.graphql.2** — The tree. `Document { ops List[Operation], frags
List[Fragment] }`, each in source order. `Operation { kind OpKind, name str,
vars List[VarDef], dirs List[Directive], sel List[Selection] }` with `OpKind`
`Query`, `Mutation` or `Subscription`; `name` is `""` for an anonymous
operation, and the shorthand `{ ... }` is an anonymous `Query`. `Fragment {
name, on_type, dirs, sel }`. `enum Selection` is `Field(alias, name, args,
dirs, sub)` (`alias` is `""` and `sub` empty when not written), `Spread(name,
dirs)` and `Inline(on_type, dirs, sub)` (`on_type` is `""` for `... @dirs {
sub }`). `Arg { name str, value Value }` and `Directive { name, args }`.
`enum Value` is `Var(name)`, `Int(n int)`, `Float(x float)`, `Str(s str)`,
`Bool(b)`, `Null`, `Enum(name)`, `Arr(items)` and `Obj(fields List[Arg])`. A
`TypeRef { name, non_null, items }` is a named type with no `items`, or a list
type with an empty `name` and its element type as the one item; `non_null` is
the trailing `!`. `VarDef { name, ty, default_value Value?, dirs }`. All
fields are `pub`. `Value.to_json()` gives the value as `Json`: numbers and
booleans as themselves, strings and enum values as strings, and a variable as
`null`. *(ADR 1277)*

```ail rules=std.graphql.1,std.graphql.2
use std.graphql

fn main() !
    doc := graphql.parse("query Q($id: ID!) \{ user(id: $id) \{ name } }")?
    op := doc.operation("Q") else return
    print("{op.kind.word()} {op.name} {op.vars.len()} {op.vars[0].ty.name} {op.vars[0].ty.non_null}")
    print(graphql.write(doc, 0))   # query Q($id: ID!) { user(id: $id) { name } }
```

**std.graphql.3** — Lexical rules. Whitespace, commas, `#` comments up to the
end of the line and a leading byte order mark separate tokens and are
dropped. A name is `[A-Za-z_][A-Za-z0-9_]*`. A failure is `error struct
ParseError { kind ParseKind, why str, line int, col int }` (1-based place) with
`kind` `Syntax`, `Literal`, `Depth` or `Unsupported`. *(ADR 1277)*

```ail rules=std.graphql.3
use std.graphql

fn main() !
    _ := graphql.parse("\{ a(x: 01) }") else err
        print("{err.kind.name()}: {err}")   # Literal: a number cannot start with 0 and more digits (line 1, column 10)
        return
```

**std.graphql.4** — Limits and what is refused. Selections, types and values
nest at most 100 deep; deeper fails with kind `Depth`. A document of only
whitespace is empty and fails with `Syntax`. A definition that starts with
`schema`, `scalar`, `type`, `interface`, `union`, `enum`, `input`, `directive`
or `extend` fails with `Unsupported`, and an empty selection set, argument
list or variable list fails with `Syntax`. A variable is refused in a
variable's default value. Nothing else is checked: names, types and arguments
are not validated against a schema. *(ADR 1277)*

```ail rules=std.graphql.4
use std.graphql

fn main() !
    _ := graphql.parse("type A \{ x }") else err
        print(err.kind.name())   # Unsupported
        return
```

**std.graphql.5** — Strings. `"..."` reads the escapes `\"`, `\\`, `\/`,
`\b`, `\f`, `\n`, `\r`, `\t`, `\uXXXX` and `\u{X...}` (a high and low
surrogate pair written as two `\uXXXX` makes one character); a line break or
an unknown escape inside it fails with `Literal`. `"""..."""` is a block
string: `\"""` is a literal `"""`, the indentation common to every line after
the first is removed, and blank lines at the start and end are dropped; no
other escape is read. *(ADR 1277)*

```ail rules=std.graphql.5
use std.graphql

fn main() !
    doc := graphql.parse("\{ f(s: \"\"\"\n    Hello\n      World\n  \"\"\", u: \"\\u00e9\") }")?
    print(graphql.write(doc, 0))   # { f(s: "Hello\n  World", u: "é") }
```

**std.graphql.6** — Numbers. `-?(0|[1-9][0-9]*)` is an `Int` (an `int`, so a
value outside 64 bits fails with `Literal`); with `.digits` or an exponent it
is a `Float`. A number cannot be followed by a name character or a `.`.
*(ADR 1277)*

```ail rules=std.graphql.6
use std.graphql

fn main() !
    doc := graphql.parse("query($a: Float = -2.5, $b: Int = 7) \{ x }")?
    print(doc.ops[0].vars.len())   # 2
```

**std.graphql.7** — Variables and directives. An operation's variables are
`($name: Type = default @dir, ...)`, and directives may follow the variables,
a field, a spread, an inline fragment, a fragment definition and each
variable. *(ADR 1277)*

```ail rules=std.graphql.7
use std.graphql

fn main() !
    doc := graphql.parse("query Q($ids: [ID!]! = [\"a\"]) @live \{ users(ids: $ids) @include(if: true) \{ id } }")?
    print(graphql.write(doc, 0))   # query Q($ids: [ID!]! = ["a"]) @live { users(ids: $ids) @include(if: true) { id } }
```

**std.graphql.8** — Writing. `write(doc, indent)` (`indent >= 0`) prints the
operations then the fragments. With `indent` 0 it is one line with a space
inside each `{ }`; above 0 each selection is on its own line, indented
`indent` spaces per level, and definitions are separated by a blank line. An
anonymous query without variables or directives is written as `{ ... }`;
anything else starts with its keyword. Writing a parsed document gives text
that parses to the same tree, except that comments and commas are gone, a
block string is written as a quoted one and a float is written as `float`
prints it. `quote(s)` is `s` as a quoted literal with `"`, `\`, line breaks,
tabs and other control characters escaped. *(ADR 1277)*

```ail rules=std.graphql.8
use std.graphql

fn main() !
    doc := graphql.parse("\{ user(id: 4) \{ name, friends \{ name } } }")?
    print(graphql.write(doc, 2))
    print(graphql.quote("a\"b\n"))   # "a\"b\n"
```

**std.graphql.9** — Lookup. `doc.operation(name)` is the operation called
`name`, or with `""` the only operation of a document that has exactly one;
otherwise `none`. `doc.fragment(name)` is the fragment called `name`.
*(ADR 1277)*

```ail rules=std.graphql.9
use std.graphql

fn main() !
    doc := graphql.parse("query A \{ x } fragment F on T \{ y }")?
    print(doc.operation("") != none)    # true
    print(doc.operation("B") == none)   # true
    print(doc.fragment("F") != none)    # true
```

**std.graphql.10** — `doc.problems()` finds what a server would refuse
without a schema: two operations with one name, an anonymous operation next to
other operations, two fragments with one name, and a spread of a fragment the
document does not define. It is empty when none is found. *(ADR 1277)*

```ail rules=std.graphql.10
use std.graphql

fn main() !
    doc := graphql.parse("query A \{ ...F } query A \{ x }")?
    for p in doc.problems()
        print(p)
    # the operation 'A' is defined twice
    # the fragment 'F' is not defined
```

## The client

**std.graphql.11** — `graphql.client(endpoint)` is a `Client { url, headers
Map[str, str], timeout_ms int }` with no headers and a 30000 ms limit; `c.header(name,
value)` (`mut self`) sets a header sent on every request. `c.execute(query,
vars, operation)` sends `POST` with the body `request_json(query, vars,
operation)`: `{"query": ..., "variables": ..., "operationName": ...}`, leaving
out `variables` when `vars` is `Json.Null` and `operationName` when it is `""`.
`Content-Type: application/json` and `Accept: application/json` are sent
unless `headers` has one of that name in any case. `c.query(query, vars)` is
`execute` with no operation name, and `c.run(doc, operation, vars)` writes
`doc` with `write(doc, 0)` first. Redirects and proxies follow `std.net`.
*(ADR 1277)*

```ail rules=std.graphql.11,std.graphql.12,std.graphql.13,std.graphql.14
use std.graphql
use std.net
use std.net.http

fn api(r net.Request) net.Response
    j := parse_json(r.body) else err
        return http.text(400, "bad")
    data := Json.Obj([JsonMember("echo", Json.Str(j.text("query", "")))])
    http.json(200, Json.Obj([JsonMember("data", data)]))

fn main() !
    srv := net.serve("127.0.0.1:0")?
    t := spawn http.run(srv, api)
    mut c := graphql.client("http://{srv.addr()}/graphql")
    c.header("Authorization", "Bearer token")
    r := c.query("\{ hello }", Json.Null)?
    print("{r.status} {r.ok()}")                      # 200 true
    echo := r.get("echo") else Json.Null
    print(echo)                                      # "{ hello }"
    print(r.need()?.to_text())                       # {"echo":"{ hello }"}
    srv.close()
    t.wait()?
```

**std.graphql.12** — A reply. `decode_reply(body, status)` reads the JSON
object of a GraphQL response into `Reply { status int, data Json, issues
List[Issue], extensions Json }`: `data` is `Json.Null` when the response has
none or `null`, and `extensions` is the response's own. Each entry of `errors`
is an `Issue { message, path, line, col, code, extensions }`: `path` is the
response path joined with dots (`user.friends.0.name`, `""` when none), `line`
and `col` come from the first of `locations` (0 when absent), `code` is
`extensions.code` (`""` when absent) and `extensions` is that value whole. A
response with `errors` is still a `Reply`: a GraphQL server answers with
errors and often partial data, and the program decides. A body that is a
GraphQL response with a non-2xx status (a server that answers `400` with
`errors`) is a `Reply` too. *(ADR 1277)*

```ail rules=std.graphql.12
use std.graphql

fn main() !
    r := graphql.decode_reply("\{\"data\":null,\"errors\":[\{\"message\":\"no\",\"path\":[\"a\",0],\"locations\":[\{\"line\":2,\"column\":5}]}]}", 200)?
    print("{r.ok()} {r.issues[0].message} {r.issues[0].path} {r.issues[0].line}:{r.issues[0].col}")   # false no a.0 2:5
```

**std.graphql.13** — Reading a reply. `r.ok()` is true when there are no
`issues`. `r.need()` is `data`, or fails with `ClientError` of kind `Server`
whose `message` is the first issue's message and whose `issues` are all of them.
`r.get(path)` follows a slash path into `data`: a member name for an object,
a number for an array index; `none` when the path leaves the data.
*(ADR 1277)*

```ail rules=std.graphql.13
use std.graphql

fn main() !
    r := graphql.decode_reply("\{\"data\":\{\"u\":\{\"tags\":[\"a\",\"b\"]}}}", 200)?
    t := r.get("u/tags/1") else Json.Null
    print(t)                       # "b"
    print(r.get("u/none") == none) # true
```

**std.graphql.14** — Failures. `error struct ClientError { kind GqlKind,
message str, status int, issues List[Issue] }` with `GqlKind` `Transport` (the
request could not be sent or no reply came: `status` 0 and `message` the
`NetError` text), `Status` (a non-2xx status whose body is not a GraphQL
response), `Decode` (a 2xx body that is not a JSON object) and `Server` (from
`need`). *(ADR 1277)*

```ail rules=std.graphql.14
use std.graphql

fn main() !
    _ := graphql.decode_reply("oops", 502) else err
        print("{err.kind.name()} {err.status}")   # Status 502
        return
```

## Rejected alternatives

- Reading type system definitions (SDL), and validating a document against a
  schema: a different, larger module with its own tree; a client program
  sends a query and reads JSON, and the server validates.
- An `error` that fails every reply with `errors`: GraphQL replies carry
  partial `data` with `errors`, and a client that throws hides the data;
  `need()` is the one call that fails.
- Keeping a number's source text in `Value`: `Int` and `Float` are what a
  program computes with. A literal outside 64 bits fails instead of rounding.
- A schema-typed client (generated types): needs code generation and a schema
  reader; `Json` with `get` and `need` serves until then.
- Subscriptions and `multipart` uploads over WebSocket or SSE: the client is
  one request and one reply. `std.net.websocket` carries a subscription for a
  program that needs one.
- Persisted queries, batching and a cache: protocol extensions of particular
  servers, not GraphQL.
