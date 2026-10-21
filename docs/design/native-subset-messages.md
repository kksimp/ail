# Native subset messages: help lines and proposed wording (lane 229)

ADR 0722 adds a `help:` line to each `native.subset.1` problem. The help text
lives in `compiler/mir/subset_help.ail` and is keyed on the message text, so the
messages themselves are untouched here. The messages are written in
`compiler/mir/lower.ail` and its siblings, which this lane does not edit (the
queue there is long). This page lists the wording changes to apply when that
queue clears: site, current text, proposed text.

## Proposed changes

| Site | Current text | Proposed text | Why |
|---|---|---|---|
| `lower.ail:1199` (generic helper) | `not in the native subset yet: {what}` | `{what} isn't in the native subset yet` | Most messages (about 300 of 457 on the examples) use this prefix form; the rest say `X isn't in the native subset yet`. One form lets help rules match on one shape. |
| `lower.ail:725`, `lower_err.ail:93,96` | `expected \`bool\`, found \`str\`` (under `native.subset.1`) | a type error with its own rule (`expr.type.N`), not `native.subset.1` | A type error is not a subset gap; it should fail `ail check`, which would have caught it first. |
| `lower.ail:1135` | `` `{name}` isn't in the native subset yet (only locals, parameters and constants) `` | `` `{name}` isn't native: a module-level value must be a constant `` | The parenthesis lists what is allowed; the help line now says it. |
| `lower.ail:403` | `` `f`: its result type isn't in the native subset yet `` | keep; add the type name: `` `f` returns `T`, which isn't in the native subset yet `` | The type is what the reader needs. |
| `lower.ail` `this expression`, `this print`, `this list type`, `this assignment target isn't in the native subset yet`, `a tuple` | the bare phrase | name the form: `` printing a `T` ``, `` a list of `T` ``, `` assigning to a `kind` ``, `` an `if` as a value with no `else` `` | Left for the `lower.ail` queue. The position points at the construct, but the message alone (in a log or an agent's summary) does not name it. ADR 0785 audit. |

Done in ADR 0785's follow-up (the `lower_*.ail` files need no queue slot): `this call`,
`labelled arguments`, `this method call`, ``this `char.from_code` `` and ``this `{name}` type``
now name the callee and the count or the argument kind, in `lower_closure`, `lower_os`,
`lower_struct`, `lower_map`, `lower_tensor` and `lower_str`.

## Help-line coverage

ADR 0785: every `native.subset.1` line now has a `help:` line that says `ail run` runs the program. Before that, 382 of 457 `native.subset.1` lines on `examples/*.ail` carry a
`help:` line (see the measurement in ADR 0722). The rest are `expected bool, found
str` (a type error, see above) and forms with no in-subset alternative yet.

## Rejected alternatives

- **Editing `lower.ail` now.** The queue there is long, and a PR at the end of it
  would not land tonight. The help is keyed in the printer instead.
- **Putting the help in the Rust `ail explain` data.** The help must print where the
  problem prints, and the native backend's text lives in `compiler/`.
