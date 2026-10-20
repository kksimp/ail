# 0189. Mutex access: `get`, `set`, `swap`, and `lock`/`unlock` pairs

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Concurrency ([status.md](../status.md) topic 11)
- **Spec rules:** conc.share.2, conc.share.5

## Context

`conc.share.2` requires shared mutable state to live in a synchronised
type such as `Mutex[T]`. AIL has no closures and no scoped `with` block, and
borrows are second-class (ADR 0062), so a guard object that refers into the
mutex can't be expressed.

## Options considered

Measured with `tools/tokbench` on a counter loop (`for i in 0..n` adding
`i` under the lock), mean of five tokenizers:

| Form | Tokens | Notes |
|---|---|---|
| `m.set(m.get() + i)` | 29.8 | racy: another task can run between get and set |
| `m.update(fn(x) x + i)` | 30.8 | needs closures |
| `with m as mut x` + `x += i` | 31.0 | needs a new block form |
| `mut g := m.lock()` + `g.value += i` | 33.2 | needs a borrowing guard |
| `x := m.lock()` + `m.unlock(x + i)` (chosen) | 34.2 | works today |

### Option A — `get`/`set`/`swap` plus `lock() T` / `unlock(v)` (chosen)
- **Pros:** no new syntax; the read-modify-write is visible and correct;
  single reads and writes stay one call.
- **Cons:** about 3 tokens more per critical section than a `with` block;
  a forgotten `unlock` is caught at run time, not compile time.

### Option B — `with m as mut x` block
- **Pros:** scoped, can't forget to unlock, fewest tokens of the safe forms.
- **Cons:** new grammar; deferred until scoped blocks are designed.

### Option C — closures (`m.update(fn(x) ...)`)
- **Cons:** closures don't exist.

## Decision

Option A. `lock` while holding, `unlock` without holding, and a task that
ends holding the lock each trap (`conc.share.5`); `get`, `set` and `swap`
wait while another task holds the lock.

## Rationale

The token difference is under 10% on a construct that is rare in programs
(critical sections), which ADR 0076 counts as a tie, so AI understanding
decides: Option A needs nothing an agent doesn't already know, and every
misuse traps with a rule ID.

## Consequences

- Open question: a scoped `with` block for locks and other resources,
  which would also serve `defer`-like cleanup.

## Rejected alternatives

- `get`/`set` alone: invites lost updates.
- Closures and guards: depend on features not in the language.
