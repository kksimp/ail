# Agent findings for AIL 0.2.0: an outside review and an app-build test

> **Non-normative handoff note,** 2026-11-25. Findings from two agent experiments run against AIL 0.2.0
> (checkout `da34c30`), handed to the AIL project to work on. Nothing here is decided: each item says what was
> seen, what the code shows, and a suggested direction. Changes to the frozen 0.2.0 language go to the designer.

The designer's goal is to show that AIL is more helpful for agentic coding than current languages. Both experiments
say the tools are real and work, but neither shows a net gain yet. The gap has two parts: **trust** (a tool's report
that disagrees with its own graph) and **familiarity** (turns spent looking up names and edge behaviour that an agent
already knows in Python).

## 1. Source A: outside review of the AI-first tools

An AI coding agent from another vendor, run by the designer on the Mac, built the same feature three ways on a
two-file incident-report CLI: by hand, with the AIL CLI tools, and over `ail mcp` with the task workflow. The
feature added an optional `--level` filter to `report.analyze`, changing its public signature, its caller, its
counting and its tests. All three results were byte-identical and passed `ail check`, three module tests and a real
CLI run.

### 1.1 Breakdown of the review, point by point

| # | Area | What the reviewer saw | Their verdict |
|---|---|---|---|
| A1 | Overall | The compiler answers graph-backed questions, builds per-function context, applies declaration-aware multi-file edits, records a change workflow, and serves it over MCP. | "More than marketing", but one agent, one feature, three sequential runs: not a productivity result. |
| A2 | `ail context . report.analyze --budget 400` | 887-byte bundle: signature, body, transitive failure effect, caller, the `Summary` type and two tests. | The most persuasive mechanism: less repository reading. |
| A3 | `ail ask . 'what calls analyze'` | 184 bytes, exact call sites; confirmed in `compiler/api/ask.ail` that it uses the checker graph, not grep. With `--json` it found `main` and all three tests. | Precise, but a **finite query grammar** (callers, callees, reads, writes, effects, uses), not open questions. |
| A4 | `ail api --text .` | 5,537 bytes for ~2 KB of source, because it includes the loaded std surface. | Noted as larger than the program. |
| A5 | Structured edits | A JSON `change_sig` op added the optional parameter and rewrote every call site; `ail edit . --diff` reported 2 files, 0 errors. | A real benefit for multi-file signature changes; still needs design and diff review. |
| A6 | Diagnostics | The hand-written draft got useful errors for a shadowed binding, an optional-value type mismatch and call sites missing the new parameter. | Ordinary and useful. |
| A7 | MCP | `initialize`, `tools/list`, `ail_context`, `task_start`, `ail_edit`, `task_verify`, `task_status`, `task_finish` all worked; 23 tools advertised. | MCP **only dispatches to the CLI**; it adds no analysis. Useful for hosts that call MCP tools. |
| A8 | Task ledger | Recorded 2,465 ms and "304 estimated tokens" for verification. | The token figure is a **character heuristic**, not model tokens or agent effort, and reads like a measurement. |
| A9 | Certificate | `ail verify --check .ail/cert.json`: 14 members reproduced. | Proves the recorded check/test reproduces on this machine and checkout, not correctness or portability. |
| A10 | Verify output | Distinguishes static check, contracts, tests and unverified declarations; the entry point listed as unverified. | A good format: green does not silently claim the CLI path was tested. |
| A11 | **Bug** | Verify said `no test reaches fn:report.analyze`, though all three passing `@test` functions call it and `ail ask` finds those calls. In the impact JSON each test's `reaches` held only its own id. The certificate keeps the same error. | "Could mislead in the other direction elsewhere; coverage must be checked rather than trusted." The top priority. |
| A12 | `task_finish` | Marks the task done while listing unverified functions. | Consistent if "done" means "verification ran", but **not a completeness guarantee**; agents must read `tested` and `unverified`. |
| A13 | `ail try FILE` | Worked: scratch copy, check, three tests, original untouched; "~430 tokens" (same heuristic). | Useful for one-file patches. It checks patch sets given to it; it does not generate competing patches. |
| A14 | **Bug** | `ail try . --set ...` failed: `scratch: can't copy .`. | Blocks real multi-file use. |
| A15 | Productivity | No matched prompts, time, context or tokens; the by-hand run benefited from the earlier runs. | No measured win. Needs many matched tasks and independent runs. |
| A16 | Contracts | This change had none; 0 proofs. | Exposed, not exercised. |
| A17 | Portability | Certificate replay only on the same Mac and checkout. | Not established. |
| A18 | Conclusion | Would use AIL tools for AIL maintenance and contained apps with explicit tests, pinned toolchains and independent checks of changed paths; would pick C, Rust or Python for general production today. | "Promising infrastructure around a young language, not yet a net productivity multiplier." Fix the reachability mismatch and package-level `ail try` first. |

The reviewer's capability table, condensed: semantic queries, context bundles, structured edits, MCP, task ledger and
certificates are all **implemented and exercised**; candidate patch trials work for a file but **failed for a package
directory**; tested-vs-unverified reporting is implemented but **contradicted direct call evidence**; contracts and
cross-platform repeatability were **not exercised**.

### 1.2 Bug A11: root cause (confirmed in the code)

`compiler/api/impact.ail`, `impact_between`: `reaches` (api.impact.4) records, for each affected test, only
`root[test]`, the **first** changed declaration whose search reached it. Every changed function is seeded into
`affected` and `root` before the search runs, with `root[id] = id`. In the review the `change_sig` rewrote the call
in each test, so each test was itself changed: its `root` was its own id, the search from `report.analyze` found it
already affected and never recorded `analyze`. `tools/verify/verify.ail` (verify.run.4) then reported every changed
declaration missing from all `reaches` lists as `no test reaches`. The same happens whenever a test and the function
it calls change together, which is the normal shape of a feature edit.

Suggested fix: for each changed declaration, walk its users and record it on every test reached, so `reaches` lists
**every** changed declaration a test reaches, its own id included. A drafted patch is in Appendix A (not built or
gated). Gate case: in `Gate.cert`, change `sign` and the test `positive` together and expect
`"reaches":{"positive":["fn:positive","fn:sign"]}` and no `no test reaches fn:sign`. Spec: reword api.impact.4
("every changed declaration each affected test reaches").

### 1.3 Bug A14: root cause (confirmed in the code)

`tools/ailtry/ailtry.ail`, `try_set`: the copy goes to `.ail/try/N/{last_part(path)}`, relative to the working
directory. For `.` that is `.ail/try/N/.`, and `fs.copy(".", ...)` copies a directory into itself, which std.fs.12
says fails. Suggested fix: copy a directory entry by entry, skipping `.ail` and `.git`, and name the copy `program`
when the last part is `.` or `..` (Appendix A). Gate case: in `Gate.try_cmd`, a `pkg/` with `main.ail` and
`calc/calc.ail`, run `ail try . --set good.json` from inside it, expect `check ok; tests 1 passed`. Spec: try.run.1
should say a directory may be the one the scratch sits in.

## 2. Source B: agent builds one app in AIL and in Python

The ail-train project ran one frontier coding agent through the same small ledger app (stdin commands, fixed-width
report) in AIL and then in Python, graded byte for byte on 8 hidden cases (ail-train PR #36,
`results/agent-app/2026-11-25-ledger/`).

| Measure | AIL | Python |
|---|---|---|
| Hidden cases | 8/8 | 8/8 |
| Model turns | 11 | 3 |
| Cost (USD) | 0.25 | 0.11 |
| Wall time (min) | 1.2 | 0.5 |
| Program size (tokens) | 1,769 | 1,435 |
| First clean `ail check` | turn 7 | n/a |

**What helped:** the card's `words()`, `pad_left`/`pad_right` and tuple sort keys (`(kind, 0 - balance, name)`
worked first try); spec type.str.17 on `words()`; the auto-fixer turning `to_string` into `to_str` with a rule id.

**What cost turns (the familiarity tax):**

| # | Pain point | Suggested direction |
|---|---|---|
| B1 | Most extra turns went to **looking up method names and edge behaviour**. | One call that answers "what does method M do exactly": signature, the spec rule sentence, a one-line example, in the CLI and MCP (`ail ask std` plus the rule text). |
| B2 | Card does not say whether `pad_left`/`pad_right` and `{x:>6}` width specs **truncate** a longer string. | State it on the card. |
| B3 | Card does not say whether `is_alpha`/`is_alnum`/`is_digit` are **ASCII-only or Unicode**. | State it on the card. |
| B4 | Card does not give the exact **whitespace set** of `words()` (does it split on `\r`, `\v`, `\f`?). | State it on the card and in type.str.17. |
| B5 | Card lists `to_string` as an auto-fixed habit but **never names `to_str`**; the agent learned it only from the fixer. | Write the correct name next to each habit. |
| B6 | **Char-to-digit** was written as `cs[i].to_str().to_int()?`; `c.to_digit(10)` exists but was not found. `parse_amount` was ~35 lines against Python's 8. | Put `to_digit` on the card next to the char predicates. |
| B7 | The agent never tested a value wider than its column because it could not tell from the docs what would happen. | Same as B2; edge facts on the card also make agents test less blindly. |

Verdict of that run: too easy to separate the languages. It shows AIL is learnable from the docs in one try (no failed
compiles, no traps), not that it is better.

## 3. Suggested work, by value to the goal

1. **Fix A11 and A14** with their gate cases (Appendix A). Trust in verify and certificates comes first.
2. **Edge facts on the card (B2-B7).** Find the true behaviour in `compiler/rt/native/` and the spec, then state it in
   the fewest tokens, inside the card's token budget. Then audit the ~40 most-used str/char/List/Map methods for
   other edge facts an agent would look up (out-of-range slices, empty input, sort stability, map order, empty split
   separator).
3. **One-call std lookup (B1, A7).** `ail ask std NAME` (and an MCP tool) returns signature + spec rule text +
   example, and maps habit names from other languages (`to_string`, `len`, `parseInt`, `append`) to the AIL name.
   Rewrite the 23 MCP tool descriptions to say when to use each tool and what it returns.
4. **Honest status words (A8, A12, A13).** `task finish`/`status` say `done, verified` only when nothing is
   unverified, else `done, N unverified: ...`; JSON gains a `complete` boolean (no renamed keys); optional `--strict`
   refuses to finish while anything is unverified. Every heuristic token figure says `estimate`.
5. **Forgiving `ail ask` (A3).** Accept common paraphrases ("who calls X", "where is X used", "which tests cover X",
   "where is X defined"), and on an unknown question print the supported forms and the closest match (exit 2, never
   answer a different question). Add "tests for X" (tests reaching X transitively) and "where is X" (file:line and
   signature) from the existing graph code.
6. **More habit fixes.** Check each common Python/Rust/JS/Go habit against `compiler/fix/` and `tests/mistakes/`
   (string `+`, `"${x}"`, `parse_int`/`int(s)`/`str(x)`, `len(x)`, `.append`, `.length`, `unwrap_or`, `lower()`,
   `None`/`null`/`nil`, `&&`/`||`/`!`, `else if`, `range(n)`, `contains_key`, `-> T`, `let`/`var`/`const`), and add a
   certain rewrite or a one-line "write X" hint for each gap, with a mistakes case.
7. **Measure, don't assert (A15).** A fair claim needs many matched tasks and independent runs, with model tokens,
   turns and time recorded per language; ail-train's `tools/agent_app` is a starting harness.

Items 2-6 change tools, docs and diagnostics only, not the frozen 0.2.0 language.

## Appendix A: drafted patches for A11 and A14 (not built or gated)

```diff
--- a/compiler/api/impact.ail
+++ b/compiler/api/impact.ail
@@ impact_between: after the affected search
+    mut test_ids := Set[str]()
+    for i in 0..after.api.fn_ids.len()
+        if after.is_test(i)
+            test_ids.add(after.api.fn_ids[i])
+    # api.impact.4: every changed declaration each test reaches. A changed test reaches itself and
+    # whatever else changed below it; the first-reached `root` alone would hide the rest.
+    mut reached_by := Map[str, List[str]]()
+    for c in changed
+        mut seen := Set[str]()
+        mut walk := List[str]()
+        for x in (if c.starts_with("type:") then type_seeds(c, old_members, new_members, known_list) else id_and(c, also_reaches(c, known_list)))
+            if not seen.contains(x)
+                seen.add(x)
+                walk.push(x)
+        mut k := 0
+        while k < walk.len()
+            cur := walk[k]
+            k += 1
+            if test_ids.contains(cur)
+                mut got := reached_by.get(cur) else []
+                if not got.contains(c)
+                    got.push(c)
+                reached_by[cur] = got
+            for f in users.get(cur) else []
+                if not seen.contains(f)
+                    seen.add(f)
+                    walk.push(f)
@@
-                reaches.push((after.prog.fns[i].name, [root.get(id) else id]))
+                reaches.push((after.prog.fns[i].name, reached_by.get(id) else [root.get(id) else id]))
@@ (drop the later, now duplicate `test_ids` block)
```

```diff
--- a/tools/ailtry/ailtry.ail
+++ b/tools/ailtry/ailtry.ail
@@ fn try_set
-    copy_to := "{root}/{last_part(path)}"
+    copy_to := "{root}/{copy_name(path)}"
@@
-    fs.copy(path, copy_to) else err
+    copy_program(path, copy_to) else err
@@ new helpers
+## The name of the copy: the last part of `path`, or `program` when that is `.` or `..` (`ail try .`).
+fn copy_name(path str) str
+    name := last_part(path)
+    if name == "." or name == ".." or name == "" then "program" else name
+
+## Copies the program at `path` to `to` (try.run.1). A directory is copied entry by entry, leaving out
+## `.ail` and `.git`: the scratch directory may sit inside the program (`ail try .`), and a copy of a
+## directory into itself fails (std.fs.12).
+fn copy_program(path str, to str) ! fs.IoError
+    m := fs.metadata(path)?
+    if not m.is_dir
+        fs.copy(path, to)?
+        return
+    fs.make_dir(to)?
+    for name in fs.list_dir(path)?
+        if name == ".ail" or name == ".git"
+            continue
+        fs.copy("{bare(path)}/{name}", "{to}/{name}")?
```
