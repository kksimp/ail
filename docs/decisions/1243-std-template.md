# 1243. `std.template`: text templates over `Json`

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 17 — Standard library
- **Spec rules:** std.template.1-12 ([std-template](../spec/std-template.md))

## Context

The std gap plan (wave 3) lists a text template module. Programs that make HTML, email, reports and config
files by string interpolation get escaping wrong, and the corpus task `template_fill` is a hand-written
placeholder replacer. Python has `string.Template`, Jinja and Mustache; Go has `text/template` and `html/template`.

## Decision

A module written in AIL, `std/template/template.ail`, in scope without `use`: `render`, `render_text`, `compile`,
`Template.render`, `Template.render_text`, `escape_html`, `TemplateError` and `TemplateKind`. The syntax is
Mustache's `{{name}}` with the block forms of Handlebars (`#each`, `#if`, `#unless`, `else`) and the loop names
`@index`, `@first`, `@last`, `@key`, `@count`. Data is a `Json`, so a struct goes in through `to_json()`.

- **Escaping is on by default** for `{{name}}` (HTML: five characters), off for `{{{name}}}`, `{{&name}}` and
  `render_text`. A missing escape is the costly mistake, so the safe form is the short one.
- **A missing name is an error** (Missing), not an empty string: a typo fails at the first render. `#if` and
  `#unless` treat it as false, so they can test for presence.
- **Truthiness** is JSON's, with `0` true (not JavaScript's).
- **Standalone block tags leave no line** (std.template.6), as in Mustache, so indented loops and ifs make clean text.
- **Braces in AIL strings:** `"{{x}}"` is a lex error (`lex.str.2`). Templates go in `r"""..."""` or come from a
  file. The template's own escape for a literal `{{` is `\{{`, which reads the same in a raw string.
- **Name `template`:** 4 local bindings in the tree (`std/gfx/host.ail`, `std/regex/regex.ail`'s internal fn,
  corpus), against 526 for `text`. `tmpl` has 0 but nobody searches for it.
- Pure AIL: no lowering, runtime or checker change. Registered in `names.ail`, `load.ail`, `stduse.ail`,
  `lower_mod.ail` and `lower_err.ail`.

## Rejected alternatives

- **`{name}` holes (like `str.format`).** Conflicts with string interpolation, so every template in an AIL string
  would need escapes.
- **A typed context (a trait or a map) instead of `Json`.** `Json` already is the universal tree, derives from
  structs and is what config, HTTP and databases give.
- **Partials, helpers, lambdas, whitespace-control marks.** They make a template a programming language. A
  program that needs them builds text in AIL; they can be added without breaking templates.
- **Missing name renders empty (Mustache).** Hides typos; an agent cannot see its mistake.
- **Escaping off by default.** The cross-site scripting mistake is silent.
- **Evaluating expressions in tags (`{{a + b}}`).** Not needed for the stated uses and needs a parser and a
  type story.

## Consequences

Not yet compiled: the first Mac gate is its first build; the golden `tests/cases/std_template.out` comes from a
Python mirror of the algorithm.
