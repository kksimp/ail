# 0140. Names that other languages spell differently get a fixed help

- **Status:** Accepted
- **Date:** 2026-10-05
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 2 — Core syntax (expressions); 20 — AI semantic API (diagnostics)
- **Spec rules:** [expr.name.2](../spec/05-expressions.md)
- **Extends:** [ADR 0107](0107-unknown-names.md) (nearest-name suggestions)
- **Evidence:** `examples/interp.ail`, `examples/json.ail`, `examples/sheet.ail` and their first drafts; a probe of the bootstrap checker; tokbench, mean of five tokenizers

## Context

ADR 0107 suggests the nearest visible names for an unknown name. The
drafting agents for three programs (2,036 lines of AIL) guessed names from
other languages, and the guesses repeat:
- `str.from_chars(xs)`: 4 calls in 2 of the 3 programs (interp 2, json 2);
- free `min(a, b)` and `max(a, b)`: 5 calls in sheet;
- `panic(...)` in interp's first draft.

Those are the observed guesses. The review also predicted the habits of
agents trained on Python, Rust, Kotlin, JavaScript and Swift (`len(x)`,
`"".join(xs)`, `contains_key`, `loop`, ...). Those are predictions, not
observations.

A probe shows what happens today. 16 foreign spellings (`len(xs)`,
`sorted(xs)`, `sum(xs)`, `min(1, 2)`, `chr`, `ord`, `str(5)`, `int("5")`,
`abs(-3)`, `s.length`, `s.strip()`, `s.startswith(p)`, `xs.append(4)`,
`xs.size()`, `panic("x")`, `str.from_chars(cs)`) each give an error
(`expr.name.1`, or `type.struct.6` for `s.length`) and no help, except
`min(1, 2)`, which gets "did you mean `main`?" (one edit away). The
bootstrap's limit is a third of the name's length, at least one.

## Options considered

| Option | Result |
|---|---|
| **Fixed table of foreign spellings, each with the AIL form (chosen)** | exact fix in one step |
| Nearest-name suggestions only (ADR 0107) | 15 of 16 probes: no help; 1: a wrong one |
| Accept the foreign names as aliases | two names for one meaning, and a larger prelude |

## Deciding criterion

Token usage **ties**: a diagnostic is not program text, and a table lookup
after resolution fails costs nothing on a valid program. **AI understanding**
decides, by ADR 0107's own criterion: one precise, fixable diagnostic. A fixed
table beats nearest names for these cases because:
1. **Most are different words, not misspellings.** `append` is not near
   `push`, `strip` is not near `trim`, `len` is not near `lines`. Only a few
   (`startswith` → `starts_with`) are one edit away, and edit distance finds
   those once the AIL name exists.
2. **The fix is often a rewrite, not a rename.** The receiver moves
   (`"".join(xs)` → `xs.join("")`, `len(x)` → `x.len()`, `min(a, b)` →
   `a.min(b)`), the construct changes (`m.get(k, d)` → `m.get(k) else d`,
   `loop` → `while true`), or the result type differs (`int(s)` gives
   `int?`, `removeprefix` gives `str`, `strip_prefix` gives `str?`).
3. **Habits repeat, typos don't.** The same wrong name came up in two of three
   programs and five times in one. An entry is written once and helps every
   later author, human or AI.
4. **The right form is stated, not guessed.** Edit distance offered `main`
   for `min`, and may offer the type `char` for `chr` once it exists.

Measurements chose the AIL forms ([ADR 0135](0135-string-literals-and-methods.md),
[ADR 0137](0137-integer-methods-and-min-max.md)), and they win over the
habits: `xs.join("")` is 30.8 tokens against 35.6 for `str.from_chars(xs)`
(−13%), and `a.min(b)` is 55.2 against 58.2 for `min(a, b)` on sheet's lines
(a tie, decided by [ADR 0084](0084-no-shadowing-module-names.md)). The table sends authors to the measured form.

## Decision

1. **Where it applies (`expr.name.2`).** When a call or method call fails
   `expr.name.1` (unknown function, unknown method on a known receiver type, or
   a built-in type used as a function, as in `str(x)`), and the name and
   receiver are in the fixed table, the diagnostic also gives a help that names
   the AIL form as a concrete call, in addition to ADR 0107's nearest names.
   Names that resolve are never looked up, so the table changes no valid
   program, and a local named `count` or `min` keeps working.
2. **The table is in the spec.** `expr.name.2` lists the required entries, so
   implementations and tests share one list, and the bootstrap tests each
   entry. The wording is the implementation's; the content (the AIL form, and
   a note where the result type differs) is the spec's.
3. **Required entries,** by the habit they serve:
   - **Python:** `len(x)` → `x.len()`, and for a str `s.chars().len()`;
     `str(x)` → `"{x}"`; `int(s)` → `s.to_int()` (`int?`); `float(x)` →
     `s.to_float()` or `n.to_float()`; `ord(c)` → `c.code()`; `chr(n)` →
     `char.from_code(n)` (`char?`); `abs(x)` → `x.abs()`; `sorted(xs)` →
     `xs.sorted()`; `sum(xs)` → `xs.sum()`; `min(a, b)` → `a.min(b)`,
     `min(xs)` → `xs.min()` (`max` likewise); `"".join(xs)` →
     `xs.join("")`; `strip`, `lstrip`, `rstrip` → `trim` (both ends);
     `startswith`, `endswith` → `starts_with`, `ends_with`; `removeprefix`,
     `removesuffix` → `strip_prefix`, `strip_suffix` (`str?`; write `else s` to
     keep the original); `rjust`, `ljust`, `zfill` → `pad_left(w, fill)`,
     `pad_right(w, fill)` (the fill char is required); `splitlines` →
     `lines`; `isdigit`, `isalpha` → `is_digit`, `is_alpha`; `m.get(k, d)` →
     `m.get(k) else d`; `m.items()` → iterate `m.keys()` and index `m[k]`;
     `xs.append(x)` → `xs.push(x)`.
   - **Rust:** `contains_key` → `contains`; `is_whitespace`, `is_alphabetic`,
     `is_numeric` → `is_space`, `is_alpha`, `is_digit` (ASCII 0–9 only);
     `loop` → `while true`.
   - **JavaScript, Java, Kotlin and Swift:** `x.length`, `x.size`,
     `x.size()`, `x.count` → `x.len()`; `xs.add(x)` on a list → `xs.push(x)`;
     `has`, `hasKey` → `contains`; `padStart` → `pad_left`.
   - **Traps:** `panic` (Rust, Go), `fatalError` (Swift) → `unreachable(msg)`
     ([ADR 0139](0139-unit-results-statement-else-and-divergence.md)); `assert(c)`
     → an `if` that calls `unreachable(msg)`.
   - **Seen in the drafts:** `str.from_chars(xs)` → `xs.join("")`, free
     `min`/`max`, and `panic`, as above.
4. **Scope.** The table covers names. A help tied to one rule lives with that
   rule: a negative index, `index_of(x) == -1`, `{{`, `for c in s`, `s[i]`.
   [ADR 0131](0131-maps-and-sets.md), [0134](0134-characters.md),
   [0135](0135-string-literals-and-methods.md),
   [0136](0136-list-methods-and-slices.md) and
   [0137](0137-integer-methods-and-min-max.md) each name the helps for their
   own API. This ADR gathers them so that one rule makes them required, and
   each entry lands with the feature it names.
5. **Growing the table.** A new entry needs a habit seen in a program or
   documented for a mainstream language, and its AIL form must exist.

```text
a.ail:7:11: error[expr.name.1]: unknown name `min`
  help: write `a.min(b)` for two values, or `xs.min()` for a list
```

## Consequences

- The bootstrap checker gets one table (name, receiver kind, help) and one
  test per required entry.
- The quick reference gets a short block, "if you'd write X in Python", with
  the same entries (the review's suggestion).
- Other habits stay open: a help is cheap to add later, and an alias isn't.
- The table is inexact for habits that mean something else in AIL
  (`is_numeric` isn't `is_digit` for Unicode digits), so such an entry states
  the difference.

## Rejected alternatives

- **Nearest-name suggestions only:** no help on 15 of 16 probes and a wrong
  one on `min`. It stays as the fallback for real typos.
- **Aliases** (accept `len(x)`, `contains_key`, `append`, `str(x)`): two
  names per meaning. Free functions also take prelude names that locals can't
  reuse (ADR 0084), and `int(s)` would hide that parsing can fail.
- **Leaving the table to the implementation:** the helps would drift, and
  nothing would test them; every implemented behaviour cites a spec rule.
- **Changing the language to fit the habit** (`str.from_chars`, free `min`):
  `str.from_chars` costs 13% more tokens than `join`, and free `min`/`max`
  tie on tokens but take prelude names
  ([ADR 0137](0137-integer-methods-and-min-max.md)).
