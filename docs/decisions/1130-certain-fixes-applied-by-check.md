# 1130. The commands that check first write certain fixes into the file

- **Status:** Accepted
- **Date:** 2026-11-15
- **Decided by:** designer (2026-11-15, option "apply certain fixes and go on"); which fixes are certain decided by AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 20 — AI semantic API
- **Spec rules:** fix.auto.1-3 (amends fix.check.1)
- **Amends:** [ADR 1047](1047-ail-check-fix.md) (its first two rejected alternatives)
- **Builds on:** [ADR 0140](0140-diagnostics-for-other-languages.md), [ADR 0203](0203-ail-fix-command.md)

## Context

AIL keeps its own spelling (the designer, 2026-11-15: no more syntax changes to look like other languages).
Agents still write other languages' forms in a first draft: `eprint`, `println`, `else if`, `&&`,
`m.contains_key(k)`, `m[k] else d`, a missing `mut`, a bare `Kind` after moving it into a module. Most of
these already carry a machine fix, but in the E13 agent benchmark agents ran `ail fix` in 3 of 307 AIL
trials, and a fix printed with a failed build only helps the next build. Under the benchmark's metric,
iterations to working code, every failed build counts.

E13 first drafts, all AIL arms, 1,336 error lines: `mod.vis.1` 631 (a separate 0.2 change), `expr.name.1`
214 (`eprint` 29, `contains_key` 17, bare module items 48, `has` 13, `exit` 12), `err.else.1` 50
(`m[k] else d` 34), `trait.method.3` 25 and `expr.bind.3` 9 (missing `mut`), `type.opt.4` 17
(`else none`), `expr.if.1` 14 (`else if`), `gram.foreign.1` 10 (`??`, `is`).

## Options considered

### Option A — apply certain fixes and go on (chosen by the designer)
- **Pros:** the slip costs no failed build and no extra command; the agent needs to know nothing.
- **Cons:** a check writes the file; an agent holding the old text must read it again.

### Option B — apply certain fixes, then fail
- **Pros:** the agent sees that the file changed before going on.
- **Cons:** still a failed build per slip.

### Option C — never edit; print `ail check --fix FILE` first
- **Pros:** no file written by a check (ADR 1047 as it was).
- **Cons:** agents seldom run a second command (3 of 307).

## Decision

1. **Certain fixes.** A fix (`fix.Fix`) carries `sure`. It is set only when the rewrite has one meaning,
   the one written: the parser's fixes of the rules in `compiler/fix/sure.ail` (another language's
   operator, keyword or punctuation: `&&`, `else if`, `??`, `let`, `const`, `->`, `::`, `use src.x`),
   and in the checker the fixes made with `Diag.with_sure_fix` or `Checker.sure_fix_last`: standard
   names other languages use (`eprint`, `exit`, `println`, `sys.exit`, a missing `use std.x`, `BigInt`),
   method renames with unchanged arguments (`contains_key`, `has`, `push_back`, `to_string`, `.iter()`,
   `.items()`, `set.keys()`), `unwrap_or(d)` and `unwrap()` on an optional, `is_some()`, `x or d` on an
   optional, `else none`, `else err` on an optional, a missing `mut` on a declaration or an argument,
   `+`, `%` and `*` on strings, a one-character string where a `char` is expected, `int` mixed with
   `float`, `(a..b).to_list()`, calls such as `len(xs)` (not `printf`, which adds no newline).
2. **Two new certain fixes.** `m[k] else d` where `m` is a map local is `m.get(k) else d` (`err.else.1`);
   a bare name that exactly one module the file imports exports as a public item is `module.Name`
   (`expr.name.1`).
3. **Applied by the check.** `check-program`, which `ail check`, `ail build`, `ail run` and `ail test`
   run first, takes the errors of its own check whose first fix is certain, applies them in the
   program's own files (the root file and its modules; never the standard library), writes each file
   changed, prints one `fixed PATH:LINE:COL: [RULE] TITLE` line per fix and a `note:` line on standard
   error, and checks again, at most five rounds. A clean check costs nothing extra: the fixing runs only
   when an error has a certain first fix (fix.auto.1-2). Module files are included because the change
   tasks fail mostly there (the R2 review, 2026-11-15).
4. **Off switch.** `ail check --no-fix` and `AIL_FIX=0`. The gates (`mac-gate.sh`, `win-gate.ps1`)
   and `ail selftest` set `AIL_FIX=0`, so suites that check wrong programs on purpose never rewrite
   them; the `fix` suite turns it on for its own test. `ail lsp` checks with `--no-fix`: an editor shows
   the problem in its buffer and leaves the repair to its code actions (fix.auto.3). `ail mcp` and
   `ail task` keep the default, as agent tools.

## Rationale

AI understanding is first: the file the agent reads after the build is correct AIL, and each `fixed` line
says what changed, so the agent learns the AIL form from its own code. Certainty is the line between
repair and guessing: a did-you-mean, a renamed keyword or a `todo` arm is the author's call and stays a
printed fix. A nearest-name guess is never certain: in E13 one turned `eprint` into `print`, which
compiles and sends errors to standard output. Every edit settles against the file it is in, from the
check's own diagnostics, so no position from one file is applied to another.

## Consequences

- E14 measures with the fixes on; a slip that only certain fixes cover no longer fails a build.
- `ail fix` and `ail check --fix` are unchanged and still apply every safe fix.
- A new fix is a guess until its author marks it certain; the marking is reviewed like any rule change.
- Not covered yet: `x in xs` inside parentheses and `x is some` in a statement (no fix exists), and
  errors the check never reaches because an earlier syntax error or bad `use` stops it.

## Rejected alternatives

- **Compile the fixed text and leave the file alone.** The file would stay wrong, and the next edit would
  start from code that doesn't compile.
- **All safe fixes by default.** Some are guesses (`did you mean`, `name it _`, `todo` arms); writing them
  unasked changes meaning.
- **Certainty by rule for the checker.** One rule carries both kinds (`expr.name.1` has did-you-mean
  guesses and standard-name rewrites); the mark has to be per fix.
- **An `ail fix` reminder in every failed build.** Option C; agents don't run it.
