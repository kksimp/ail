# 1205. Four agent gaps from the audit: temp paths, `else` with an operator, the full card, task handles in a list

- **Status:** Accepted
- **Date:** 2026-11-26
- **Decided by:** the designer approved the four fixes on 2026-11-26 (temp paths: "allow an exception for tmp"); the wording and the `@tmp` spelling are the AI's, delegated, [ADR 0076](0076-delegated-decisions.md)
- **Topic:** 13 — Package management (capabilities); 21 — Tooling
- **Spec rules:** [eff.allow.13](../spec/18-effects.md), [tool.new.6](../spec/22-tools.md), [expr.prec.1](../spec/05-expressions.md), [tool.card.1](../spec/22-tools.md), [tool.new.3](../spec/22-tools.md), [conc.task.5](../spec/11-concurrency.md)
- **Builds on:** [ADR 1195](1195-new-projects-deny-by-default.md), [ADR 0144](0144-else-as-an-expression.md), [ADR 1110](1110-language-card-budget-16000.md), [ADR 1204](1204-task-group-results.md)

## Context

A read-only audit of `main` found four causes of agent trouble that are not unfamiliarity.

1. A project from `ail new` runs with `caps fs=.` (ADR 1195). A grader that gives the program a path under `/tmp`, `$TMPDIR` or the macOS `/var/folders/...` directory makes the program trap under `ail run`, though it passed where the agent tested it.
2. `n := m.get(k) else 0 + 1` is `else (0 + 1)` (`expr.prec.1`). It compiles with no diagnostic and counts nothing.
3. The `AGENTS.md` that `ail new` writes says to read `ail card`, which prints the 3,000-token core card. The measured gain (0.97 against 2.51 failed builds) came from the full card. `llms.txt`, `for-agents.md` and the card gave different load orders and wrote `./ail` and `ail` interchangeably.
4. `tasks.push(spawn f(x))` (Opus trial 4) is refused by `conc.task.5`, whose help did not name `TaskGroup[T]` first.

## Options considered

### Temp paths
- **A. A scope word `@tmp` in `caps`, written by `ail new` (chosen).** `fs=.,fs=@tmp` expands, when the policy is built, to `/tmp`, `/var/tmp` and the absolute directories of `TMPDIR`, `TEMP` and `TMP`. One visible, versioned word; `ail review` reports it like any scope; removing it narrows the project again.
- **B. Write `fs=/tmp` literally.** Rejected: the macOS temp directory is a per-user path under `/var/folders` that no manifest can name, which is the case that failed.
- **C. Grant temp to every run, manifest or not.** Rejected: it hides the limit in the runtime and an old manifest changes meaning.

### `else` followed by an operator
- **A. A warning at the operator with a suggested (not certain) fix, `(e else a) OP b` (chosen).** The reading the line looks like is a guess; the written reading is valid code, so the fix is not certain.
- **B. Make `else` bind tighter.** Rejected: no syntax or precedence changes (designer's rule; ADR 0144 settled it).
- **C. A certain fix that wraps the fallback, `else (a OP b)`.** Rejected: it silences the warning and keeps the bug.

The parser records the operator of each unparenthesized binary fallback (`Program.else_ops`, optional like `asks`), because the tree has no parenthesis node and the checker has no source text.

### The full card
- **A. `ail card full` prints the full card; the kit, the MCP `instructions`, `llms.txt`, `for-agents.md` and the card name it first (chosen).** Bare `ail card` stays the core card for a context too small for the full one.
- **B. Make bare `ail card` print the full card.** Rejected: it changes `tool.card.1` and the core card's purpose, and a small-context agent loses its short route.

### Task handles in a list
- **A. A dedicated `conc.task.5` message and help for `push(spawn ...)` that names `TaskGroup[T]` with a snippet (chosen).**
- **B. A certain fix for the loop.** Rejected for now: it must rewrite the list's declaration, the push and the later reads, and the result type is not known at the push.

## Decision

1. `@tmp` as a scope of `fs`, `fs.read` and `fs.write` is the system temporary directories (`eff.allow.13`); `ail new` writes `caps fs=.,fs=@tmp,env,clock,random` (`tool.new.6`). Everything else stays scoped.
2. A fallback that is a binary operation written without parentheses is a warning `expr.prec.1` at its operator, with a suggested fix. The card's block "Compiles, but is not what you meant" says so.
3. `ail card full` prints the whole full card (`tool.card.1`). The kit lesson, `ail mcp`'s `initialize` instructions, the README copy, `ail help`, `llms.txt`, `for-agents.md` and the card agree: full card first, bare `ail card` is the core, `ail` is the installed command and `./ail` the checkout's.
4. `xs.push(spawn f(x))` reports `conc.task.5` with help that shows `TaskGroup[T]`, `g.add(spawn f(x))` and `g.wait_all()?`.

## Rationale

AI understanding first: each change turns a silent or late failure into a message at the line, or removes a wrong instruction. No syntax changes. Speed: the policy gains up to five path entries; the parser records a position per qualifying `else`.

## Consequences

- Existing projects are unchanged; a project made before this ADR keeps `fs=.` until its author adds `fs=@tmp`.
- A std source line with an unparenthesized operator after a fallback now warns; `std/time/date.ail` was changed to bind the fallback first.
- The AGENTS.md lesson is a little longer (the token limit of `tool.new.3` is unchanged).
- Not run here: the change was written in a cloud container and is gated on the Mac (ADR 0842).
