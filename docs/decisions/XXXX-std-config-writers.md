# TBD. `toml.write` and `yaml.write`, and `inf`/`nan` in the TOML and YAML readers

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked for a fuller std on 2026-11-26
- **Topic:** 17 — Standard library
- **Spec rules:** [std.toml.9-12, std.yaml.11-14](../spec/17-std.md)

## Context

`std.toml` and `std.yaml` read config text into `Json`; `std.ini` already has `write` (ADR 1220). A program that
changes a config file or generates one had no way back from `Json` to TOML or YAML text and wrote strings by hand,
with the quoting mistakes that follow. Both readers also lacked the TOML `inf`/`nan` floats and the YAML core-schema
`.inf`/`.nan`.

## Decision

- `toml.write(j Json) str ! TomlWriteError` and `yaml.write(j Json) str`, pure AIL in the existing files, next to the
  readers (no runtime, checker or registration work; the modules are already registered).
- Output is canonical, not format-preserving: member order is kept, comments and layout are not (the `Json` value
  keeps neither).
- TOML: values first, then `[a.b]` tables and `[[a]]` arrays of tables, because TOML requires it. The only reorder
  is a member that sat after a table. Dates read as `Str` are written as quoted strings (std.toml.11).
- TOML has no null: `write` fails with `TomlWriteError(why, path)` naming the key. The root must be a table.
- YAML: block style, two spaces, `- key: v` for maps in lists, flow style for lists in lists and inside flow
  collections. Every `Json` has YAML text, so `write` cannot fail. Strings are plain only when `parse` gives the
  same string back; `yes`/`no`/`on`/`off` are quoted too for readers that take them as booleans.
- Whole numbers below 1e15 print as integers (as `Json.to_text` does, json.print.1); infinity and NaN print as
  `inf`/`nan` (TOML) and `.inf`/`.nan` (YAML), which the readers now accept (TOML 1.0, YAML 1.2 core schema).
- INI needs nothing: `ini.write` exists (ADR 1220).
- No syntax changes.

## Rejected alternatives

- **Preserving comments and layout (a document model).** Needs a second value type beside `Json`, so every accessor
  would be written twice. A program that must keep comments edits the text.
- **Writing date-shaped strings as TOML dates.** The reader keeps dates as `Str`, so a real string such as a tag
  `2026-11-27` would silently change type. Quoting is always correct as a string; a date field is not lost in the
  `Json` value.
- **Writing `null` as an empty value or leaving the key out.** Both change the data without a signal; the error
  names the path (ADR 1077: AI understanding first).
- **`yaml.write` returning an error for unsupported values.** There are none: the subset writer covers all of `Json`.
- **Block scalars (`|`) for multi-line strings.** Double-quoted `\n` is one rule and round-trips through the reader;
  `|` needs chomping and indentation decisions per string.
- **Unicode bare keys in TOML.** TOML 1.0 allows ASCII only; the writer quotes anything else, which every reader takes.
