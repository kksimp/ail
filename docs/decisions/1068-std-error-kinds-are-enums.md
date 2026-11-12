# 1068. The kinds of the standard library's errors are enums

- **Status:** Accepted
- **Date:** 2026-11-12
- **Decided by:** Designer (Kaleb, 2026-11-12: every recommendation of the
  language review goes into 0.1.0, breaking changes accepted; "never
  compromise on AI understanding"). Details: AI (delegated, ADR 0076).
- **Topic:** Errors; standard library
- **Spec rules:** std.err.1, std.net.4, std.compress.2, std.ui.1, std.app.9,
  std.audio.9, std.regex.11-23, type.enum.7
- **Amends:** [ADR 0158](0158-standard-io-errors.md) (kind as a string)

## Context

`fs`, `proc`, `net`, `compress`, `ui`, `app`, `audio.device` and `regex`
errors carried `kind str`, told apart with `err.kind == "not_found"`. A typo
(`"notfound"`) compiled and was never true, which is the class of mistake the
rest of AIL catches at compile time. A `match` on the string needed a `_` arm,
so a new kind was never reported either. The language review of 2026-11-12
named this first among its recommendations.

## Options considered

### Option A — one fieldless enum per error type, uniform display (chosen)
`fs.IoKind`, `proc.ProcKind`, `net.NetKind`, `compress.CompressKind`,
`ui.UiKind`, `app.AppKind`, `device.DeviceKind`, `regex.RegexKind`. A kind is
compared with `err.kind == NotFound` (the expected type makes the bare variant
legal, type.enum.3) or matched exhaustively. It prints like any enum:
`fs.IoKind.NotFound`.
- **Pros:** typos are compile-time errors; `match` is checked for every kind
  (expr.match.2); the card lists the kinds next to the type; one display rule
  for every enum.
- **Cons:** breaking; `print(err.kind)` output changes. Tokens: ADR 0158
  measured the string and the variant spellings as a tie.

### Option B — one shared `Kind` enum for every module
- **Pros:** one type to learn.
- **Cons:** an exhaustive `match` on a file error would have to list `Tls`
  and `TooBig`, which a file call never gives; the type no longer says what
  can happen.

### Option C — enums that display as the old snake-case name
- **Pros:** no output change for programs that print a kind.
- **Cons:** a second display rule for some enums only; the text no longer
  names the value to match on.

### Option D — kinds as error enums (`IoError.NotFound(path)`)
- **Pros:** the error itself is matched.
- **Cons:** changes the error's shape and every field access (`err.path`);
  larger break for no gain over A with ADR 1078's member matching.

## Decision

Every `kind` field of a standard error is a fieldless enum named for its
error (`IoKind` for `IoError`, and so on). `proc.ProcKind` starts with the
variants of `fs.IoKind` in the same order and adds `TimedOut` and
`Cancelled`; `compress.CompressKind` adds its own five before the I/O ones.
A kind prints as any enum does. A string literal compared with or matched
against an enum is an error (type.enum.7) whose fix writes the variant when
the string is its snake-case name, so `ail fix` converts old code.

## Rationale

AI understanding first (ADR 1065): the compiler, not a test run, finds the
misspelt kind, and an exhaustive `match` shows every case. Per-error enums
keep a `match` to the kinds a call can give. Uniform display keeps one rule
for every enum and names the variant to write.

## Consequences

- The runtime builds `fs.IoError`, `proc.ProcError`, `net.NetError` and
  `compress.CompressError` (`compiler/rt/native/kinds.ail`). It stores the
  variant's index when `rt.kinds_as_enums()` is true. The lowering of this
  ADR replaces that function's body with `true`
  (`compiler/mir/lower_kinds.ail`); a compiler lowered by a seed from before
  this ADR keeps the text, which is what that seed's code reads. So the
  seed-built CLI works until the next seed, and nothing is left to remove
  after it beyond the flag.
- `ProcKind` shares `IoKind`'s indices so one record format (`ail run
  --record`) serves both.
- `app.Event.kind` and the `ui` keys and mouse buttons are still strings;
  they follow in a separate change.
- `DeviceError`'s display text is its `message` (it was `kind: message`).

## Rejected alternatives

- B: an exhaustive match would list kinds that can't happen.
- C: two display rules for enums; the printed text would not be code.
- D: a larger break than A for the same checking.
