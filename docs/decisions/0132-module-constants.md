# 0132. Module constants: `NAME := expr` with a constant expression

- **Status:** Accepted
- **Date:** 2026-10-05
- **Decided by:** Designer (the value of a module constant is a constant expression)
- **Details decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)): scope, `pub`, allowed forms, evaluation and error rules
- **Topic:** 12 — Modules; 2 — Core syntax
- **Spec rules:** [mod.const.1–3](../spec/12-modules.md)
- **Amended by:** [ADR 0225](0225-constants-call-compile-time-safe-functions.md) (a constant may call compile-time-safe functions)
- **Extends:** [ADR 0098](0098-pub-visibility.md) (it lists constants among the `pub`-able declarations but nothing declared one), [ADR 0097](0097-directory-modules.md) (mod.unit.1: a module-wide namespace)
- **Evidence:** `examples/interp.ail`, `examples/json.ail`, `examples/sheet.ail`

## Context

AIL has no module-level values. The three programs that motivated this batch
need them: the interpreter has seven (`MAX_DEPTH`, `KEYWORDS`, `TWO_CHAR_OPS`,
`ONE_CHAR_OPS`, `ESCAPES`, `BINARY_LEVELS` and a 16-entry `SCRIPTS` list) and
the spreadsheet has two (`COLS`, `MAX_ROW`). Without them each use becomes a
function that rebuilds the set or list on every call. There is no shared
mutable state to fall back on either: shared mutation only goes through
`ref T` (ADRs 0060–0063).

## Options considered

What the value may be (the designer chose the middle row):

| Option | Covers | Cost |
|---|---|---|
| Literals only | `MAX_DEPTH`, `COLS` | `KEYWORDS`, `ESCAPES` and the nested `BINARY_LEVELS` and `SCRIPTS` need sets, maps and lists |
| **Constant expressions (chosen)** | all nine | needs a rule for what is constant |
| Any expression, run at startup | everything | initialisation order and side effects (Go's `init` problems) |

Syntax: `NAME := expr` or `const NAME = expr`. The keyword costs one token per
constant, about 0.1% of `interp.ail` (mean of five tokenizers, tokbench).

## Deciding criterion

The designer decided the value rule. For the details, **AI understanding**
decides:
- a constant has no order and runs no user code, so reading a definition never
  needs "what ran first?" or "what does this impl do?";
- the syntax is the binding form AIL already has (`:=`), so it needs no new
  keyword. Tokens tie (0.1%, under the 2% threshold);
- the scope is the one top-level names already have (mod.unit.1), not a third.

## Decision

1. **Declaration (mod.const.1).** A top-level `NAME := expr` declares a module
   constant. It is visible in every file of the module (mod.unit.1), shares the
   namespace of functions and types, and is private unless marked `pub`
   (mod.vis.1). Importers write `m.NAME` (mod.use.2). Its type is the
   expression's type. Top-level `mut`, `const`, `let`, `var` and `static` are
   errors, with the help "write `NAME := value`" or "no mutable globals".
2. **Constant expressions (mod.const.2).** The expression may use:
   - literals (int, float, str, char, bool) and other constants (a cycle is an
     error that names it);
   - list, map and set literals, and `List[T]()`, `Map[K, V]()`, `Set[T]()`;
   - struct literals and variants with constant arguments;
   - `none`, `[]` and `{}` where a field or element gives the expected type;
   - the built-in operators and interpolation, on built-in types only: int,
     float, bool, str, char, and lists, maps, sets and optionals of them.

   Calls, method calls, `?`, `ref`, indexing, field access, ranges, `if`,
   `match`, and anything that would run a user `impl` (a user `ToStr`, `Eq` or
   `Ord`) are errors. A bare `none`, `[]` or `{}`
   can't be a constant, because it has no expected type.
3. **Evaluation (mod.const.3).** Constants have no order and no side effects,
   and are computed before `main` runs. A failure while computing one
   (overflow, division by zero) is an error
   reported at the constant before the program runs. The bootstrap reports it
   as a trap at the constant, before `main` starts.

```ail fragment
MAX_DEPTH := 200
KEYWORDS := {"let", "if", "else", "while", "fn", "return"}
ESCAPES := {'n': '\n', '"': '"', '\\': '\\'}
pub COLS := "ABCDEFGH"
WIDE := "{COLS} x {MAX_DEPTH}"      # interpolation of built-in types
NAMES := List[str]()                # a bare [] would have no type

BAD := KEYWORDS.contains("if")      # error: a method call
mut COUNT := 0                      # error: write `COUNT := 0`; no mutable globals
```

## Consequences

- A table computed by code (`PARTS := "a b".split(" ")`) can't be a constant.
  Build it in `main` or in a function.
- A constant is an immutable binding, so it is never a mutable place
  (own.place.1), and a local can't reuse its name (expr.bind.5).
- Evaluation is limited to built-in operations, so computing constants never
  has to run user code.
- A user type can be a constant's value (a struct literal) but can't be
  interpolated or compared in a constant, since that would run its `impl`.
  Allowing derived and automatic impls there is possible later.
- Custom initialisation (a user function run at startup) stays undesigned.

## Rejected alternatives

- **Any expression run at startup:** initialisation order and side effects
  (Go's `init` ordering problems); constants would no longer be order-free.
- **Literals only:** the programs' `KEYWORDS`, `ESCAPES`, `BINARY_LEVELS` and
  `SCRIPTS` are sets, maps and nested lists.
- **`const NAME = expr` keyword:** one more token per constant, and `:=` already
  means "bind a name to a value".
- **File-scoped constants:** mod.unit.1 already makes every top-level name
  visible module-wide, and a second scope rule is one more thing to learn.
- **Calls to user functions, or user `ToStr`/`Eq`/`Ord` impls, in a constant:**
  that is user code run at definition time, which is the case "no side effects"
  rules out.
- **Traps at startup:** a bad constant is a defect in the program text, so it is
  reported before the program runs, naming the constant.
