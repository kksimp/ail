# 0301. Scoped mutex access: `with m as mut x`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 11 Concurrency
- **Spec rules:** `conc.share.6`, `lex.kw.1`, `conc.par.2`

## Context

ADR 0189 gave `Mutex[T]` a read-modify-write pair: `x := m.lock()` then
`m.unlock(v)`. The pair can be forgotten on one path, and an early
`return` between the two traps at task end. The brief asked for a scoped
form that writes the value back by itself.

Measured with `tools/tokbench/run.sh` (mean of five tokenizers) on a
three-line `add(m, n)`:

| Form | Tokens |
|---|---|
| `x := m.lock()` / `m.unlock(x + n)` | 25.6 |
| `m.update(\|x\| x + n)` | 21.6 |
| `with m as x` / `x += n` | 21.4 |
| `with m as mut x` / `x += n` (chosen) | 22.4 |

On the whole of `workers.ail` the `mut` costs 1 token of 366 (0.3%), a tie.

## Options considered

### Option A — `with m as mut x` block (chosen)
- **Pros:** −12.5% against the pair; the lock can't be left held; the body
  can hold any statements (`workers.ail` closes a channel inside); `mut`
  says the variable is written back, as in `for mut x in xs` (expr.loop.6).
- **Cons:** a new reserved word, `with`.

### Option B — `with m as x` (no `mut`)
- **Pros:** one token less.
- **Cons:** a variable that is silently written back without `mut` breaks
  the rule that writes are marked (ADR 0060); a tie on tokens, so AI
  understanding (criterion 2) decides for `mut`.

### Option C — `m.update(|x| x + n)`
- **Pros:** no new keyword; same tokens as `with`.
- **Cons:** `|x|` is only for key functions (ADR 0250); one expression can't
  close a channel or return early, so the pair would still be needed.

### Option D — keep `lock` / `unlock`
- **Pros:** nothing new.
- **Cons:** 14–20% more tokens; easy to get wrong on early exits.

## Decision

`with m as mut x` followed by a block takes the lock, binds a copy of the
value to `x`, and on every exit stores `x` and releases. A trap releases
and keeps the old value. The block can't name `m` again. `lock`/`unlock`
stay for code that needs them.

## Rationale

Tokens (criterion 1) rule out the pair; between the `with` spellings the
difference is a tie, so AI understanding (criterion 2) decides: `mut` marks
the write-back, and a trap that leaves the old value makes each `with` all
or nothing.

## Consequences

- `with` is reserved; no program in the repository used it as a name.
- `conc.par.2` treats a `with` as synchronisation.
- `with m as x` is an error whose help gives both fixes (add `mut`, or use
  `m.get()`).

## Rejected alternatives

- `with m as x`: hides a write; tie on tokens.
- `m.update(|x| ...)`: one expression only; widens closures.
- Only `lock`/`unlock`: more tokens and easy to misuse.
