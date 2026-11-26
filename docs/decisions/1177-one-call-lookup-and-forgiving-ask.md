# 1177. One call answers a lookup: names in full, questions in any common form, the nearest command

- **Status:** Accepted
- **Date:** 2026-11-25
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); asked for by the designer (tooling lane of the 2026-11-25 pain-point pass)
- **Topic:** 22 — Tools
- **Spec rules:** [api.ask.1, api.ask.7, api.ask.8, api.ask.9, api.find.9, api.find.10, tool.help.4](../spec/22-tools.md)
- **Builds on:** [ADR 0881](0881-ail-ask.md), [ADR 1051](1051-core-card-and-std-lookup.md), [ADR 1123](1123-ail-ask-std-answers-in-one-step.md), [ADR 1057](1057-ail-help.md)

## Context

Agents with a shell use `build`, `check`, `run`, `test` and a lookup, and nothing else: MCP, `task`, `context` and
`edit` were called in no measured trial. So a tool helps only through what those few commands print in one call.
Three measured costs sit there:

- In the 2026-11-25 app test the same program took 11 model turns in AIL against 3 in Python, and most of the extra
  turns were lookups of one method's exact behaviour (`pad_left`, `is_alpha`, `words`, `to_digit`).
  `ail ask std NAME` printed one sentence cut at 120 characters, one name per call, and nothing for a name from
  another language (`to_string`), which the checker already knows how to answer.
- The outside review found `ail ask PROGRAM` takes a fixed set of verbs: `where is X used` and `which tests cover X`
  were errors. In E13, 195 `explain` calls and 37 `card` topics failed on a guessed word: each lookup door has its
  own grammar, and `ail ask pad_left` (no `std`) tried to load a program named `pad_left`.
- A guessed command word (`ail compile`, `ail docs`) costs a turn and then the whole command list.

## Decision

1. **Names are answered in full (api.find.9).** When every word of `ail ask std WORDS` is a function or method
   name, each is printed with the sentences of its rule and the lines of the rule's checked example that call it;
   several names in one call are each answered. A name from another language is answered from the checker's
   foreign-name tables (`compiler/check/foreign.ail`), which gain `rename_of_kind` and `help_of_kind` so the
   lookup and the diagnostics read one table.
2. **`ail ask` with no program is a lookup (api.ask.9).** One name goes to the std lookup; several words go to the
   how-to that answers them, else to the std search.
3. **Questions in any common form (api.ask.1), and two new ones.** `called`, `used`, `written`, `defined` and the
   like are verbs; `what is called by f` is `callees`. `tests` lists the tests that reach a declaration
   (api.ask.7), `where` its declaration and signature (api.ask.8). An unreadable question lists the forms.
4. **An unknown command names the one meant (tool.help.4):** another toolchain's word, a prefix, or one or two
   edits. The launchers pass an unknown word to the native `help`, so the table lives once.
5. `ail help` ends with where the language is (`ail card`) and how to look a function up.

6. **The JSON prelude is in the lookup (api.find.10).** `Json.to_text` was in no index: its rules are `json.*`,
   not `std.*`. A hands-on trial lost its worst bug to this (`to_json().to_str()` compiles and writes the wrong
   text; `ail ask std json to text` found nothing).
7. **The compiler binary run without its launcher** checks a program for `check FILE` (it read the file as a
   bootstrap dump and crashed) and says where the launcher is for `test`, `task` and the other commands the
   launcher runs.

No new command and no new MCP tool: `ail_find` and `ail_ask` reach the same code.

## Consequences

- A lookup of names prints more than ten lines (four functions a name at most). Against a second model turn
  (about 25 times the price of re-read tokens) the extra text is cheaper, so tokens rank below AI understanding
  here as ADR 1077 orders them.
- A phrase made only of method names (`split lines`) is answered as names, not ranked. Both methods are what the
  phrase asks for. A phrase a synonym row answers (`exit code`) stays a phrase.
- `test`, `tests`, `where`, `used`, `called` can no longer be the bare subject of a question; an id (`fn:test`)
  still names one.

## Rejected alternatives

- **A new command (`ail doc NAME`).** The panel of 2026-11-15 found agents use what they already reach for; a new
  door is one more to guess. `doc` and `docs` now suggest `ask`.
- **Print the whole rule paragraph.** Several `type.*` rules define five or more methods in one paragraph; the
  sentences after the next signature are about another method.
- **Run the meant command for an unknown word.** `ail compile x.ail` writing an executable under a name the user
  did not type hides the real command from the agent's context; the answer line teaches it once.
- **A copy of the foreign-name table for the lookup.** Two tables drift (panel item F).
- **Free-text questions answered by search.** An answer to a different question than the one asked is worse than
  an error that lists the forms.
