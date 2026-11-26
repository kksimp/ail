# TBD. `std.url`: parse, write and resolve URLs, and the percent and query codecs

- **Status:** Accepted
- **Date:** 2026-11-26
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 17 — Standard library
- **Spec rules:** [std.url.1-11](../spec/std-url.md)

## Context

The std gap survey (2026-11-26) found URL work hand-rolled across the corpus (`url_parser`,
`query_string_parser`), in the agent training tasks (`percent_encode`, `url_parts`, `url_resolve`,
`query_string`) and in garlic (219 lines). Garlic's code is AGPL-3.0 and std is Apache 2.0 with the runtime
exception (ADR 0510), so this module is written fresh from RFC 3986.

## Decision

- A module written in AIL, `std/url/url.ail`, imported as `std.url` and in scope as `url` (std.use.4).
- `parse(s) Url ! UrlError` gives a struct of plain fields: `scheme`, `user`, `host`, `port int?`, `path`,
  `query`, `fragment`. An absent part is `""`. `scheme` and `host` are lower case; the rest stays
  percent-encoded as written. `impl ToStr for Url` writes it back, so `print(u)` shows the URL.
- `resolve(base, r) str ! UrlError` and `u.resolve(r Url) Url` follow RFC 3986 section 5.2 strictly; the test is
  every example of section 5.4.
- `join_path(base, parts)` adds path segments; `encode`/`decode` are the component codec (unreserved kept,
  upper-case `%XX`); `encode_form`/`decode_form` add `+` for a space; `parse_query` gives ordered pairs,
  `query_map` keeps a repeated key's first value, `build_query` writes pairs.
- `UrlError` carries `kind UrlKind` (`BadChar`, `BadEscape`, `BadScheme`, `BadHost`, `BadPort`), a `why` that
  says the fix, the text and the character offset (type.enum.7).
- Out of scope: IDNA, WHATWG browser rules, default ports. Switching `net.Options.query` and `std.net.jar` to
  this module is a follow-up.

## Rationale

The name. `url` is the word every agent tries first (Go's `url.Parse`, Python's `urllib.parse`). It clashes
with locals: 25 files in this repository bind a local, parameter or loop variable `url` (7 in
`compiler/rt/native`, 7 in `tests/cases`, 4 in `examples`), counted with
`grep -E "^\s*(mut )?url :=|\(url str|, url str|for url in"`; none of them calls a `url.` member that the
module exports, so under std.use.4 none loads it or changes meaning. A program that binds `url` and also
wants the module, `url := args[1]; u := url.parse(url)`, gets a type error on `str`; the card, the
quick reference and std.url.1 say to name the text `link`. `std.net.url` would still be used as `url` (the
last name), so it buys nothing against the clash and is a longer import; `uri` or `urls` would dodge the
clash but would be the second guess of every agent. AI understanding (ADR 1077, first) favours the first
guess.

Plain `str` fields over `str?` for `query` and `fragment`: `u.query.split("&")` and `url.query_map(u.query)`
work without an `else`, at the cost of not telling `a?` from `a` (std.url.11), which only exact
recomposition needs. The parts stay encoded so `to_str(parse(s))` gives `s` back apart from case, and so
resolution works on the text RFC 3986 defines it on.

Pure AIL, not built-ins: the parser touches no OS and needs no checker, effects or lowering changes, and it
keeps the lane inside its own files.

## Rejected alternatives

- **`std.net.url`:** the same `url` name in code, a longer `use`, and URLs are not only for `net`.
- **`uri` or another name:** avoids hiding by a local `url`, but agents and people guess `url` first.
- **Decoded fields (Go's `Path`):** loses `%2F` against `/` and needs a second raw field; Python's
  `urlparse` keeps them encoded too.
- **`query str?` and `fragment str?`:** RFC-exact recomposition for `a?` and `a#`, but every read needs an
  `else`.
- **WHATWG parsing:** browser-compatible, but its special-scheme rules, IDNA and default-port dropping are a
  much larger surface; RFC 3986 is what HTTP clients and servers exchange.
- **Last value wins in `query_map`:** PHP's rule; first wins matches Go's `Values.Get` and makes a later
  duplicate unable to override an earlier key.
- **Copying garlic's `url.ail`:** AGPL-3.0, incompatible with the std licence.

## Consequences

- Programs parse, build and resolve URLs and queries without hand-rolled code; `ail ask std` finds the
  functions by the words in `docs/design/std-synonyms.txt`.
- A local named `url` hides the module. The diagnostic for that is the generic one on `str`; giving AIL std
  modules the std.use.4 "the local hides the module" message (today built-in modules only, `check/call.ail`)
  is a follow-up checker change.
- Follow-ups: `net.Options.query` and `std.net.jar` may take a `Url`.
