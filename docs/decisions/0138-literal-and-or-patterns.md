# 0138. Literal patterns, or-patterns, and one-line statement arms

- **Status:** Accepted
- **Date:** 2026-10-05
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 2 — Core syntax (expressions); 4 — Type system
- **Spec rules:** [expr.match.4](../spec/05-expressions.md) (amended), [expr.match.5](../spec/05-expressions.md) (amended), [expr.match.7–9](../spec/05-expressions.md), [expr.match.1](../spec/05-expressions.md) (amended by `expr.match.9`), [lex.num.3](../spec/02-lexical.md) (the `-` rule)
- **Amends:** [ADR 0054](0054-match-arm-syntax.md) (resolves its "still to design" list for literals and or-patterns; `expr.match.1` gains one-line arms), [ADR 0117](0117-typing-rules.md) (decision 6, "until general patterns are designed"), [ADR 0123](0123-variants-with-data.md) (patterns stay positional)
- **Evidence:** `examples/interp.ail`, `examples/json.ail`, `examples/sheet.ail`; `tools/tokbench` (mean of five tokenizers)

## Context

`expr.match.4` allows only `_` and variants "until general patterns are
designed", and ADR 0054 lists literals and or-patterns among the pieces still
to design. The three example programs (an interpreter, a JSON tool and a
spreadsheet engine) needed:
- string literals (the interpreter's operator `match`, the spreadsheet's
  function names) and char literals (the JSON parser's `'{' =>`);
- or-patterns (`Ident | Keyword | Op => ...`), because a catch-all `_` over
  named variants triggers the `expr.match.3` warning, so many arms were
  written out only to avoid it;
- one-statement arms on the arm's line. Two drafters wrote `=> return`,
  `=> continue` and `=> total += n` unprompted and hit `gram.syntax.1` and
  `expr.match.1`. The drafts have 21 two-line arms (JSON 4, sheet 16,
  interpreter 1).

ADR 0054 only requires a block for a result that "needs several statements",
and its consequences list the pattern forms as open, so this resolves its open
list and doesn't contradict the designer's decision.

## Options considered

| Option | Tokens |
|---|---|
| The 21 arms on two lines | 234.0 |
| **One line (chosen)** | **207.4** (−11%, 1.3 per arm) |
| Sheet, whole file: two-line / one-line | 6019.4 / 5998.8 (−0.34%) |
| JSON, whole file | 4274.4 / 4269.6 (−0.11%) |
| Interpreter, whole file | 6845.2 / 6844.0 (−0.02%) |
| Sheet, one-line plus or-patterns | 5984.0 (a further −14.8, −0.25%) |

Literal and or-patterns weren't measured against `if` chains (a different
construct); the alternatives are listed under Rejected alternatives.

## Deciding criterion

Token usage **ties** everywhere: −0.34% at most on a whole file. **AI
understanding** decides: agents write `'{' =>`, `"+" =>`, `A | B =>` and
`=> return` without being told, so rejecting them costs a failed first attempt
and a workaround that spends a line (or a throwaway binding) per arm.
Or-patterns also remove the arms that existed only to dodge `expr.match.3`.

## Decision

1. **expr.match.4 (amended).** A pattern is `_`, a variant pattern
   (`expr.match.6`), a literal (`expr.match.7`) or an or-pattern
   (`expr.match.8`).
2. **expr.match.7.** Literal patterns are int literals (with an optional `-`),
   char, str (no interpolation; raw strings are fine) and bool literals.
   - The literal has the scrutinee's type, with no conversion; the diagnostic
     for a char against a str (or the reverse) offers the other quote.
   - A float literal pattern is an error: "compare with `==` or `<` in an
     `if`" (float equality is IEEE, type.num.6).
   - int, str and char matches need `_` even when every value is listed; bool
     is covered by `true` and `false`.
   - `expr.match.5` compares literals by value: `0x41` after `65` is
     redundant, and so is `'\u{41}'` after `'A'`.
   - A `-` directly before an integer literal is part of the literal for
     `lex.num.3`, so `-9223372036854775808` is valid in patterns, expressions
     and constants.
   - Fields of a variant pattern stay a name or `_` (`expr.match.6`).
3. **expr.match.8.** `p1 | p2 | ...` matches if any alternative does. No
   alternative may bind a name; a variant pattern whose fields are all `_` is
   fine (`Number(_) | Label(_)`). `expr.match.5` applies to the alternatives:
   `'a' | 'a'`, or an alternative after `_` (`_ | 'a'`), is an error.
4. **expr.match.9 (amends `expr.match.1`).** An arm's result may be a one-line
   statement on the arm's line: `=> return x`, `=> break`, `=> continue`,
   `=> total += n`, `=> x = y`, `=> print(x)`. It means the same as a block
   holding that statement (`expr.block.3`).
5. **Not designed.** Range patterns (`'0'..='9'`) and guards (`x if x > 0`)
   are errors whose help suggests an `if` chain inside a `_` arm.

```ail excerpt
for c in text.chars()
    match c
        '(' | '[' => depth += 1
        ')' | ']' => depth -= 1
        '\n' => line += 1
        _ => continue

match content
    Formula(_, expr) => collect_refs(expr, mut refs)
    Number(_) | Label(_) | Broken(_, _) => return []
```

## Consequences

- A match on `Ident | Keyword | Op` names the variants, so `expr.match.3`
  doesn't warn, and a new variant still makes the match non-exhaustive.
- A statement-bodied arm still uses an indented block when it has several
  statements (ADR 0054).
- Range patterns, guards, binding or-patterns and literals nested inside
  variant fields remain open; each has a fixed help for now.
- The interpreter's `match op` and the spreadsheet's `Func.parse` match on
  string literals directly.

## Rejected alternatives

- **Float literal patterns:** IEEE equality (NaN never matches, `-0.0` equals
  `0.0`) makes "matches by value" misleading, and `expr.match.5` can't
  compare them reliably.
- **Binding or-patterns** (`Num(x) | Float(x)`): need the same bindings with
  the same types in every alternative; left for when binding patterns grow.
- **Keeping arms on two lines:** a tie on tokens, but two agents wrote the
  one-line form unprompted, and it needed a throwaway binding or a duplicate
  arm to work around.
- **An `if` chain instead of literal patterns:** a longer and unchecked form
  of the same dispatch, with no exhaustiveness check.
- **Exhaustiveness from listing every int, str or char value:** impractical
  for these types, so `_` is always required.
