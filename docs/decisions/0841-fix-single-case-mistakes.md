# ADR 0841: `ail fix` for four one-rewrite mistakes

- Status: Accepted
- Decided by: AI (delegated, ADR 0076)
- Date: 2026-10-21

## Context

Four mistakes from the corpus had one correct rewrite that the checker could
name, but carried no fix.

## Decision

- gram.fn.2, `fn(int) -> int` as a parameter type: delete the `->` (and the
  blank after it). The result type follows `)` directly.
- expr.call.1, `t.reshape(2, 3)`: wrap the arguments, `t.reshape([2, 3])`,
  when `Tensor.reshape` gets two or more positional arguments.
- expr.name.1, `e.message()` (also `msg`, `description`, `getMessage`,
  `get_message`) on an error type: rename to `to_str`. An argument, if any,
  is left for the next round to report.
- lex.bytes.1, `\u{e9}` in a `b"..."` literal: replace the escape with the
  UTF-8 bytes of the code point as `\xHH` escapes. The literal is still an
  error until fixed (lex.bytes.1 is unchanged), so no program changes meaning.

Not done, with the reason, from the same list of candidates:

- `regex.sub(...)` to `regex.replace(...)`: Python's `re.sub(pattern, repl,
  string)` and AIL's `replace(re, s, repl)` order their arguments differently,
  and both are `str`, so a rename could silently swap two values.
- `x if x > 0 =>` in a `match`: the guard's uses of `x` would need rewriting
  to the scrutinee, from a different diagnostic than the pattern's.
- A missing return type (mod.main.3): the diagnostic is at the call, the
  edit would be at the declaration, and removing the `print` is as valid a
  reading as adding the type.

## Rejected alternatives

- Guessing in the ambiguous three: a fix that compiles but means something
  else is worse than the help text alone.
