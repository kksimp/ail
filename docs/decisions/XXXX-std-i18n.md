# XXXX. `std.i18n`: message catalogs (number assigned by the integrator)

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 17 — Standard library
- **Spec rules:** std.i18n.1-14 ([std-i18n](../spec/std-i18n.md))

## Context

The std gap plan lists localisation. Programs that print to people in more than one language need a lookup by
message, plural forms that follow the language (Russian has three, Arabic six), a fallback from `fr-CA` to `fr`
to `en`, and a way to fill values into a translated text. gettext, Python's `gettext`, Go's `x/text/message`,
Swift's `String Catalog` and Java's `ResourceBundle` all cover this; translators already work in `.po` files.

## Decision

A module written in AIL, `std/i18n/i18n.ail`, in scope without `use`, pure text and bytes (the caller reads files):

- **gettext is the model**: the source text is the key, so an untranslated program still reads correctly; plural
  messages are keyed by their singular; contexts (`msgctxt`) separate words. `from_po`, `from_mo` and `from_json`
  build a `Catalog`.
- **Plural rules are gettext expressions** (`nplurals=3; plural=...`). A small evaluator reads a `.po` file's own
  header; a table of the common languages gives the default by locale. One engine serves both.
- **Fallback chains** (`zh-Hant-TW` then `zh-Hant` then `zh`) and `Bundle` (first catalog with the message wins) are
  values, not global state: no process-wide current locale, which would make a program's output depend on hidden
  state and break local reasoning.
- **`format`** fills `{name}` from a `Map[str, str]`; a missing name stays visible and does not fail, because a
  translation with a wrong hole should not take a server down.
- **`negotiate`** reads an `Accept-Language` header for the HTTP case.
- Name `i18n`: no local binding of that name exists in the tree.
- Pure AIL: no lowering, runtime or checker change. Registered in `names.ail`, `load.ail`, `stduse.ail`,
  `lower_mod.ail` and `lower_err.ail`.

## Rejected alternatives

- **A process-wide current locale (`i18n.set_locale`).** Hidden global state; tests and servers with many
  locales at once need values.
- **ICU message syntax (`{n, plural, one {...} other {...}}`).** A second language inside strings, and braces in AIL
  strings are already costly (`lex.str.2`). gettext plurals do the same job.
- **CLDR plural categories (`one`, `few`, `many`, `other`).** `.po` and `.mo` files carry gettext indexes, so a
  CLDR layer would need a mapping per language and would disagree with the files translators ship.
- **Reading files in the module.** Keeps it pure (`eff.infer.1`) and testable; `fs.read_text` is one call.
- **Locale-aware number and date formatting.** A large data problem of its own; a later module can add it.

## Consequences

Not yet compiled: the first Mac gate is its first build; the golden `tests/cases/std_i18n.out` comes from a Python
mirror of the plural evaluator and hand-checked messages.
