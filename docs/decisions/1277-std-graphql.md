# 1277. `std.graphql`: GraphQL documents into a tree, a printer, and a client over `std.net`

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked for a fuller std on 2026-11-26
- **Topic:** 17 — Standard library
- **Spec rules:** [std.graphql.1-14](../spec/std-graphql.md)

## Context

Programs that call a GraphQL API (GitHub, Shopify, Linear, Hasura) build the query text by hand, post it, and dig
the answer out of `Json`. Programs that serve or proxy GraphQL, lint queries or generate them need the document as a
tree. `std.net` has the HTTP client and `Json` the codec; there was no GraphQL layer, so every program wrote its own
string building and its own `errors` handling.

## Decision

- `std.graphql` is an AIL module (`std/graphql/graphql.ail`): pure parser and printer, and a client that is a thin
  layer over `net.request`. No checker, effects or runtime work.
- `graphql.parse(text) Document ! ParseError` reads executable documents only: operations, fragments, variables,
  directives, all value kinds, strings with every escape and block strings. The tree is plain `pub` enums and
  structs (`Document`, `Operation`, `Selection`, `Value`, `TypeRef`, ...), so a program reads it with `match`.
- `graphql.write(doc, indent)` prints a tree, one line at indent 0, so a program can build or edit a document and send
  it. `graphql.quote(s)` writes a string literal.
- `doc.problems()` reports the schema-free mistakes a server refuses (duplicate names, an anonymous operation next to
  others, a spread of an undefined fragment).
- `graphql.client(endpoint)` with `c.header`, then `c.query(text, vars)`, `c.execute(text, vars, operation)` or
  `c.run(doc, operation, vars)`, sends `POST` with the JSON body `{query, variables, operationName}` and returns a
  `Reply` with `data`, `issues` (the `errors`) and `extensions`. `r.need()` is the one call that turns issues into an
  error (kind `Server`); `r.get("a/b/0")` reads `data` by path.
- A reply with `errors` is a `Reply`, not a failure: GraphQL returns partial data with errors. Only a failed exchange
  is a `ClientError` with a `GqlKind`: `Transport`, `Status` (non-2xx and not a GraphQL body) or `Decode`.
- The module name `graphql` clashes with no local name (it is a new word), so it is a safe implicit module under
  std.use.4.
- No syntax changes.

## Rejected alternatives

- **Reading SDL and validating against a schema.** A second tree and a validator the size of this module; the
  server validates, and a client needs none of it. A program that needs SDL can add it as a separate module.
- **Failing every reply that has `errors`.** Hides partial `data`, which GraphQL servers send on purpose. `need()`
  gives the strict form in one call.
- **Keeping number text in `Value`.** `Int` and `Float` are what programs compute with; a literal outside 64 bits
  is a `Literal` error, never a rounded value.
- **A query builder with a typed schema.** Needs code generation; `write` over the tree and `Json` variables cover
  building a document by hand.
- **Subscriptions, batching, persisted queries, multipart upload.** Server-specific protocol layers; a subscription
  is `std.net.websocket` carrying the same documents.
- **Runtime built-ins.** Checker, effects and lowering cost per function for text that AIL parses at a speed the
  network call dwarfs.
